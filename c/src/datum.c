/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <stdlib.h>
#include <time.h>

#include "datum.h"

DATUM_API int datum_str_cstr_eq(const datum_str_t * str, const char * cstr) {
	size_t str_len = str->end - str->start;
	size_t cstr_len = strlen(cstr);
	if (str_len != cstr_len)
		return 0;
	return !memcmp(str->start, cstr, str_len);
}

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
	} else if (((c >= 0) && (c < ' ')) || c == 127 || c == '\\') {
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
			if (input == 'x') {
				*state = DATUM_CDEC_STATE_HEX(0);
				return 0;
			}
			/* If not a hex escape, will always end in default state. */
			*state = DATUM_CDEC_STATE_DEFAULT;
			if (input == 'r') {
				input = '\r';
			} else if (input == 'n') {
				input = '\n';
			} else if (input == 't') {
				input = '\t';
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
	switch (*state) {
		/* Default state (eating whitespace) */
		case DATUM_TKN_STATE_DEFAULT:
		default:
			if (chrc & DATUM_CHRC_NONPRINT) {
				/* nothing interesting */
				return DATUM_TKN_CORE_ACT_NOP;
			} else if (chrc & DATUM_CHRC_ALONETKN) {
				/* 'alone token' (single-char) */
				int aloneToken = (chrc & DATUM_CHRC_ALONETKN_MASK) >> DATUM_CHRC_ALONETKN_SHIFT;
				return DATUM_TKN_CORE_ACT_END_ALONE | aloneToken;
			} else if (chrc == DATUM_CHRC_LINE_COMMENT) {
				*state = DATUM_TKN_STATE_LINE_COMMENT;
				return DATUM_TKN_CORE_ACT_NOP;
			} else if (chrc == DATUM_CHRC_STRING) {
				/* drop quotes */
				*state = DATUM_TKN_STATE_STRING;
				return DATUM_TKN_CORE_ACT_START_POST;
			} else if (chrc == DATUM_CHRC_SPECIAL_ID) {
				/* special ID drops its prefix char */
				*state = DATUM_TKN_STATE_PID_SPECIALID;
				return DATUM_TKN_CORE_ACT_START_POST;
				/* all other PID routes do not */
			} else if (chrc == DATUM_CHRC_DIGIT) {
				*state = DATUM_TKN_STATE_PID_DIGIT;
				return DATUM_TKN_CORE_ACT_START_PRE;
			} else if (chrc == DATUM_CHRC_SIGN) {
				*state = DATUM_TKN_STATE_PID_SIGN;
				return DATUM_TKN_CORE_ACT_START_PRE;
			}
			*state = DATUM_TKN_STATE_PID;
			return DATUM_TKN_CORE_ACT_START_PRE;
		case DATUM_TKN_STATE_LINE_COMMENT:
			if (chrc == DATUM_CHRC_NEWLINE)
				*state = DATUM_TKN_STATE_DEFAULT;
			return DATUM_TKN_CORE_ACT_NOP;
		case DATUM_TKN_STATE_STRING:
			if (chrc == DATUM_CHRC_EOF) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_PRE | DATUM_TKNTY_ERROR;
			} else if (chrc == DATUM_CHRC_STRING) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_POSTSKIP | DATUM_TKNTY_STRING;
			}
			return DATUM_TKN_CORE_ACT_NOP;
		/* PID loop always continues until not VALIDPID (EOF is obviously not VALIDPID) */
		case DATUM_TKN_STATE_PID_DIGIT:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_PRE | DATUM_TKNTY_NUMERIC;
			}
			return DATUM_TKN_CORE_ACT_NOP;
		case DATUM_TKN_STATE_PID_SIGN:
			/* Sign either becomes ID immediately or switches to digit codepath */
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_PRE | DATUM_TKNTY_ID;
			}
			*state = DATUM_TKN_STATE_PID_DIGIT;
			return DATUM_TKN_CORE_ACT_NOP;
		case DATUM_TKN_STATE_PID_SPECIALID:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_PRE | DATUM_TKNTY_SPECIAL_ID;
			}
			return DATUM_TKN_CORE_ACT_NOP;
		case DATUM_TKN_STATE_PID:
			if (!(chrc & DATUM_CHRC_VALIDPID)) {
				*state = DATUM_TKN_STATE_DEFAULT;
				return DATUM_TKN_CORE_ACT_END_PRE | DATUM_TKNTY_ID;
			}
			return DATUM_TKN_CORE_ACT_NOP;
	}
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
	const char * cdecStart, * cdecEnd;
	cdecStart = cdecEnd = content->start = content->end = input->start;
	/*
	 * To ensure input->start only moves in CDEC-aligned units,
	 *  our read pointer is cdecEnd, which is the end of the current CDEC unit.
	 */
	while (cdecEnd != input->end) {
		char chr = *(cdecEnd++);
		int cls;
		cdecIfo = datum_cdec_decode(&cdecState, chr);
		if (cdecIfo & DATUM_CDEC_ERROR)
			goto error;
		if (!(cdecIfo & DATUM_CDEC_PRESENT))
			continue;
		/*
		 * Got a 'real character'.
		 * For "\n", cdecStart is at the start while cdecEnd is at the end (so empty string here).
		 */
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
		switch (tknIfo & DATUM_TKN_CORE_ACT_MASK) {
			default:
			case DATUM_TKN_CORE_ACT_NOP:
				break;
			case DATUM_TKN_CORE_ACT_START_PRE:
				content->start = content->end = cdecStart;
				break;
			case DATUM_TKN_CORE_ACT_START_POST:
				content->start = content->end = cdecEnd;
				break;
				/*
				 * All 'END' ACTs set input->start and content->end, then return.
				 * Some do that + other things.
				 */
			case DATUM_TKN_CORE_ACT_END_PRE:
				input->start = content->end = cdecStart;
				return tknIfo & DATUM_TKN_CORE_TOKEN_MASK;
			case DATUM_TKN_CORE_ACT_END_POST:
				input->start = content->end = cdecEnd;
				return tknIfo & DATUM_TKN_CORE_TOKEN_MASK;
			case DATUM_TKN_CORE_ACT_END_POSTSKIP:
				content->end = cdecStart;
				input->start = cdecEnd;
				return tknIfo & DATUM_TKN_CORE_TOKEN_MASK;
			case DATUM_TKN_CORE_ACT_END_ALONE:
				content->start = input->start = content->end = cdecEnd;
				return tknIfo & DATUM_TKN_CORE_TOKEN_MASK;
		}
		/* Finally, advance CDEC window. */
		cdecStart = cdecEnd;
	}
	/*
	 * EOF hit.
	 * The following are absolutes.
	 * errorEOF helps express this (we still have to adjust content->start for error)
	 */
	content->end = input->start = input->end;
	/* check for incomplete CDEC */
	if (cdecState)
		return DATUM_TKNTY_ERROR;
	tknIfo = datum_tkn_core(&tknState, DATUM_CHRC_EOF);
	if (tknIfo & DATUM_TKN_CORE_ACT_END_FLAG)
		return tknIfo & DATUM_TKN_CORE_TOKEN_MASK;
	/* Since we're returning no token, act like it */
	content->start = input->end;
	return DATUM_TKNTY_NONE;

	/*
	 * Most error handlers follow this format for consistency.
	 * Input is advanced so that 'naive forgiving loops' will not freeze.
	 */
	error:
	content->end = cdecEnd;
	input->start = cdecEnd;
	return DATUM_TKNTY_ERROR;
}

