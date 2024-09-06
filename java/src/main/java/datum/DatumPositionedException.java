/*
 * gabien-datum - Quick to implement S-expression format
 * Written starting in 2023 by contributors (see CREDITS.txt)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */
package datum;

/**
 * RuntimeException with a DatumSrcLoc.
 * Created 6th September 2024.
 */
@SuppressWarnings("serial")
public class DatumPositionedException extends RuntimeException {
    public final DatumSrcLoc srcLoc;
    public DatumPositionedException(DatumSrcLoc srcLoc, String text) {
        super("@ " + srcLoc + ": " + text);
        this.srcLoc = srcLoc;
    }
    public DatumPositionedException(DatumSrcLoc srcLoc, String text, Throwable ioe) {
        super("@ " + srcLoc + ": " + text, ioe);
        this.srcLoc = srcLoc;
    }
    public DatumPositionedException(DatumSrcLoc srcLoc, Throwable ioe) {
        super("@ " + srcLoc.toString(), ioe);
        this.srcLoc = srcLoc;
    }
}
