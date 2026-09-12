/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

/*
 * `datum.h`: datum-c 'core API'.
 * Scope is:
 * * Basic reading/writing
 * * Enough APIs for construction of simple DSLs
 * Scope is NOT:
 * * AST
 * * Atom comprehension (implies floats)
 * * Anything that requires malloc, free
 */

#ifndef _IT_DATUM_C_H_

#define _IT_DATUM_C_H_

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* For Windows export/import, other desired alterations. */

#ifndef DATUM_API
#define DATUM_API
#endif

/*
 * Datum string slice type.
 * This 'core API' of datum-c never allocates strings by itself.
 * As a side-effect of this, any streamed decoding must be managed through the exposed state machines.
 * However, this keeps memory management/etc. extremely simple, which is important in a C codebase.
 */
typedef struct datum_str {
	const char * start;
	/*
	 * End of the slice. The end MUST NOT be before the start.
	 * Therefore, when setting start, set end.
	 */
	const char * end;
} datum_str_t;

/*
 * Source location type.
 */
typedef struct datum_loc {
	/* Arbitrary. Datum does not interact with this, but will forward it through. */
	void * loc;
	int lineNumber;
} datum_loc_t;

/*
 * Token type.
 * Note that these have only an 4-bit room to move due to the space of DATUM_TKN_CORE_TOKEN_MASK.
 */
typedef enum {
	/* This token type value is an intentional 'null'. */
	DATUM_TKNTY_NONE,
	DATUM_TKNTY_STRING,
	DATUM_TKNTY_ID,
	DATUM_TKNTY_SPECIAL_ID,
	DATUM_TKNTY_NUMERIC,
	DATUM_TKNTY_LIST_START,
	DATUM_TKNTY_LIST_END,
	DATUM_TKNTY_COUNT,
	/* Special value for datum_tkn_string. */
	DATUM_TKNTY_ERROR = DATUM_TKNTY_COUNT
} datum_tknty_t;

/* Describes a token type. */
DATUM_API const char * datum_tknty_describe(datum_tknty_t ty);

/*
 * Character classes.
 * Canonically int; bounded to int16_t.
 */
#define DATUM_CHRC_VALIDPID      0x0010
#define DATUM_CHRC_NUMSTART      0x0020
#define DATUM_CHRC_ALONETKN      0x0040
#define DATUM_CHRC_NONPRINT      0x0080

#define DATUM_CHRC_ALONETKN_SHIFT 8
#define DATUM_CHRC_ALONETKN_MASK 0x0F00
#define DATUM_CHRC_ALONETKN_ENC(V) (DATUM_CHRC_ALONETKN | ((V) << DATUM_CHRC_ALONETKN_SHIFT))

#define DATUM_CHRC_UNCLASSIFIED  0
#define DATUM_CHRC_CONTENT       (1 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_WHITESPACE    (2 | DATUM_CHRC_NONPRINT)
#define DATUM_CHRC_NEWLINE       (3 | DATUM_CHRC_NONPRINT)
#define DATUM_CHRC_LINE_COMMENT  4
#define DATUM_CHRC_STRING        5
#define DATUM_CHRC_LIST_START    (6 | DATUM_CHRC_ALONETKN_ENC(DATUM_TKNTY_LIST_START))
#define DATUM_CHRC_LIST_END      (7 | DATUM_CHRC_ALONETKN_ENC(DATUM_TKNTY_LIST_END))
#define DATUM_CHRC_SPECIAL_ID    (8 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_DIGIT         (9 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)
#define DATUM_CHRC_SIGN         (10 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)
/* Special 'end of file/stream' character class. */
#define DATUM_CHRC_EOF          (11 | DATUM_CHRC_NONPRINT)

/* Identifies the character class of a character. */
DATUM_API int datum_chrc_identify(char c);

/* Dummy flag to ensure a 'returned' CDEC result is always non-zero. */
#define DATUM_CDEC_PRESENT   0x80000000
/* Indicates this CDEC result represents something escaped. */
#define DATUM_CDEC_ESCAPED   0x40000000
/* UTF-8 escape flag. */
#define DATUM_CDEC_UTF8      0x20000000
/* Error flag. CDEC can continue regardless. */
#define DATUM_CDEC_ERROR     0x10000000
#define DATUM_CDEC_UTF8_MASK 0x001FFFFF
#define DATUM_CDEC_BYTE_MASK 0x000000FF

/*
 * Character decoder.
 * State must be initialized to 0.
 * If state is not 0 on EOF, an escape was incomplete.
 * This can output one of four things:
 * * Nothing (0)
 * * Unescaped byte (DATUM_CDEC_PRESENT)
 * * Escaped byte (DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED)
 * * Escaped UTF-8 (DATUM_CDEC_PRESENT | DATUM_CDEC_ESCAPED | DATUM_CDEC_UTF8)
 * * An error (DATUM_CDEC_ERROR) may be combined with any of these, including nothing, but also including a character.
 *   It is safe to continue after an error.
 */
DATUM_API uint32_t datum_cdec_decode(uint32_t * state, char input);

/*
 * Writes up to 4 bytes of UTF-8 to the target, returning the end of the written bytes.
 * If the codepoint is not encodable in 4 bytes, it will wrap around.
 * (This may encode codepoints >= U+10FFFF.)
 */
DATUM_API char * datum_cdec_utf8_emit(char * to, uint32_t codepoint);