#define DATUM_TKNWR_PUT(C) { if (outf->put(C, outf->stream) < 0) error |= DATUM_TKNWR_IOERROR; }

static const char datum_tknwr_hex[16] = "0123456789abcdef";

DATUM_API int datum_tkn_escape(datum_tknwr_escape_t mode, int cannotEscape, char c, const datum_outf_t * outf) {
	int error = 0;
	int cls = datum_chrc_identify(c);
	/* Things that always MUST be escaped to be written. */
	if (c == '\n') {
		if (cannotEscape)
			return DATUM_TKNWR_UNREPRESENTABLE;
		DATUM_TKNWR_PUT('\\');
		DATUM_TKNWR_PUT('n');
		goto complete;
	} else if (c == '\r') {
		if (cannotEscape)
			return DATUM_TKNWR_UNREPRESENTABLE;
		DATUM_TKNWR_PUT('\\');
		DATUM_TKNWR_PUT('r');
		goto complete;
	} else if (c == '\t') {
		if (cannotEscape)
			return DATUM_TKNWR_UNREPRESENTABLE;
		DATUM_TKNWR_PUT('\\');
		DATUM_TKNWR_PUT('t');
		goto complete;
	} else if (c == '\\') {
		goto trivial;
	} else if (cls == DATUM_CHRC_UNCLASSIFIED) {
		if (cannotEscape)
			return DATUM_TKNWR_UNREPRESENTABLE;
		DATUM_TKNWR_PUT('\\');
		DATUM_TKNWR_PUT('x');
		DATUM_TKNWR_PUT(datum_tknwr_hex[(c >> 4) & 0xF]);
		DATUM_TKNWR_PUT(datum_tknwr_hex[c & 0xF]);
		DATUM_TKNWR_PUT(';');
		goto complete;
	}
	/* Per-case logic. Assumes cannotEscape all by itself. */
	switch (mode) {
		case DATUM_TKNWR_ESCAPE_STRING:
			if (c == '\"')
				goto trivial;
			break;
		case DATUM_TKNWR_ESCAPE_VALIDPID:
			if (!(cls & DATUM_CHRC_VALIDPID))
				goto trivial;
			break;
		case DATUM_TKNWR_ESCAPE_NUMSTART:
			if (!(cls & DATUM_CHRC_NUMSTART))
				goto trivial;
			break;
		case DATUM_TKNWR_ESCAPE_DIGIT:
			if (cls != DATUM_CHRC_DIGIT)
				goto trivial;
			break;
		case DATUM_TKNWR_ESCAPE_CONTENT:
			if (cls != DATUM_CHRC_CONTENT)
				goto trivial;
			break;
		case DATUM_TKNWR_ESCAPE_CONTENT_OR_SIGN:
			if (cls != DATUM_CHRC_CONTENT && cls != DATUM_CHRC_SIGN)
				goto trivial;
			break;
	}
	/* No need to escape (or only required a backslash) */
	goto unnecessary;
	trivial:
	if (cannotEscape)
		return DATUM_TKNWR_UNREPRESENTABLE;
	DATUM_TKNWR_PUT('\\');
	unnecessary:
	DATUM_TKNWR_PUT(c);
	complete:
	return error;
}

