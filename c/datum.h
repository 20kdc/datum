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
 * Note that these have only an 4-bit room to move due to the space of DATUM_TKN_CORE flags.
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
#define DATUM_CHRC_SPACEISH      0x0080

#define DATUM_CHRC_ALONETKN_SHIFT 8
#define DATUM_CHRC_ALONETKN_MASK 0x0F00
#define DATUM_CHRC_ALONETKN_ENC(V) (DATUM_CHRC_ALONETKN | ((V) << DATUM_CHRC_ALONETKN_SHIFT))

#define DATUM_CHRC_UNCLASSIFIED  0
#define DATUM_CHRC_CONTENT       (1 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_WHITESPACE    (2 | DATUM_CHRC_SPACEISH)
#define DATUM_CHRC_NEWLINE       (3 | DATUM_CHRC_SPACEISH)
#define DATUM_CHRC_LINE_COMMENT  4
#define DATUM_CHRC_STRING        5
#define DATUM_CHRC_LIST_START    (6 | DATUM_CHRC_ALONETKN_ENC(DATUM_TKNTY_LIST_START))
#define DATUM_CHRC_LIST_END      (7 | DATUM_CHRC_ALONETKN_ENC(DATUM_TKNTY_LIST_END))
#define DATUM_CHRC_SPECIAL_ID    (8 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_DIGIT         (9 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)
#define DATUM_CHRC_SIGN         (10 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)
/* Special 'end of file/stream' character class. */
#define DATUM_CHRC_EOF          (11 | DATUM_CHRC_SPACEISH)

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
 * This implements the *rules* of tokenization, but doesn't conveniently package them.
 * Tokenizer responses are made of a series of flags.
 * These flags apply in the given order.
 */

/*
 * Before this character, end the current token. Uses DATUM_TKN_CORE_PRE_MASK / DATUM_TKN_CORE_PRE_SHIFT.
 * IMPORTANT RULE: There is an absolute guarantee this event reset the tokenizer after the emitted token.
 * Therefore, you can do the same to safely resume parsing.
 * This means that APIs do not have to emit more than one token at a time.
 */
#define DATUM_TKN_CORE_PRE_END_AND_RESET 0x8000
/* Before this character, start a new token. */
#define DATUM_TKN_CORE_PRE_START         0x4000
/* After this character, end the current token. Uses DATUM_TKN_CORE_POST_MASK / DATUM_TKN_CORE_POST_SHIFT */
#define DATUM_TKN_CORE_POST_END          0x2000
/* After this character, start a new token. */
#define DATUM_TKN_CORE_POST_START        0x1000
/* Indicates an incomplete token error (in response to EOF 'character'). */
#define DATUM_TKN_CORE_ERROR             0x0800

/*
 * This flag indicates POST_END should not include the final character in content.
 * Remember that 'character' here means CDEC unit, so multiple `char` can be in a character.
 * This is used for strings. Strings end proper after the final '"', but that mustn't be included in content.
 */
#define DATUM_TKN_CORE_POST_END_SKIP    0x0400

/* Token type for PRE_END */
#define DATUM_TKN_CORE_PRE_MASK         0x00F0
#define DATUM_TKN_CORE_PRE_SHIFT        4
/* Token type for POST_END */
#define DATUM_TKN_CORE_POST_MASK        0x000F
#define DATUM_TKN_CORE_POST_SHIFT       0

/*
 * Similar to CDEC, state must be initialized to 0.
 * Unlike CDEC, EOF is an explicit character class rather than implied by state.
 * This is because EOF might i.e. end a token normally.
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

#ifdef __cplusplus
}
#endif

#endif
