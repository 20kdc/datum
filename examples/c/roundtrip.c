/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "datum.h"

#define FILE_BUF_SIZE 0x10000

char buf_input[FILE_BUF_SIZE + 1];
char buf_compare[FILE_BUF_SIZE + 1];
char buf_output[FILE_BUF_SIZE + 1];
char * buf_output_ptr = buf_output;

datum_str_t readfile(char * buf, const char * fn) {
	size_t pos;
	datum_str_t str;
	size_t length;
	FILE * f = fopen(fn, "rb");
	if (!f) {
		printf("invalid file %s", fn);
		exit(1);
	}
	fseek(f, 0, SEEK_END);
	length = (size_t) ftell(f);
	if (length >= FILE_BUF_SIZE) {
		printf("file %s is too big", fn);
		exit(1);
	}
	fseek(f, 0, SEEK_SET);
	for (pos = 0; pos < length; pos++)
		buf[pos] = fgetc(f);
	fclose(f);
	str.start = buf;
	str.end = buf + length;
	return str;
}

int outputfn(int c, void * stream) {
	if (buf_output_ptr == (buf_output + FILE_BUF_SIZE))
		return -1;
	*(buf_output_ptr++) = c;
	return 0;
}

datum_outf_t outputstr = {outputfn, NULL};

void dumpstr(const datum_str_t * str) {
	const char * report_ptr = str->start;
	while (report_ptr != str->end)
		putchar(*(report_ptr++));
}

void dumpoutput() {
	const char * report_ptr = buf_output;
	while (report_ptr != buf_output_ptr)
		putchar(*(report_ptr++));
}

int main(int argc, char ** argv) {
	datum_str_t input, compare;
	int listdepth = 0;
	int space = 0;
	if (argc != 3) {
		puts("roundtrip INPUT COMPARE");
		return 1;
	}
	input = readfile(buf_input, argv[1]);
	compare = readfile(buf_compare, argv[2]);
	/* tokenize input and write to output inline */
	while (input.start != input.end) {
		datum_str_t content;
		datum_tknty_t tknt = datum_tkn_string(&input, &content);
		int isatom = 0;
		datum_atom_t atom;
		if (tknt == DATUM_TKNTY_ERROR) {
			printf("roundtrip: Unexpected error reading %s - dumping output so far", argv[1]);
			dumpoutput();
			return 1;
		}
		if (tknt == DATUM_TKNTY_NONE)
			continue;
		if (space) {
			if (tknt != DATUM_TKNTY_LIST_END)
				outputfn(' ', NULL);
			space = 0;
		}
		content.end = datum_cdec_collapse((char *) content.start, (char *) content.end);
		if (tknt == DATUM_TKNTY_LIST_START)
			listdepth++;
		else if (tknt == DATUM_TKNTY_LIST_END)
			listdepth--;
		else {
			/* This should be an atom. */
			if (datum_atom_parse(tknt, &content, &atom)) {
				fputs("roundtrip: '", stdout);
				dumpstr(&content);
				fputs("' did not read as atom\n", stdout);
			} else {
				isatom = 1;
			}
		}
		/* Write either as a token directly or an atom. */
		if (isatom) {
			if (datum_atom_write(&atom, &outputstr)) {
				printf("roundtrip: Unexpected error atomwrite %s - dumping output so far", argv[1]);
				dumpoutput();
				return 1;
			}
		} else if (datum_tkn_write(tknt, &content, &outputstr)) {
			printf("roundtrip: Unexpected error mimicking %s - dumping output so far", argv[1]);
			dumpoutput();
			return 1;
		}
		if (tknt != DATUM_TKNTY_LIST_START)
			space = 1;
		if (listdepth == 0) {
			outputfn('\n', NULL);
			space = 0;
		}
	}
	if (((compare.end - compare.start) != (buf_output_ptr - buf_output)) || memcmp(buf_compare, buf_output, buf_output_ptr - buf_output)) {
		printf("roundtrip: comparison failure for %s, %s\n", argv[1], argv[2]);
		dumpoutput();
		return 1;
	}
	return 0;
}