DATUM_API int datum_tkn_write(datum_tknty_t token, const datum_str_t * content, const datum_outf_t * outf) {
	int error = 0;
	const char * contentPtr = content->start;
	switch (token) {
		case DATUM_TKNTY_STRING:
			DATUM_TKNWR_PUT('"');
			while (contentPtr != content->end) {
				char c = *(contentPtr++);
				error |= datum_tkn_escape(DATUM_TKNWR_ESCAPE_STRING, 0, c, outf);
			}
			DATUM_TKNWR_PUT('"');
			return error;
		case DATUM_TKNTY_SPECIAL_ID:
			DATUM_TKNWR_PUT('#');
			goto validpid;
		case DATUM_TKNTY_ID:
			if (contentPtr == content->end)
				return DATUM_TKNWR_UNREPRESENTABLE;
			/*
			 * First character has to be CONTENT so this ends up an ID, unless the only character.
			 * If it's the only character, it can be CONTENT or SIGN.
			 */
			error |= datum_tkn_escape(
				((contentPtr + 1) == content->end) ? DATUM_TKNWR_ESCAPE_CONTENT_OR_SIGN : DATUM_TKNWR_ESCAPE_CONTENT,
				0,
				*contentPtr,
				outf
			);
			contentPtr++;
			/*
			 * Generic VALIDPID path.
			 */
			validpid:
			while (contentPtr != content->end) {
				char c = *(contentPtr++);
				error |= datum_tkn_escape(DATUM_TKNWR_ESCAPE_VALIDPID, 0, c, outf);
			}
			return error;
		case DATUM_TKNTY_NUMERIC:
			if (contentPtr == content->end)
				return DATUM_TKNWR_UNREPRESENTABLE;
			/*
			 * First character has to be DIGIT or SIGN (NUMSTART).
			 * But if it is SIGN, it must be followed by *anything* to be valid.
			 * So we lock out SIGN if content length is 1.
			 */
			error |= datum_tkn_escape(
				((contentPtr + 1) == content->end) ? DATUM_TKNWR_ESCAPE_DIGIT : DATUM_TKNWR_ESCAPE_NUMSTART,
				1,
				*contentPtr,
				outf
			);
			contentPtr++;
			/*
			 * We now funnel back into the VALIDPID path.
			 * Tokenizer knows this is NUMERIC and will parse as VALIDPID.
			 */
			goto validpid;
		case DATUM_TKNTY_LIST_START:
			DATUM_TKNWR_PUT('(');
			return error;
		case DATUM_TKNTY_LIST_END:
			DATUM_TKNWR_PUT(')');
			return error;
		default:
			return DATUM_TKNWR_UNREPRESENTABLE;
	}
}

