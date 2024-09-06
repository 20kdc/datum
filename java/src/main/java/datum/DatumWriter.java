/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

import java.io.IOException;
import java.io.StringWriter;

/**
 * Writes out a Datum (or a stream of them) to a Writer.
 * Includes utilities for pretty-printed writing.
 * But due to the formatting-varying nature of Datum, will not pretty-print totally automatically.
 * Created 15th February 2023.
 */
public final class DatumWriter extends DatumStreamingVisitor {
    public final Appendable base;
    public State queued = State.None;

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
        if (queued == State.QueuedIndent) {
            for (int i = 0; i < indent; i++)
                putChar('\t');
        } else if (queued == State.AfterToken && !listEnd) {
            putChar(' ');
        }
        queued = State.None;
    }

    private void putEscape(char c) {
        if (c == '\r') {
            putChar('\\');
            putChar('r');
        } else if (c == '\n') {
            putChar('\\');
            putChar('n');
        } else if (c == '\t') {
            putChar('\\');
            putChar('t');
        } else if (c < 32 || c == 127 || c == 'r' || c == 'n' || c == 't' || c == 'x') {
            putChar('\\');
            putChar('x');
            String hex = Integer.toHexString((int) c);
            // roundtrip tests assume this
            if (hex.length() == 1)
                putChar('0');
            for (char c2 : hex.toCharArray())
                putChar(c2);
            putChar(';');
        } else {
            putChar('\\');
            putChar(c);
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
        queued = State.QueuedIndent;
    }

    @Override
    public void visitString(String s, DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('"');
        for (char c : s.toCharArray()) {
            if (c < 32 || c == 127 || c == '"') {
                putEscape(c);
            } else {
                putChar(c);
            }
        }
        putChar('"');
        queued = State.AfterToken;
    }

    private void putPIDChar(char c) {
        DatumCharClass cc = DatumCharClass.identify(c);
        if (!cc.isValidPID) {
            putEscape(c);
        } else {
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
        } else if (s.length() == 1) {
            char chr = s.charAt(0);
            DatumCharClass cls = DatumCharClass.identify(chr);
            if (cls != DatumCharClass.Content && cls != DatumCharClass.Sign) {
                putEscape(chr);
            } else {
                putChar(chr);
            }
        } else {
            boolean isFirst = true;
            for (char c : s.toCharArray()) {
                if (isFirst) {
                    if (DatumCharClass.identify(c) != DatumCharClass.Content) {
                        putEscape(c);
                    } else {
                        putChar(c);
                    }
                    isFirst = false;
                } else {
                    putPIDChar(c);
                }
            }
        }
        queued = State.AfterToken;
    }

    @Override
    public void visitBoolean(boolean value, DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('#');
        putChar(value ? 't' : 'f');
        queued = State.AfterToken;
    }

    @Override
    public void visitNull(DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('#');
        putChar('n');
        putChar('i');
        putChar('l');
        queued = State.AfterToken;
    }

    @Override
    public void visitInt(long value, DatumSrcLoc srcLoc) {
        emitQueued(false);
        for (char c : Long.toString(value).toCharArray())
            putChar(c);
        queued = State.AfterToken;
    }

    @Override
    public void visitFloat(double value, DatumSrcLoc srcLoc) {
        emitQueued(false);
        if (!Double.isFinite(value)) {
            putChar('#');
            putChar('i');
            if (Double.isInfinite(value)) {
                putChar(value > 0 ? '+' : '-');
                putChar('i');
                putChar('n');
                putChar('f');
                putChar('.');
                putChar('0');
            } else {
                putChar('+');
                putChar('n');
                putChar('a');
                putChar('n');
                putChar('.');
                putChar('0');
            }
        } else {
            for (char c : Double.toString(value).toCharArray())
                putChar(c);
        }
        queued = State.AfterToken;
    }

    /**
     * Visits a list.
     * For DatumWriter there is an API guarantee that the returned writer will always be the callee.
     */
    @Override
    public DatumWriter visitList(DatumSrcLoc srcLoc) {
        emitQueued(false);
        putChar('(');
        queued = State.None;
        return this;
    }

    /**
     * Ends a list.
     */
    @Override
    public void visitEnd(DatumSrcLoc srcLoc) {
        emitQueued(true);
        putChar(')');
        queued = State.AfterToken;
    }

    public enum State {
        None,
        QueuedIndent,
        // After a token (that isn't a list start, for "(example)" kinda thing)
        AfterToken
    }
}
