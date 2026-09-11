/*
 * datum-c - Quick to implement S-expression format
 * Written starting in 2026 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#include <stdio.h>
#include <string.h>
#include "datum.h"

char testbuf[8192];

int main(int argc, char ** argv) {
	while (1) {
		datum_str_t area, content;
		char * tmp;
		if (!fgets(testbuf, 8192, stdin))
			break;
		tmp = strchr(testbuf, '\n');
		if (!tmp)
			break;
		*tmp = 0;
		area.start = testbuf;
		area.end = testbuf + strlen(testbuf);
		while (1) {
			const char * contentview;
			datum_tknty_t tkn = datum_tkn_string(&area, &content);
			fputs(datum_tknty_describe(tkn), stdout);
			putchar('|');
			contentview = content.start;
			while (contentview != content.end) {
				putchar(*contentview++);
			}
			putchar('|');
			content.end = datum_cdec_collapse((char *) content.start, (char *) content.end);
			contentview = content.start;
			while (contentview != content.end) {
				putchar(*contentview++);
			}
			putchar(10);
			if (tkn == DATUM_TKNTY_NONE)
				break;
		}
	}
}