DATUM_API int datum_atom_parse_num(const datum_str_t * content, datum_atom_t * atom) {
	/*
	 * This is where things get 'controversial' in terms of sensible memory behaviour.
	 * To use strtod, we need a traditional C string.
	 * So, we need to copy across the number.
	 */
	size_t numSz = content->end - content->start;
	char * tmp = (void *) malloc(numSz + 1);
	char * endptr = NULL;
	if (!tmp)
		return 1;
	memcpy(tmp, content->start, numSz);
	tmp[numSz] = 0;
	/* check if this might be a valid integer */
	if (tmp[0] == '-') {
		if (strspn(tmp + 1, "0123456789") == numSz - 1)
			goto parseint; /* valid negative integer */
	} else if (strspn(tmp, "0123456789") == numSz) {
		/* valid positive integer */
		parseint:
		atom->type = DATUM_ATOMTY_INT;
		atom->content.intnum = strtoll(tmp, &endptr, 10);
		goto done;
	}
	/* whatever approach was chosen failed, so try as float */
	atom->type = DATUM_ATOMTY_FLOAT;
	atom->content.fpnum = strtod(tmp, &endptr);
	done:
	free(tmp);
	return endptr != (tmp + numSz);
}

#ifndef INFINITY
#define INFINITY (1.0 / 0.0)
#endif

#ifndef NAN
#define NAN (-(0.0 / 0.0))
#endif

