# gcpm results

| Engine | Without the 6 tentative tests | All 100 tests |
|---|---|---|
| ReflowPDF | 94 of 94 | 100 |
| Prince 17 | 77 of 94 | 80 |
| WeasyPrint 69.0 | 85 of 94 | 85 |

The tentative tests check behaviour the specifications do not define ([README.md](README.md#tentative-tests)), and their references follow ReflowPDF's choices, so the first column is the comparison. 95 of the 100 references are confirmed by Prince or WeasyPrint. Why each engine fails a test is in [README.md](README.md#references): outside the tentative tests, Prince's 17 failures come from 9 causes and WeasyPrint's 9 from 7.

The suite was written by the makers of ReflowPDF, alongside the engine. That ReflowPDF passes all of it shows that the engine does what these tests check; it does not show that the tests check everything.

All three engines were run on 11 November 2026 with [run.py](../run.py) and judged by [compare](../compare). ReflowPDF is the engine build in production that day (b815fc4), run through the engine's command-line renderer with `run.py --inline`, so that it read nothing but the document, as through the HTTP API. Per-test results are in [results/](results).

Prince was run without a license, so it prints its logo near the top right of the first page (on the 300 × 200 px page, about x 246 to 280 px and y 20 to 53 px). The logo is the same in test and reference, so a difference under it is not seen. In these tests that area holds the `@top-right` margin box and the end of the first line.

## Changes before the first publication

- `leaders/leader-end-aligned-001` was replaced by `leaders/leader-line-edge-001`; the old test is kept in [withdrawn/](withdrawn) with the reason. With the old test, ReflowPDF failed and Prince passed; with the new one, ReflowPDF and WeasyPrint pass and Prince fails.
- Reviews found that Ahem draws every letter and digit as the same square, so that about twenty tests could not tell a wrong value from the right one, and that the references drew their expected headers with margin boxes, so that an engine without margin boxes passed them. Values are now encoded in the number of glyphs, by position or with the few glyphs Ahem draws differently; the tests of `leader(dotted)` and `leader(solid)` use a font with a visible dot and underscore; references draw headers and footers in normal flow; footnote tests whose subject is not the marker suppress it; and the tests were checked by 121 mutations (see [README.md](README.md#how-the-tests-are-written)). Six tests that only repeated a `-roman` test in decimal digits were removed, and six tests were marked tentative.

| Test | ReflowPDF | Prince 17 | WeasyPrint 69.0 |
|---|---|---|---|
| [footnotes/footnote-area-border-001.html](footnotes/footnote-area-border-001.html) | pass | pass | pass |
| [footnotes/footnote-area-bottom-001.html](footnotes/footnote-area-bottom-001.html) | pass | pass | pass |
| [footnotes/footnote-basic-001.html](footnotes/footnote-basic-001.html) | pass | pass | pass |
| [footnotes/footnote-block-content-001.html](footnotes/footnote-block-content-001.html) | pass | pass | pass |
| [footnotes/footnote-call-marker-custom-001.html](footnotes/footnote-call-marker-custom-001.html) | pass | **fail** | pass |
| [footnotes/footnote-counter-continues-001.html](footnotes/footnote-counter-continues-001.html) | pass | pass | pass |
| [footnotes/footnote-counter-reset-page-001.html](footnotes/footnote-counter-reset-page-001.html) | pass | pass | **fail** |
| [footnotes/footnote-display-inline-001.html](footnotes/footnote-display-inline-001.html) | pass | pass | pass |
| [footnotes/footnote-max-height-001.tentative.html](footnotes/footnote-max-height-001.tentative.html) | pass | **fail** | **fail** |
| [footnotes/footnote-multiple-001.html](footnotes/footnote-multiple-001.html) | pass | pass | pass |
| [footnotes/footnote-only-page-mid-001.tentative.html](footnotes/footnote-only-page-mid-001.tentative.html) | pass | pass | **fail** |
| [footnotes/footnote-overflow-page-001.tentative.html](footnotes/footnote-overflow-page-001.tentative.html) | pass | pass | **fail** |
| [footnotes/footnote-split-001.tentative.html](footnotes/footnote-split-001.tentative.html) | pass | pass | **fail** |
| [footnotes/footnote-styled-body-001.html](footnotes/footnote-styled-body-001.html) | pass | pass | pass |
| [leaders/leader-dotted-001.html](leaders/leader-dotted-001.html) | pass | **fail** | pass |
| [leaders/leader-line-edge-001.html](leaders/leader-line-edge-001.html) | pass | **fail** | pass |
| [leaders/leader-per-line-001.html](leaders/leader-per-line-001.html) | pass | **fail** | pass |
| [leaders/leader-solid-001.html](leaders/leader-solid-001.html) | pass | pass | pass |
| [leaders/leader-space-001.html](leaders/leader-space-001.html) | pass | pass | pass |
| [leaders/leader-string-001.html](leaders/leader-string-001.html) | pass | pass | pass |
| [leaders/leader-two-001.tentative.html](leaders/leader-two-001.tentative.html) | pass | **fail** | **fail** |
| [leaders/leader-wrap-001.html](leaders/leader-wrap-001.html) | pass | **fail** | pass |
| [page/margin-box-bottom-001.html](page/margin-box-bottom-001.html) | pass | pass | pass |
| [page/margin-box-corners-001.html](page/margin-box-corners-001.html) | pass | pass | pass |
| [page/margin-box-image-001.html](page/margin-box-image-001.html) | pass | pass | pass |
| [page/margin-box-inherit-page-context-001.html](page/margin-box-inherit-page-context-001.html) | pass | pass | pass |
| [page/margin-box-inline-style-001.html](page/margin-box-inline-style-001.html) | pass | pass | pass |
| [page/margin-box-sides-001.html](page/margin-box-sides-001.html) | pass | pass | pass |
| [page/margin-box-text-align-default-001.html](page/margin-box-text-align-default-001.html) | pass | pass | pass |
| [page/margin-box-top-001.html](page/margin-box-top-001.html) | pass | pass | pass |
| [page/margin-box-vertical-align-default-001.html](page/margin-box-vertical-align-default-001.html) | pass | pass | pass |
| [page/margin-box-zero-size-001.html](page/margin-box-zero-size-001.html) | pass | **fail** | **fail** |
| [page/named-page-001.html](page/named-page-001.html) | pass | pass | pass |
| [page/named-page-break-001.html](page/named-page-break-001.html) | pass | pass | pass |
| [page/named-page-inherit-001.html](page/named-page-inherit-001.html) | pass | pass | pass |
| [page/page-area-empty-001.tentative.html](page/page-area-empty-001.tentative.html) | pass | **fail** | **fail** |
| [page/page-counter-001.html](page/page-counter-001.html) | pass | pass | pass |
| [page/page-counter-reset-001.html](page/page-counter-reset-001.html) | pass | pass | **fail** |
| [page/page-selector-blank-001.html](page/page-selector-blank-001.html) | pass | pass | pass |
| [page/page-selector-blank-recto-001.html](page/page-selector-blank-recto-001.html) | pass | pass | pass |
| [page/page-selector-first-001.html](page/page-selector-first-001.html) | pass | pass | pass |
| [page/page-selector-left-right-001.html](page/page-selector-left-right-001.html) | pass | pass | pass |
| [page/pages-total-001.html](page/pages-total-001.html) | pass | pass | pass |
| [running/running-abspos-001.html](running/running-abspos-001.html) | pass | pass | **fail** |
| [running/running-element-001.html](running/running-element-001.html) | pass | pass | pass |
| [running/running-element-background-001.html](running/running-element-background-001.html) | pass | pass | **fail** |
| [running/running-element-block-children-001.html](running/running-element-block-children-001.html) | pass | pass | pass |
| [running/running-element-first-001.html](running/running-element-first-001.html) | pass | pass | pass |
| [running/running-element-image-001.html](running/running-element-image-001.html) | pass | pass | pass |
| [running/running-element-inline-style-001.html](running/running-element-inline-style-001.html) | pass | pass | pass |
| [running/running-element-last-001.html](running/running-element-last-001.html) | pass | pass | pass |
| [running/running-element-later-page-001.html](running/running-element-later-page-001.html) | pass | pass | pass |
| [running/running-element-replaces-page-content-001.html](running/running-element-replaces-page-content-001.html) | pass | pass | pass |
| [running/running-element-styled-001.html](running/running-element-styled-001.html) | pass | pass | pass |
| [strings/string-first-001.html](strings/string-first-001.html) | pass | pass | pass |
| [strings/string-first-except-001.html](strings/string-first-except-001.html) | pass | pass | pass |
| [strings/string-in-element-content-001.html](strings/string-in-element-content-001.html) | pass | pass | **fail** |
| [strings/string-last-001.html](strings/string-last-001.html) | pass | pass | pass |
| [strings/string-set-attr-001.html](strings/string-set-attr-001.html) | pass | pass | pass |
| [strings/string-set-basic-001.html](strings/string-set-basic-001.html) | pass | pass | pass |
| [strings/string-set-concat-001.html](strings/string-set-concat-001.html) | pass | pass | pass |
| [strings/string-set-content-after-001.html](strings/string-set-content-after-001.html) | pass | **fail** | pass |
| [strings/string-set-content-before-001.html](strings/string-set-content-before-001.html) | pass | **fail** | pass |
| [strings/string-set-content-first-letter-001.html](strings/string-set-content-first-letter-001.html) | pass | pass | pass |
| [strings/string-set-content-text-001.html](strings/string-set-content-text-001.html) | pass | pass | pass |
| [strings/string-set-counter-001.html](strings/string-set-counter-001.html) | pass | pass | pass |
| [strings/string-set-multiple-001.html](strings/string-set-multiple-001.html) | pass | pass | pass |
| [strings/string-set-nested-text-001.html](strings/string-set-nested-text-001.html) | pass | pass | pass |
| [strings/string-set-page-counter-001.html](strings/string-set-page-counter-001.html) | pass | **fail** | pass |
| [strings/string-set-per-page-001.html](strings/string-set-per-page-001.html) | pass | pass | pass |
| [strings/string-start-001.html](strings/string-start-001.html) | pass | **fail** | pass |
| [strings/string-start-first-element-001.html](strings/string-start-first-element-001.html) | pass | **fail** | pass |
| [strings/string-unset-001.html](strings/string-unset-001.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-001.html](wpt-ports/wpt-string-set-001.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-002.html](wpt-ports/wpt-string-set-002.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-003.html](wpt-ports/wpt-string-set-003.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-004.html](wpt-ports/wpt-string-set-004.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-005.html](wpt-ports/wpt-string-set-005.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-006.html](wpt-ports/wpt-string-set-006.html) | pass | **fail** | pass |
| [wpt-ports/wpt-string-set-007.html](wpt-ports/wpt-string-set-007.html) | pass | **fail** | pass |
| [wpt-ports/wpt-string-set-008.html](wpt-ports/wpt-string-set-008.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-009.html](wpt-ports/wpt-string-set-009.html) | pass | pass | pass |
| [wpt-ports/wpt-string-set-010.html](wpt-ports/wpt-string-set-010.html) | pass | pass | **fail** |
| [wpt-ports/wpt-string-set-011.html](wpt-ports/wpt-string-set-011.html) | pass | **fail** | **fail** |
| [wpt-ports/wpt-string-set-012.html](wpt-ports/wpt-string-set-012.html) | pass | pass | pass |
| [wpt-ports/wpt-using-strings-001.html](wpt-ports/wpt-using-strings-001.html) | pass | pass | pass |
| [wpt-ports/wpt-using-strings-002.html](wpt-ports/wpt-using-strings-002.html) | pass | pass | pass |
| [wpt-ports/wpt-using-strings-003.html](wpt-ports/wpt-using-strings-003.html) | pass | **fail** | pass |
| [wpt-ports/wpt-using-strings-004.html](wpt-ports/wpt-using-strings-004.html) | pass | pass | pass |
| [wpt-ports/wpt-using-strings-005.html](wpt-ports/wpt-using-strings-005.html) | pass | pass | pass |
| [xref/target-counter-custom-001-roman.html](xref/target-counter-custom-001-roman.html) | pass | pass | pass |
| [xref/target-counter-page-001-roman.html](xref/target-counter-page-001-roman.html) | pass | pass | pass |
| [xref/target-counter-page-002-roman.html](xref/target-counter-page-002-roman.html) | pass | pass | pass |
| [xref/target-counter-page-counter-reset-001.html](xref/target-counter-page-counter-reset-001.html) | pass | pass | **fail** |
| [xref/target-counter-page-footnote-001.html](xref/target-counter-page-footnote-001.html) | pass | pass | pass |
| [xref/target-counter-page-forward-back-001-roman.html](xref/target-counter-page-forward-back-001-roman.html) | pass | pass | pass |
| [xref/target-counter-url-001-roman.html](xref/target-counter-url-001-roman.html) | pass | pass | pass |
| [xref/target-counters-001-roman.html](xref/target-counters-001-roman.html) | pass | pass | pass |
| [xref/target-text-001.html](xref/target-text-001.html) | pass | **fail** | pass |
| [xref/target-text-before-001.html](xref/target-text-before-001.html) | pass | **fail** | pass |
