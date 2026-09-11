/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#include "datum.h"

DATUM_API const char * datum_tknty_describe(datum_tknty_t ty) {
	if (ty == DATUM_TKNTY_NONE) return "DATUM_TKNTY_NONE";
	if (ty == DATUM_TKNTY_STRING) return "DATUM_TKNTY_STRING";
	if (ty == DATUM_TKNTY_ID) return "DATUM_TKNTY_ID";
	if (ty == DATUM_TKNTY_SPECIAL_ID) return "DATUM_TKNTY_SPECIAL_ID";
	if (ty == DATUM_TKNTY_NUMERIC) return "DATUM_TKNTY_NUMERIC";
	if (ty == DATUM_TKNTY_LIST_START) return "DATUM_TKNTY_LIST_START";
	if (ty == DATUM_TKNTY_LIST_END) return "DATUM_TKNTY_LIST_END";
	if (ty == DATUM_TKNTY_ERROR) return "DATUM_TKNTY_ERROR";
	return "DATUM_TKNTY_unknown";
}

DATUM_API int datum_chrc_identify(char c) {
	/* Almost a direct copy/paste from DatumCharClass.java */
	if (c == '\n') {
		return DATUM_CHRC_NEWLINE;
	} else if (c == '\t' || c == ' ') {
		return DATUM_CHRC_WHITESPACE;
	} else if (c < ' ' || c == 127 || c == '\\') {
		return DATUM_CHRC_UNCLASSIFIED;
	} else if (c == ';') {
		return DATUM_CHRC_LINE_COMMENT;
	} else if (c == '"') {
		return DATUM_CHRC_STRING;
	} else if (c == '(') {
		return DATUM_CHRC_LIST_START;
	} else if (c == ')') {
		return DATUM_CHRC_LIST_END;
	} else if (c == '#') {
		return DATUM_CHRC_SPECIAL_ID;
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
				*state = DATUM_CDEC_STATE_DEFAULT;
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

#define DATUM_TKN_STATE_DEFAULT 0
#define DATUM_TKN_STATE_LINE_COMMENT 1
#define DATUM_TKN_STATE_STRING 2
#define DATUM_TKN_STATE_PID_DIGIT 3
#define DATUM_TKN_STATE_PID_SIGN 4
#define DATUM_TKN_STATE_PID_SPECIALID 5
#define DATUM_TKN_STATE_PID 6

DATUM_API int datum_tkn_core(int * state, int chrc) {
	int events = 0;
	reinterpret:
	/*
	printf("tkn %i <- %i\n", *state, chrc);
	*/
	switch (*state) {
		/* Default state (eating whitespace) */
		case DATUM_TKN_STATE_DEFAULT:
		default:
			if (chrc & DATUM_CHRC_SPACEISH) {
				/* nothing interesting */
			} else if (chrc & DATUM_CHRC_ALONETKN) {
				/* 'alone token' (single-char) */
				int aloneToken = (chrc & DATUM_CHRC_ALONETKN_MASK) >> DATUM_CHRC_ALONETKN_SHIFT;
				events |= DATUM_TKN_CORE_PRE_START | DATUM_TKN_CORE_POST_END | (aloneToken << DATUM_TKN_CORE_POST_SHIFT);
			} else if (chrc == DATUM_CHRC_LINE_COMMENT) {
				*state = DATUM_TKN_STATE_LINE_COMMENT;
			} else if (chrc == DATUM_CHRC_STRING) {
				/* drop quotes */
				events |= DATUM_TKN_CORE_POST_START;
				*state = DATUM_TKN_STATE_STRING;
			} else if (chrc == DATUM_CHRC_SPECIAL_ID) {
				/* special ID drops its prefix char */
				events |= DATUM_TKN_CORE_POST_START;
				*state = DATUM_TKN_STATE_PID_SPECIALID;
				/* all other PID routes do not */
			} else if (chrc == DATUM_CHRC_DIGIT) {
				events |= DATUM_TKN_CORE_PRE_START;
				*state = DATUM_TKN_STATE_PID_DIGIT;
			} else if (chrc == DATUM_CHRC_SIGN) {
				events |= DATUM_TKN_CORE_PRE_START;
				*state = DATUM_TKN_STATE_PID_SIGN;
			} else {
				events |= DATUM_TKN_CORE_PRE_START;
				*state = DATUM_TKN_STATE_PID;
			}
			break;
		case DATUM_TKN_STATE_LINE_COMMENT:
			if (chrc == DATUM_CHRC_NEWLINE)
				*state = DATUM_TKN_STATE_DEFAULT;
			break;
		case DATUM_TKN_STATE_STRING:
			if (chrc == DATUM_CHRC_EOF) {
				events |= DATUM_TKN_CORE_ERROR | DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_STRING << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
			} else if (chrc == DATUM_CHRC_STRING) {
				events |= DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_STRING << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
			}
			break;
		/* PID loop always continues until not VALIDPID (EOF is obviously not VALIDPID) */
		case DATUM_TKN_STATE_PID_DIGIT:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				events |= DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_NUMERIC << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
				goto reinterpret;
			}
			break;
		case DATUM_TKN_STATE_PID_SIGN:
			/* Sign either becomes ID immediately or switches to digit codepath */
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				events |= DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_ID << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
				goto reinterpret;
			} else {
				*state = DATUM_TKN_STATE_PID_DIGIT;
			}
			break;
		case DATUM_TKN_STATE_PID_SPECIALID:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				events |= DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_SPECIAL_ID << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
				goto reinterpret;
			}
			break;
		case DATUM_TKN_STATE_PID:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				events |= DATUM_TKN_CORE_PRE_END | (DATUM_TKNTY_ID << DATUM_TKN_CORE_PRE_SHIFT);
				*state = DATUM_TKN_STATE_DEFAULT;
				goto reinterpret;
			}
			break;
	}
	return events;
}