/*
 * The given slice is assumed to be a mutable 'content string'.
 * It is 'collapsed' into a smaller string; 'end' is mutated accordingly.
 * This function will gracefully ignore syntactic issues (which are detected by `datum_tokenize`.)
 * Therefore, the new `end` is returned. NULL is never returned unless `start == NULL` (and this is presumably valid memory, or else undefined behaviour occurs).
 */
DATUM_API char * datum_cdec_collapse(char * start, char * end);

/*
 * Tokenizer core.
 *
 * This implements the *rules* of tokenization, but doesn't conveniently package them.
 *
 * A character class (corresponding to an input character) is fed in.
 * The tokenizer responds with an ACT, and mutates its state.
 *
 * Remember that 'character' here means CDEC unit, so multiple `char` can be in a character.
 *
 * ACTs are split into:
 * * 'NOP' acts (more decoded characters needed, continue)
 * * 'START' acts (place token start marker before/after current character, continue)
 * * 'END' acts (finish token before/after current character, do not continue. tokenizer reset guaranteed)
 *   * ACT_END_PRE requires the input NOT advance.
 *
 * Errors are returned with 'END' with the error token type.
 *
 * Regarding state:
 *
 * Similar to CDEC, state must be initialized to 0.
 * Unlike CDEC, EOF is an explicit character class rather than implied by state.
 * This is because EOF might i.e. end a token normally.
 * The state == 0 check is still useful; if state == 0, it is a guarantee that the content start/end is meaningless.
 * Conversely, the content start/end does not indicate the true token start, while the state change necessarily does.
 * The state will always be 0 before a token starts, and will always be 0 immediately after it ends.
 */

#define DATUM_TKN_CORE_ACT_MASK              0xF0
/*
 * Token type mask for *_END.
 */
#define DATUM_TKN_CORE_TOKEN_MASK            0x0F

/* Nothing to do here. */
#define DATUM_TKN_CORE_ACT_NOP               0x00

/* Before this character, start a new token. */
#define DATUM_TKN_CORE_ACT_START_PRE         0x10
/* After this character, start a new token. */
#define DATUM_TKN_CORE_ACT_START_POST        0x20

/* All 'END' ACTs have this flag. */
#define DATUM_TKN_CORE_ACT_END_FLAG          0x80

/*
 * Before the input character, end the current token.
 * The tokenizer WILL be reset when this act is returned.
 * Parsing resumes by re-parsing the input character.
 */
#define DATUM_TKN_CORE_ACT_END_PRE           0x80
/* After this character, end the current token. */
#define DATUM_TKN_CORE_ACT_END_POST          0x90

/*
 * Input parsing continues after this character (like END_POST).
 * However, content ends before this character (like END_PRE).
 * This is used for strings. Strings end proper after the final '"', but that mustn't be included in content.
 */
#define DATUM_TKN_CORE_ACT_END_POSTSKIP      0xA0

/*
 * This act is a concatenation of START_POST and END_POST.
 * This creates an empty content buffer.
 */
#define DATUM_TKN_CORE_ACT_END_ALONE         0xB0

/*
 * See above description of tokenizer core.
 */
DATUM_API int datum_tkn_core(int * state, int chrc);

/*
 * Advances an `input` slice. The content slice of the token is placed in `content`.
 * Notably, if a token is returned, then it is guaranteed that input and content will not overlap.
 * Therefore, it is safe to call this in a loop with `datum_cdec_collapse`.
 * (Note however that list start/end tokens will waste time if collapsed unnecessarily.)
 * Returns:
 * * A token type
 * * DATUM_TKNTY_NONE (no token found)
 * * DATUM_TKNTY_ERROR (half-open string, CDEC error...)
 */
DATUM_API datum_tknty_t datum_tkn_string(datum_str_t * input, datum_str_t * content);

typedef enum {
	/* String content. */
	DATUM_TKNWR_ESCAPE_STRING,
	/* Escape anything not VALIDPID. */
	DATUM_TKNWR_ESCAPE_VALIDPID,
	/* Escape anything not NUMSTART. (You probably want to set cannotEscape.) */
	DATUM_TKNWR_ESCAPE_NUMSTART,
	/* Escape anything not DIGIT. (You probably want to set cannotEscape.) */
	DATUM_TKNWR_ESCAPE_DIGIT,
	/* Escape anything not CONTENT. */
	DATUM_TKNWR_ESCAPE_CONTENT,
	/* Escape anything not CONTENT or SIGN. */
	DATUM_TKNWR_ESCAPE_CONTENT_OR_SIGN
} datum_tknwr_escape_t;

/*
 * These are bitflags representing errors.
 */
#define DATUM_TKNWR_UNREPRESENTABLE 1
#define DATUM_TKNWR_IOERROR 2

/*
 * Escapes a single character into an fputc-like function.
 * Same return values as datum_tknwr below.
 */
DATUM_API int datum_tknwr_escape(datum_tknwr_escape_t mode, int cannotEscape, char c, int (*put)(int c, void * stream), void * stream);

/*
 * Writes a token using an fputc-like function.
 * It is assumed negative numbers are errors (i.e. EOF == -1).
 * The function will not abort on IO error, but the error will be reported.
 */
DATUM_API int datum_tknwr(datum_tknty_t token, const datum_str_t * content, int (*put)(int c, void * stream), void * stream);

#ifdef __cplusplus
}
#endif

#endif
