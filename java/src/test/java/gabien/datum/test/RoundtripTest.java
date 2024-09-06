/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package gabien.datum.test;

import java.io.FileInputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.io.StringReader;
import java.nio.charset.StandardCharsets;

import org.junit.Test;
import org.junit.Assert;

import datum.DatumReaderTokenSource;
import datum.DatumSrcLoc;
import datum.DatumTreeVisitor;
import datum.DatumWriter;

/**
 * Created 6th September 2024.
 */
public class RoundtripTest {
    @Test
    public void test() throws Exception {
        String rin = readFileToString("../doc/roundtrip-input.scm");
        String ron = readFileToString("../doc/roundtrip-output.scm");
        StringBuilder output = new StringBuilder();
        DatumReaderTokenSource drts = new DatumReaderTokenSource("roundtrip-input", new StringReader(rin));
        DatumWriter dw = new DatumWriter(output);
        drts.visit(new DatumTreeVisitor() {
            @Override
            public void visitTree(Object obj, DatumSrcLoc srcLoc) {
                dw.visitTree(obj, srcLoc);
                dw.visitNewline();
            }
            @Override
            public void visitEnd(DatumSrcLoc loc) {
            }
        });
        System.out.println("Roundtrip test: Expected:");
        System.out.println(ron);
        System.out.println("Roundtrip test: Got:");
        System.out.println(output);
        Assert.assertEquals(ron, output.toString());
    }
    private String readFileToString(String fn) throws Exception {
        try (Reader r = new InputStreamReader(new FileInputStream(fn), StandardCharsets.UTF_8)) {
            StringBuilder sb = new StringBuilder();
            while (true) {
                int v = r.read();
                if (v == -1)
                    break;
                sb.append((char) v);
            }
            return sb.toString();
        }
    }
}