DATUM_API datum_tknty_t datum_tkn_string(datum_str_t * input, datum_str_t * content) {
	int tknState = 0;
	uint32_t cdecState = 0;
	int tknIfo;
	uint32_t cdecIfo;
	/*
	 * Say we have `\x1234;` as first token.
	 * 'pre' character in this case should be pointing to '\'.
	 * cdecStart therefore is the start of the current CDEC character.
	 */
	const char * cdecStart;
	cdecStart = content->start = content->end = input->start;
	while (input->start != input->end) {
		char chr = *input->start;
		int cls;
		cdecIfo = datum_cdec_decode(&cdecState, chr);
		if (cdecIfo & DATUM_CDEC_ERROR)
			return DATUM_TKNTY_ERROR;
		if (!(cdecIfo & DATUM_CDEC_PRESENT)) {
			input->start++;
			continue;
		}
		/* got a 'real character', get class */
		if (cdecIfo & DATUM_CDEC_ESCAPED) {
			cls = DATUM_CHRC_CONTENT;
		} else {
			cls = datum_chrc_identify((char) (cdecIfo & DATUM_CDEC_BYTE_MASK));
		}
		/*
		printf("CDEC %c %i\n", (char) (cdecIfo & DATUM_CDEC_BYTE_MASK), cls);
		 */
		/* tokenizer */
		tknIfo = datum_tkn_core(&tknState, cls);
		/* 'pre' phase */
		if (tknIfo & DATUM_TKN_CORE_PRE_END) {
			content->end = cdecStart;
			/* it's safe to stop now, see comment on DATUM_TKN_CORE_PRE_END */
			return (tknIfo & DATUM_TKN_CORE_PRE_MASK) >> DATUM_TKN_CORE_PRE_SHIFT;
		}
		if (tknIfo & DATUM_TKN_CORE_PRE_START)
			content->start = cdecStart;
		/* between phases */
		input->start++;
		/*
		 * We just passed the end of something CDEC considers a character boundary.
		 * This is the only time cdecStart can advance.
		 */
		cdecStart = input->start;
		/* 'post' phase */
		if (tknIfo & DATUM_TKN_CORE_POST_END) {
			content->end = cdecStart;
			return (tknIfo & DATUM_TKN_CORE_POST_MASK) >> DATUM_TKN_CORE_POST_SHIFT;
		}
		if (tknIfo & DATUM_TKN_CORE_POST_START)
			content->start = cdecStart;
	}
	/* check for incomplete CDEC */
	if (cdecState)
		return DATUM_TKNTY_ERROR;
	/* there is no outcome where a token is returned AND content->end is not input->end */
	tknIfo = datum_tkn_core(&tknState, DATUM_CHRC_EOF);
	if (tknIfo & DATUM_TKN_CORE_ERROR)
		return DATUM_TKNTY_ERROR;
	if (tknIfo & DATUM_TKN_CORE_PRE_END) {
		content->end = input->end;
		return (tknIfo & DATUM_TKN_CORE_PRE_MASK) >> DATUM_TKN_CORE_PRE_SHIFT;
	}
	return DATUM_TKNTY_NONE;
}
