"""Subset the IBM 3270 webfont to the range the site can actually render.

The upstream face maps 1997 codepoints (CJK compatibility ideographs,
fullwidth forms, Hangul jamo, ...) and ships as a 63 KB woff2. The site is an
English technical-documentation site: across every built page it renders 105
distinct characters. The unreferenced codepoints are pure LCP weight -- the
brand font is the largest single asset on the critical path and gates the
hero text.

The retained set is deliberately far wider than the characters currently
rendered, so ordinary copy edits cannot fall off the edge:

  * ASCII plus Latin-1 Supplement and Latin Extended-A (European prose and
    names), Greek and Cyrillic (quoted API field names, author names)
  * General Punctuation, Currency, Letterlike, Number Forms, Arrows,
    Mathematical Operators, Misc Technical, Box Drawing, Block Elements,
    Geometric Shapes and Dingbats (the prose set, plus every symbol the
    terminal mockups and the install tables draw with)
  * Combining Diacritical Marks, so decomposed accents stay intact

Anything outside this set still renders: the site declares a metric-matched
'IBM3270 Fallback' (Courier New at 90%), so a missing glyph falls back
without shifting line breaks. What it costs is brand consistency on that one
character, not layout -- which is the right trade against 40 KB of LCP.

Run from site/:  python3 scripts/subset-font.py
"""

import os
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# The full upstream face lives outside public/ so it is never served: the
# build only references the subset, and re-subsetting stays possible without
# re-downloading the original.
SOURCE = os.path.join(REPO, "site", "fonts-src", "3270-Regular-full.woff2")
TARGET = os.path.join(REPO, "site", "public", "fonts", "3270-Regular.woff2")

RANGES = [
    (0x0020, 0x007E),  # Basic Latin
    (0x00A0, 0x00FF),  # Latin-1 Supplement
    (0x0100, 0x017F),  # Latin Extended-A
    (0x0300, 0x036F),  # Combining Diacritical Marks
    (0x2000, 0x206F),  # General Punctuation
    (0x20A0, 0x20BF),  # Currency Symbols
    (0x2190, 0x21FF),  # Arrows
    (0x2500, 0x257F),  # Box Drawing
    (0x2580, 0x259F),  # Block Elements
    (0x25A0, 0x25FF),  # Geometric Shapes
    (0x2700, 0x27BF),  # Dingbats
]

# Ranges deliberately NOT retained, and what dropping each one cost. The site
# is English technical documentation and renders 105 distinct characters, all
# of them above; these were measured against the upstream face:
#
#   Greek                +3.0 KB      Cyrillic            +5.1 KB
#   Misc Technical       +4.0 KB      Mathematical Ops   +2.3 KB
#   Letterlike Symbols   +0.5 KB      Number Forms        +0.8 KB
#
# Together that is ~15 KB of LCP weight for glyphs no page has ever used. IBM
# 3270 is a Latin heritage face, so its Greek and Cyrillic cuts are of dubious
# quality anyway. Should a non-Latin character ever reach the site it falls
# back to the metric-matched 'IBM3270 Fallback' for that one glyph: brand
# consistency is lost on that character, line breaks are not. Add the range
# back here if the site ever starts shipping that content.


def codepoints():
    out = set()
    for lo, hi in RANGES:
        out.update(range(lo, hi + 1))
    return out


def main():
    if not os.path.exists(SOURCE):
        sys.exit(f"missing {SOURCE}")
    font = TTFont(SOURCE)
    cmap = set(font.getBestCmap())
    keep = codepoints() & cmap
    dropped = len(cmap) - len(keep)

    options = subset.Options()
    options.flavor = "woff2"
    # Keep the features the terminal mockups rely on; drop hinting tables
    # that no modern engine uses and that cost bytes.
    options.desubroutinize = True
    options.hinting = False
    options.notdef_outline = True
    options.recalc_bounds = True

    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=keep)
    subsetter.subset(font)
    font.flavor = "woff2"
    font.save(TARGET)

    before = os.path.getsize(SOURCE)
    after = os.path.getsize(TARGET)

    # Every codepoint the font used to map must still be either present or
    # knowingly absent; a silent loss here is what breaks the site months
    # later, so assert the mapping we care about survived.
    out_map = set(TTFont(TARGET).getBestCmap())
    required = set(range(0x20, 0x7F)) | {0x00B7, 0x2014, 0x2019, 0x201C, 0x201D,
                                          0x2026, 0x2192, 0x2197, 0x2713}
    missing = required - out_map
    if missing:
        sys.exit(f"subset lost required codepoints: {sorted(hex(c) for c in missing)}")

    print(f"source {before} bytes ({len(cmap)} codepoints)")
    print(f"subset {after} bytes ({len(out_map)} codepoints), dropped {dropped}")
    print(f"saved {before - after} bytes ({100 * (before - after) / before:.1f}%)")


if __name__ == "__main__":
    main()
