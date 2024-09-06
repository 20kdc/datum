/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

import java.util.List;

/**
 * Created 17th February 2023
 */
public final class DatumTreeUtils {
    private DatumTreeUtils() {
        
    }

    public static DatumVisitor decVisitor(VisitorLambda h) {
        return new LambdaVisitor(h);
    }

    /**
     * Simply passes to a lambda.
     * Mainly useful for syntactic simplicity.
     * Created 13th March 2023.
     * Moved into DatumTreeUtils 6th September 2024.
     */
    private final static class LambdaVisitor extends DatumTreeVisitor {
        public final VisitorLambda handler;

        public LambdaVisitor(VisitorLambda h) {
            this.handler = h;
        }

        @Override
        public void visitTree(Object obj, DatumSrcLoc srcLoc) {
            handler.handle(obj, srcLoc);
        }

        @Override
        public void visitEnd(DatumSrcLoc loc) {
        }
    }

    public static DatumSymbol sym(String s) {
        return new DatumSymbol(s);
    }

    public static boolean isSym(Object o, String s) {
        if (o instanceof DatumSymbol)
            return ((DatumSymbol) o).id.equals(s);
        return false;
    }

    public static int cInt(Object o) {
        return ((Number) o).intValue();
    }

    public static long cLong(Object o) {
        return ((Number) o).longValue();
    }

    public static double cDouble(Object o) {
        return ((Number) o).doubleValue();
    }

    public static float cFloat(Object o) {
        return ((Number) o).floatValue();
    }

    /**
     * Best not to confuse this with Arrays.asList.
     */
    @SuppressWarnings("unchecked")
    public static List<Object> cList(Object o) {
        return (List<Object>) o;
    }

    public interface VisitorLambda {
        void handle(Object value, DatumSrcLoc srcLoc);
    }
}
