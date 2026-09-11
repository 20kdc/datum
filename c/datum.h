/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
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
 * Datum-C never allocates strings by itself.
 * As a side-effect of this, it isn't capable of the 'pure streaming' workflow of other implementations.
 * However, this keeps memory management/etc. extremely simple, which is important in a C codebase.
 */
typedef struct datum_str {
	const char * start;
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
 * Character classes.
 */
#define DATUM_CHRC_VALIDPID 0x0100
#define DATUM_CHRC_NUMSTART 0x0200

#define DATUM_CHRC_UNCLASSIFIED  0
#define DATUM_CHRC_CONTENT       (1 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_WHITESPACE    2
#define DATUM_CHRC_NEWLINE       3
#define DATUM_CHRC_LINECOMMENT   4
#define DATUM_CHRC_STRING        5
#define DATUM_CHRC_LISTSTART     6
#define DATUM_CHRC_LISTEND       7
#define DATUM_CHRC_SPECIALID     (8 | DATUM_CHRC_VALIDPID)
#define DATUM_CHRC_DIGIT         (9 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)
#define DATUM_CHRC_SIGN         (10 | DATUM_CHRC_VALIDPID | DATUM_CHRC_NUMSTART)

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
 * Token type.
 */
typedef enum {
	DATUM_TKNT_STRING,
	DATUM_TKNT_ID,
	DATUM_TKNT_SPECIAL_ID,
	DATUM_TKNT_NUMERIC,
	DATUM_TKNT_LIST_START,
	DATUM_TKNT_LIST_END,
	DATUM_TKNT_COUNT
} datum_tknt_t;

/*
 * Tokenizer responses.
 */
#define DATUM_TKNR_INCOMPLETE 0x0100

#define DATUM_TKNR_OK                0
#define DATUM_TKNR_CDEC_INCOMPLETE   (1 | DATUM_TKNR_INCOMPLETE)
#define DATUM_TKNR_STRING_INCOMPLETE (2 | DATUM_TKNR_INCOMPLETE)
#define DATUM_TKNR_CDEC_ERROR        3

/*
 * Tokenize the given input text.
 * Returns a DATUM_TKNR_ code.
 * 'DATUM_TKNR_INCOMPLETE' can be used to test for the obvious.
 * The `input` slice will have its start move forward as tokens are successfully completed.
 * This is exactly aligned to calls to the `token` function.
 * This function is given a 'content' slice into `input`.
 * For strings, for instance, this is the 'inside' of the string.
 * The contents of this slice may safely be mutated if that is allowed for `input`.
 * In particular `datum_chrc_collapse` may be used.
 * If the `token` function returns a non-zero result, tokenization stops immediately.
 * This can be used for a variety of purposes, like one-token-at-a-time retrieval.
 */
/* Not yet implemented. */
DATUM_API int datum_tokenize(datum_str_t * input, int (*token)(void * userdata, datum_tknt_t tokenType, datum_str_t * content), void * userdata);

#ifdef __cplusplus
}
#endif

#endif
