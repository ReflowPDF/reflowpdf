# Withdrawn tests

Tests taken out of the suite, kept as they were so that the change can be checked. `run.py` does not run this folder.

- `leader-end-aligned-001` (withdrawn on 11 November 2026, before the first publication). It expected the row of leader copies to start at the end of the leader itself, before the content after it. css-content-3 §5.3 step 2 draws the row "from the end edge to the start edge" of the line, and its example shows the dots under the whole line before the content is drawn over them. ReflowPDF failed this test and Prince passed it. It is replaced by `leaders/leader-line-edge-001`, which ReflowPDF and WeasyPrint pass and Prince fails. What the old test also depended on, whether a leader copy that the content after it only partly covers is hidden whole (§5.3 step 2) or glyph by glyph (step 3), is not tested.

The files are kept exactly as they were, including an `assert` attribute that the unescaped quotes in it cut short and a section number (§2.5.3) from the 2019 Working Draft of css-content-3.
