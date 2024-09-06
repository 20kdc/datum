/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

import java.util.Stack;

/**
 * Stream of read-in Datum tokens.
 * Also contains the logic that turns this into a parsed stream, so this is the parser too.
 * Created February 16th, 2023.
 */
public abstract class DatumTokenSource {
    /**
     * Reads a token.
     * Returns true if a token was read successfully (see contents, type)
     */
    public abstract boolean read();

    /**
     * Contents of last read token, if any.
     * Meaning is dependent on type but is pretty obvious for cases where it isn't null.
     */
    public abstract String contents();

    /**
     * A human-readable representation of the position within the token stream.
     */
    public abstract String position();

    /**
     * A computer-readable representation of the position within the token stream (for less severe errors).
     * Maybe this should be refactored at some later point.
     */
    public abstract DatumSrcLoc srcLoc();

    /**
     * Type of last read token.
     */
    public abstract DatumTokenType type();

    /**
     * Parses a value from the token stream into a visitor.
     */
    public final boolean visitValue(DatumVisitor visitor) {
        Stack<DatumVisitor> storedListVisitors = new Stack<>();
        Stack<String> storedListStarts = new Stack<>();
        String listStart = null;
        while (true) {
            // First token of the value.
            if (!read()) {
                if (storedListVisitors.isEmpty())
                    return false;
                throw new DatumPositionedException(srcLoc(), "EOF during list started here");
            }
            switch (type()) {
            case ID:
                visitor.visitId(contents(), srcLoc());
                break;
            case SpecialID:
            {
                String c = contents();
                if (c.equalsIgnoreCase("t")) {
                    visitor.visitBoolean(true, srcLoc());
                } else if (c.equalsIgnoreCase("f")) {
                    visitor.visitBoolean(false, srcLoc());
                } else if (c.equals("{}#")) {
                    visitor.visitId("", srcLoc());
                } else if (c.equalsIgnoreCase("nil")) {
                    visitor.visitNull(srcLoc());
                } else if (c.startsWith("x") || c.startsWith("X")) {
                    long l;
                    try {
                        l = Long.valueOf(c.substring(1), 16);
                    } catch (NumberFormatException nfe2) {
                        throw new DatumPositionedException(srcLoc(), "Invalid hex constant: " + c, nfe2);
                    }
                    visitor.visitInt(l, srcLoc());
                } else if (c.equalsIgnoreCase("i+inf.0")) {
                    visitor.visitFloat(Double.POSITIVE_INFINITY, srcLoc());
                } else if (c.equalsIgnoreCase("i-inf.0")) {
                    visitor.visitFloat(Double.NEGATIVE_INFINITY, srcLoc());
                } else if (c.equalsIgnoreCase("i+nan.0")) {
                    visitor.visitFloat(Double.NaN, srcLoc());
                } else {
                    throw new DatumPositionedException(srcLoc(), "Unknown special ID sequence: " + c);
                }
            }
                break;
            case String:
                visitor.visitString(contents(), srcLoc());
                break;
            case ListStart:
                storedListVisitors.push(visitor);
                storedListStarts.push(listStart);
                visitor = visitor.visitList(srcLoc());
                listStart = position();
                break;
            case ListEnd:
                if (storedListVisitors.isEmpty())
                    throw new DatumPositionedException(srcLoc(), "List end with no list");
                visitor.visitEnd(srcLoc());
                visitor = storedListVisitors.pop();
                listStart = storedListStarts.pop();
                break;
            case Numeric:
                visitNumeric(visitor, contents());
                break;
            }
            // Read in a value (or started or ended a list), so if the stack is empty we are done here
            if (storedListVisitors.isEmpty())
                return true;
        }
    }

    private final void visitNumeric(DatumVisitor visitor, String c) {
        // Conversion...
        long l = 0L;
        try {
            l = Long.valueOf(c);
        } catch (NumberFormatException nfe1) {
            double d = 0d;
            try {
                d = Double.valueOf(c);
            } catch (NumberFormatException nfe2) {
                throw new DatumPositionedException(srcLoc(), "Invalid number: " + c);
            }
            visitor.visitFloat(d, srcLoc());
            return;
        }
        visitor.visitInt(l, srcLoc());
    }

    /**
     * Parses the whole token stream into a visitor.
     */
    public final void visit(DatumVisitor visitor) {
        while (visitValue(visitor));
    }
}
