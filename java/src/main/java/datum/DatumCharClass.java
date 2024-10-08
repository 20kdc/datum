/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

/**
 * Utilities to identify characters.
 * Created 15th February 2023, turned into an enum the next day.
 */
public enum DatumCharClass {
    //          PID    NS
    Content    (true,  false, null),
    Whitespace (false, false, null),
    Newline    (false, false, null),
    LineComment(false, false, null),
    String     (false, false, null),
    ListStart  (false, false, DatumTokenType.ListStart),
    ListEnd    (false, false, DatumTokenType.ListEnd),
    SpecialID  (true,  false, null),
    Digit      (true,  true,  null),
    Sign       (true,  true,  null),
    Meta       (false, false, null);

    /**
     * If true, this is valid in potential identifiers.
     */
    public final boolean isValidPID;

    /**
     * If true, this is a numeric start.
     */
    public final boolean isNumericStart;

    /**
     * If non-null, this character class is supposed to represent an alone token.
     */
    public final DatumTokenType aloneToken;

    DatumCharClass(boolean pid, boolean ns, DatumTokenType alone) {
        isValidPID = pid;
        isNumericStart = ns;
        aloneToken = alone;
    }

    public static DatumCharClass identify(char c) {
        if (c == '\n') {
            return Newline;
        } else if (c == '\t' || c == ' ') {
            return Whitespace;
        } else if (c < ' ' || c == 127 || c == '\\') {
            return Meta;
        } else if (c == ';') {
            return LineComment;
        } else if (c == '"') {
            return String;
        } else if (c == '(') {
            return ListStart;
        } else if (c == ')') {
            return ListEnd;
        } else if (c == '#') {
            return SpecialID;
        } else if (c == '-') {
            return Sign;
        } else if (c >= '0' && c <= '9') {
            return Digit;
        } else {
            return Content;
        }
    }
}
