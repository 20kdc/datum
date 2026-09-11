/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#include "datum.h"

DATUM_API int datum_chrc_identify(char c) {
	/* Almost a direct copy/paste from DatumCharClass.java */
	if (c == '\n') {
		return DATUM_CHRC_NEWLINE;
	} else if (c == '\t' || c == ' ') {
		return DATUM_CHRC_WHITESPACE;
	} else if (c < ' ' || c == 127 || c == '\\') {
		return DATUM_CHRC_UNCLASSIFIED;
	} else if (c == ';') {
		return DATUM_CHRC_LINECOMMENT;
	} else if (c == '"') {
		return DATUM_CHRC_STRING;
	} else if (c == '(') {
		return DATUM_CHRC_LISTSTART;
	} else if (c == ')') {
		return DATUM_CHRC_LISTEND;
	} else if (c == '#') {
		return DATUM_CHRC_SPECIALID;
	} else if (c == '-') {
		return DATUM_CHRC_SIGN;
	} else if (c >= '0' && c <= '9') {
		return DATUM_CHRC_DIGIT;
	} else {
		return DATUM_CHRC_CONTENT;
	}
}

/*
 * CDEC state machine guts (private)
 */
#define DATUM_CDEC_STATE_MASK 0xF
#define DATUM_CDEC_STATE_DEFAULT 0
#define DATUM_CDEC_STATE_ESCAPE 1
#define DATUM_CDEC_STATE_HEX(N) (2 | ((N) << 4))

DATUM_API uint32_t datum_cdec_decode(uint32_t * state, char input) {
	uint32_t stateActual, stateSubval;
	/* discard rule */
	if (input == 13)
		return 0;
	stateActual = (*state) & DATUM_CDEC_STATE_MASK;
	switch (stateActual) {
		/* default (not in escape) */
		case DATUM_CDEC_STATE_DEFAULT:
		default:
			if (input == '\\') {
				*state = DATUM_CDEC_STATE_ESCAPE;
				return 0;
			}
			/* Not an escape. If unclassified, we fire off an ERROR. */
			if ((input != '\n') && (input != '\t') && ((input < ' ') || (input == 127)))
				return DATUM_CDEC_PRESENT | DATUM_CDEC_ERROR | (input & DATUM_CDEC_BYTE_MASK);
			return DATUM_CDEC_PRESENT | (input & DATUM_CDEC_BYTE_MASK);
		/* escape (deciding outcome) */
		case DATUM_CDEC_STATE_ESCAPE:
			if (input == 'r') {
				return DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | '\r';
			} else if (input == 'n') {
				return DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | '\n';
			} else if (input == 't') {
				return DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | '\t';
			} else if (input == 'x') {
				*state = DATUM_CDEC_STATE_HEX(0);
				return 0;
			}
			return DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | (input & DATUM_CDEC_BYTE_MASK);
		/* escape hex */
		case DATUM_CDEC_STATE_HEX(0):
			stateSubval = (*state) >> 4;
			if (input >= '0' && input <= '9') {
				*state = DATUM_CDEC_STATE_HEX((stateSubval << 4) | (input - '0'));
				return 0;
			} else if (input >= 'a' && input <= 'f') {
				*state = DATUM_CDEC_STATE_HEX((stateSubval << 4) | ((input - 'a') + 0xA));
				return 0;
			} else if (input >= 'A' && input <= 'F') {
				*state = DATUM_CDEC_STATE_HEX((stateSubval << 4) | ((input - 'A') + 0xA));
				return 0;
			} else {
				/*
				 * Treat any unknown character here as semicolon, but mix in as an error later.
				 * Out of range UTF-8 values are AND'd to be at least properly encodable and also signal error.
				 */
				stateSubval = (stateSubval & DATUM_CDEC_UTF8_MASK) | ((stateSubval > 0x10FFFF) ? DATUM_CDEC_ERROR : 0);
				stateSubval |= DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | DATUM_CDEC_UTF8;
				if (input != ';')
					stateSubval |= DATUM_CDEC_ERROR;
				return stateSubval;
			}
	}
}

DATUM_API char * datum_cdec_utf8_emit(char * to, uint32_t codepoint) {
	uint8_t termOr, termMask, cCount, shift;
	if (codepoint < 0x80) {
		termOr = 0;
		termMask = 0x7F;
		cCount = 0;
	} else if (codepoint < 0x800) {
		termOr = 0xC0;
		termMask = 0x1F;
		cCount = 1;
	} else if (codepoint < 0x10000) {
		termOr = 0xE0;
		termMask = 0x0F;
		cCount = 2;
	} else {
		termOr = 0xF0;
		termMask = 0x07;
		cCount = 3;
	}
	shift = cCount * 6;
	*(to++) = (char) (termOr | ((codepoint >> shift) & termMask));
	while (cCount) {
		cCount--;
		shift -= 6;
		*(to++) = (char) (0x80 | ((codepoint >> shift) & 0x3F));
	}
	return to;
}

DATUM_API char * datum_cdec_collapse(char * start, char * end) {
	char * newend = start;
	uint32_t state = 0;
	while (start != end) {
		char chr = *(start++);
		uint32_t res = datum_cdec_decode(&state, chr);
		if (res & DATUM_CDEC_PRESENT) {
			if (res & DATUM_CDEC_UTF8) {
				newend = datum_cdec_utf8_emit(newend, res & DATUM_CDEC_UTF8_MASK);
			} else {
				*(newend++) = res & DATUM_CDEC_BYTE_MASK;
			}
		}
	}
	return newend;
}

DATUM_API int datum_tokenize(datum_str_t * input, int (*token)(void * userdata, datum_tknt_t tokenType, datum_str_t * content), void * userdata) {
	/* NYI */
	return DATUM_TKNR_INCOMPLETE;
}