DATUM_API int datum_atom_parse(datum_tknty_t token, const datum_str_t * content, datum_atom_t * atom) {
	const char * rdptr = content->start;
	/* Set this 'just to be sure'. */
	atom->type = DATUM_ATOMTY_NIL;
	switch (token) {
		default:
			/* This cannot be converted. */
			return 1;
		case DATUM_TKNTY_STRING:
			atom->type = DATUM_ATOMTY_STRING;
			atom->content.symbol = *content;
			return 0;
		case DATUM_TKNTY_ID:
			atom->type = DATUM_ATOMTY_SYMBOL;
			atom->content.symbol = *content;
			return 0;
		case DATUM_TKNTY_SPECIAL_ID:
			if (datum_str_cstr_eq(content, "nil")) {
				atom->type = DATUM_ATOMTY_NIL;
				return 0;
			} else if (datum_str_cstr_eq(content, "f")) {
				atom->type = DATUM_ATOMTY_FALSE;
				return 0;
			} else if (datum_str_cstr_eq(content, "t")) {
				atom->type = DATUM_ATOMTY_TRUE;
				return 0;
			} else if (datum_str_cstr_eq(content, "{}#")) {
				/* Empty symbol, so 'fake it'. */
				atom->type = DATUM_ATOMTY_SYMBOL;
				atom->content.symbol.start = atom->content.symbol.end = content->start;
				return 0;
			} else if (datum_str_cstr_eq(content, "i+inf.0")) {
				atom->type = DATUM_ATOMTY_FLOAT;
				atom->content.fpnum = INFINITY;
				return 0;
			} else if (datum_str_cstr_eq(content, "i-inf.0")) {
				atom->type = DATUM_ATOMTY_FLOAT;
				atom->content.fpnum = -INFINITY;
				return 0;
			} else if (datum_str_cstr_eq(content, "i+nan.0")) {
				atom->type = DATUM_ATOMTY_FLOAT;
				atom->content.fpnum = NAN;
				return 0;
			} else if (content->start != content->end && content->start[0] == 'x') {
				/* Hexadecimal integer. */
				atom->type = DATUM_ATOMTY_INT;
				atom->content.intnum = 0;
				rdptr++;
				while (rdptr != content->end) {
					char chr = *(rdptr++);
					atom->content.intnum <<= 4;
					if (chr >= '0' && chr <= '9') {
						atom->content.intnum += chr - '0';
					} else if (chr >= 'a' && chr <= 'f') {
						atom->content.intnum += (chr - 'a') + 0xA;
					} else if (chr >= 'A' && chr <= 'F') {
						atom->content.intnum += (chr - 'A') + 0xA;
					} else {
						return 1;
					}
				}
				return 0;
			}
			return 1;
		case DATUM_TKNTY_NUMERIC:
			return datum_atom_parse_num(content, atom);
	}
}

/*
 * Like datum_atom_parse. Performs content collapse internally.
 */
DATUM_API int datum_atom_collapse_parse(datum_tknty_t token, datum_str_t * content, datum_atom_t * atom) {
	 content->end = datum_cdec_collapse((char *) content->start, (char *) content->end);
	 return datum_atom_parse(token, content, atom);
}

DATUM_API int datum_atom_write(const datum_atom_t * atom, const datum_outf_t * outf) {
	int error = 0;
	/* 20 digits 'should be enough' it seems, make it 32 to be safe */
	char fmtbuf[32];
	char * fmtptr = fmtbuf;
	switch (atom->type) {
		case DATUM_ATOMTY_NIL:
			DATUM_TKNWR_PUT('#');
			DATUM_TKNWR_PUT('n');
			DATUM_TKNWR_PUT('i');
			DATUM_TKNWR_PUT('l');
			break;
		case DATUM_ATOMTY_TRUE:
			DATUM_TKNWR_PUT('#');
			DATUM_TKNWR_PUT('t');
			break;
		case DATUM_ATOMTY_FALSE:
			DATUM_TKNWR_PUT('#');
			DATUM_TKNWR_PUT('f');
			break;
		case DATUM_ATOMTY_STRING:
			error |= datum_tkn_write(DATUM_TKNTY_STRING, &atom->content.string, outf);
			break;
		case DATUM_ATOMTY_SYMBOL:
			if (atom->content.symbol.start == atom->content.symbol.end) {
				DATUM_TKNWR_PUT('#');
				DATUM_TKNWR_PUT('{');
				DATUM_TKNWR_PUT('}');
				DATUM_TKNWR_PUT('#');
			} else {
				error |= datum_tkn_write(DATUM_TKNTY_ID, &atom->content.symbol, outf);
			}
			break;
		case DATUM_ATOMTY_INT:
			sprintf(fmtbuf, "%" PRIi64, atom->content.intnum);
			goto formatted;
		case DATUM_ATOMTY_FLOAT:
			if (isnan(atom->content.fpnum)) {
				strcpy(fmtbuf, "#i+nan.0");
				goto formatted;
			} else if (isinf(atom->content.fpnum) == 1) {
				strcpy(fmtbuf, "#i+inf.0");
				goto formatted;
			} else if (isinf(atom->content.fpnum) == -1) {
				strcpy(fmtbuf, "#i-inf.0");
				goto formatted;
			}
			sprintf(fmtbuf, "%.17g", atom->content.fpnum);
			formatted:
			while (*fmtptr)
				DATUM_TKNWR_PUT(*(fmtptr++));
			break;
	}
	return error;
}
