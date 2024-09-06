/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

import java.util.LinkedList;

/**
 * Turns a visitor on its head so that it outputs objects.
 * Created 15th February 2023.
 */
public abstract class DatumTreeVisitor extends DatumVisitor {
    public DatumTreeVisitor() {
        
    }

    @Override
    public abstract void visitTree(Object obj, DatumSrcLoc srcLoc);

    @Override
    public final void visitString(String s, DatumSrcLoc srcLoc) {
        visitTree(s, srcLoc);
    }

    @Override
    public final void visitId(String s, DatumSrcLoc srcLoc) {
        visitTree(new DatumSymbol(s), srcLoc);
    }

    @Override
    public final void visitBoolean(boolean value, DatumSrcLoc srcLoc) {
        visitTree(value, srcLoc);
    }

    @Override
    public final void visitNull(DatumSrcLoc srcLoc) {
        visitTree(null, srcLoc);
    }

    @Override
    public final void visitInt(long value, DatumSrcLoc srcLoc) {
        visitTree(value, srcLoc);
    }

    @Override
    public final void visitFloat(double value, DatumSrcLoc srcLoc) {
        visitTree(value, srcLoc);
    }

    @Override
    public final DatumVisitor visitList(DatumSrcLoc srcLoc) {
        final LinkedList<Object> buildingList = new LinkedList<>();
        final DatumTreeVisitor me = this;
        return new DatumTreeVisitor() {
            @Override
            public void visitTree(Object obj, DatumSrcLoc srcLoc) {
                buildingList.add(obj);
            }

            @Override
            public void visitEnd(DatumSrcLoc srcLoc) {
                me.visitTree(buildingList, srcLoc);
            }
        };
    }
}
