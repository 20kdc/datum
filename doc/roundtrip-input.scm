"datum - Terse, human-writable dataformat"
"Written starting in 2024 by contributors (see CREDITS.txt at repository's root)"
"To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty."
"A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>."
""

; How To Use This
; A Datum reader/writer pair that works perfectly should be able to read this file and produce the data in roundtrip-output.scm byte-for-byte.
; That's it.

hello
(hello)
1.23
-6.5
#i+nan.0
#i+inf.0
#i-inf.0
10
-10
-
\-a
#t
#f
#nil
#{}#
(quote hello)
"mi ken \"awen\" e nimi kepeken ni"
escape\ me
"\x7f;\x00;\x10;\r\n\t"
