/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package gabien.datum;

import java.io.IOException;
import java.io.StringWriter;

/**
 * Writes out a Datum (or a stream of them) to a Writer.
 * Includes utilities for pretty-printed writing.
 * But due to the formatting-varying nature of Datum, will not pretty-print totally automatically.
 * Created 15th February 2023.
 */
public class DatumWriter extends DatumEncodingVisitor {
    protected final Appendable base;
    protected SpacingState queued = SpacingState.None;

    /**
     * Current indentation level. Turns into tabs/etc.
     */
    public int indent = 0;

    public DatumWriter(Appendable base) {
        this.base = base;
    }

    /**
     * Converts an object to string with the minimum required spacing.
     */
    public static String objectToString(Object obj) {
        StringWriter sw = new StringWriter();
        DatumWriter dw = new DatumWriter(sw);
        dw.visitTree(obj, DatumSrcLoc.NONE);
        return sw.toString();
    }

    protected void putChar(char c) {
        try {
            base.append(c);
        } catch (IOException e) {
            throw new DatumRuntimeIOException(e);
        }
    }

    protected void emitQueued(boolean listEnd) {
        if (queued == SpacingState.QueuedIndent) {
            for (int i = 0; i < indent; i++)
                putChar('\t');
        } else if (queued == SpacingState.AfterToken && !listEnd) {
            putChar(' ');
        }
        queued = SpacingState.None;
    }

    private void putUnprintableEscape(char c) {
        if (c == '\r') {
            putChar('\\');
            putChar('r');
        } else if (c == '\n') {
            putChar('\\');
            putChar('n');
        } else if (c == '\t') {
            putChar('\\');
            putChar('t');
        } else {
            putChar('\\');
            putChar('x');
            for (char c2 : Integer.toHexString((int) c).toCharArray())
                putChar(c2);
            putChar(';');
        }
    }

    /**
     * Writes a line comment (can contain newlines, these will be handled), followed by newline.
     */
    public void visitComment(String comment) {
        emitQueued(false);
        putChar(';');
        putChar(' ');
        for (char c : comment.toCharArray()) {
            if (c == '\n') {
                visitNewline();
                emitQueued(false);
                putChar(';');
                putChar(' ');
            } else {
                putChar(c);
            }
        }
        visitNewline();
    }

    /**
     * Writes a newline.
     */
    public void visitNewline() {
        putChar('\n');
        queued = SpacingState.QueuedIndent;
    }

    @Override
    public void visitString(String s, DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('"');
        for (char c : s.toCharArray()) {
            if (c == '"') {
                putChar('\\');
                putChar(c);
            } else if (c < 32 || c == 127) {
                putUnprintableEscape(c);
            } else {
                putChar(c);
            }
        }
        putChar('"');
        queued = SpacingState.AfterToken;
    }

    private void putPIDChar(char c) {
        if (c < 32 || c == 127) {
            putUnprintableEscape(c);
        } else {
            DatumCharClass cc = DatumCharClass.identify(c);
            if (!cc.isValidPID)
                putChar('\\');
            putChar(c);
        }
    }

    @Override
    public void visitId(String s, DatumSrcLoc srcLoc) {
        emitQueued(false);
        if (s.length() == 0) {
            // Emit #{}# to work around this
            putChar('#');
            putChar('{');
            putChar('}');
            putChar('#');
        } else {
            boolean isFirst = true;
            for (char c : s.toCharArray()) {
                if (isFirst) {
                    if (DatumCharClass.identify(c) != DatumCharClass.Content) {
                        if (c < 32 || c == 127) {
                            putUnprintableEscape(c);
                        } else {
                            putChar('\\');
                            putChar(c);
                        }
                    } else {
                        putChar(c);
                    }
                    isFirst = false;
                } else {
                    putPIDChar(c);
                }
            }
        }
        queued = SpacingState.AfterToken;
    }

    @Override
    public void visitNumericUnknown(String s, DatumSrcLoc srcLoc) {
        emitQueued(false);
        if (s.length() == 0) {
            putChar('#');
            putChar('i');
        } else if (s.length() == 1) {
            char c = s.charAt(0);
            if (DatumCharClass.identify(c) != DatumCharClass.Digit) {
                putChar('#');
                putChar('i');
            }
            putChar(c);
        } else {
            if (!DatumCharClass.identify(s.charAt(0)).isNumericStart) {
                putChar('#');
                putChar('i');
            }
            for (char c : s.toCharArray())
                putPIDChar(c);
        }
        queued = SpacingState.AfterToken;
    }

    @Override
    public void visitSpecialUnknown(String s, DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('#');
        for (char c : s.toCharArray()) {
            if (c < 32) {
                putUnprintableEscape(c);
            } else {
                DatumCharClass cc = DatumCharClass.identify(c);
                if (!cc.isValidPID)
                    putChar('\\');
                putChar(c);
            }
        }
        queued = SpacingState.AfterToken;
    }

    @Override
    public void visitBoolean(boolean value, DatumSrcLoc srcLoc) {
        visitSpecialUnknown(value ? "t" : "f", srcLoc);
    }

    @Override
    public void visitNull(DatumSrcLoc srcLoc) {
        visitSpecialUnknown("nil", srcLoc);
    }

    @Override
    public void visitInt(long value, String raw, DatumSrcLoc srcLoc) {
        visitNumericUnknown(raw, srcLoc);
    }

    @Override
    public void visitFloat(double value, String raw, DatumSrcLoc srcLoc) {
        visitNumericUnknown(raw, srcLoc);
    }

    /**
     * Visits a list.
     * For DatumWriter there is an API guarantee that the returned writer will always be the callee.
     */
    @Override
    public DatumWriter visitList(DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('(');
        queued = SpacingState.None;
        return this;
    }

    /**
     * Ends a list.
     */
    @Override
    public void visitEnd(DatumSrcLoc srcLoc) {
        emitQueued(true);
        putChar(')');
        queued = SpacingState.AfterToken;
    }

    protected enum SpacingState {
        None,
        QueuedIndent,
        // After a token (that isn't a list start, for "(example)" kinda thing)
        AfterToken
    }
}
