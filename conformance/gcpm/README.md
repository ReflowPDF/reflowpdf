# gcpm: reftests for paged media

100 reftests for what a print engine does with paged media, written from the specifications in the format of the web-platform-tests (WPT). Results for ReflowPDF, Prince and WeasyPrint are in [RESULTS.md](RESULTS.md).

WPT covers margin boxes, page counters and named pages in `css/css-page`, with 38 reftests in `css/css-page/margin-boxes` alone. For named strings and leaders it has 20 manual tests in `css/css-gcpm` without references, and for running elements, footnotes and `target-counter()` it has none. This suite has reftests for all of these, and for margin boxes, page counters and page selectors, most of them on one page size.

| Folder | Specification | What is tested | Tests |
|---|---|---|---|
| `page/` | [css-page-3](https://drafts.csswg.org/css-page-3/) §4.2, §5, §6, §8.1 | the margin boxes and their default alignment, inheritance from the page context, images in margin boxes, `counter(page)` and `counter(pages)`, `counter-reset: page`, `:first`, `:left`, `:right`, `:blank`, `break-before: recto`, named pages | 21 |
| `strings/` | [css-gcpm-3](https://drafts.csswg.org/css-gcpm/) §1.1 | `string-set` with `content()`, `content(text)`, `content(before)`, `content(after)`, `content(first-letter)`, `counter()`, `attr()` and several values; `string()` with `first`, `start`, `last`, `first-except`; a string that was never set; `string()` in an element's content | 19 |
| `running/` | css-gcpm-3 §1.2 | `position: running()` and `element()`, `first` and `last`, styles, block children, images and backgrounds inside a running element, absolute positioning inside it | 11 |
| `footnotes/` | css-gcpm-3 §2 | `float: footnote`, `::footnote-call`, `::footnote-marker`, the `@footnote` area and its border, the footnote counter, `footnote-display: inline`, `max-height` of the area, a footnote body continued on the next page, pages that hold only footnotes | 14 |
| `leaders/` | [css-content-3](https://drafts.csswg.org/css-content-3/) §5 | `leader(dotted)`, `leader(solid)`, `leader(space)`, `leader(<string>)`, the row of copies laid from the end edge of the line, two leaders on a line, a leader that moves to the next line | 8 |
| `xref/` | css-content-3 §6 | `target-counter()` with `attr(href url)`, `url()` and an author counter, forward and backward; `target-counters()`; `target-text()`; the page of a target placed after the footnote area | 10 |
| `wpt-ports/` | css-gcpm-3 §1.1 | the 17 manual tests of WPT `css/css-gcpm` (`string-set-001` to `012`, `using-strings-001` to `005`) with references added | 17 |

Not covered: PDF bookmarks (`bookmark-level`, `bookmark-label`, which are checked in the PDF outline, not in pixels), page floats, `marks` and `bleed`, page groups and `:nth()`, and the three manual leader tests of WPT, whose references depend on the default font. Whether a leader copy that the content after it only partly covers is hidden whole or glyph by glyph is not tested: css-content-3 §5.3 says both (step 2: "only those boxes not overlaid by the before or after content"; step 3: "that glyph is not displayed"). A test withdrawn before publication is kept in [withdrawn/](withdrawn).

## Tentative tests

Six tests check behaviour the specifications do not define. They are named `.tentative.html`, as in WPT, and [RESULTS.md](RESULTS.md) gives the totals with and without them.

- `footnotes/footnote-split-001`, `footnote-overflow-page-001`, `footnote-only-page-mid-001`: a footnote body that does not fit on its page continues on the next. css-gcpm-3 has no layout algorithm for footnotes (§2.8: "We need an algorithm for laying out footnotes").
- `footnotes/footnote-max-height-001`: what happens to the part of a footnote body that `max-height` cuts off.
- `leaders/leader-two-001`: how two leaders on one line share its free space.
- `page/page-area-empty-001`: an `@page` rule whose margins leave no page area.

## How the tests are written

1. Most tests use the same page: `@page { size: 300px 200px; margin: 40px }`, with `20px/1 Ahem` everywhere, the page context included. The page area is then 11 by 6 cells of 20 px. A few tests need another page size or margin and set it in their own style sheet. The reference draws the same picture without the feature under test.
2. A reference is built only from features outside the test's subject: blocks of fixed height in normal flow, absolute positioning on single-page documents, and literal text. Outside `wpt-ports/`, no reference uses margin boxes, page selectors or named pages.
3. Where a specification leaves the user agent style open, the test sets it, so that it checks the rule and not a default. Every footnote test sets `vertical-align`, `font-size`, `line-height` and `font-variant-position: normal` on `::footnote-call`, `content: none` on `::footnote-marker::after`, and the margins, padding and border of `@footnote`, against the defaults in css-gcpm-3 Appendix B. Tests whose subject is not the footnote marker suppress it (`::footnote-marker { content: none }`); the three that show it (`footnote-call-marker-custom-001`, `footnote-display-inline-001` and `xref/target-counter-page-footnote-001`) set `list-style-position: inside`. `footnote-counter-continues-001` sets `@page { counter-reset: none }`, because Appendix B resets the footnote counter on every page.
4. Ahem draws every letter, digit, `.` and `_` as the same full square. Only a few characters differ, among them the space, `p` (a bar in the descender area) and `É` (the ascender area). A value under test is therefore told apart by its number of glyphs (A, BB, CCC; i, ii, iii in `lower-roman`), by its position, or by `p`. The leader tests of `dotted` and `solid` use [GCPM Leaders](support/leaders.ttf), Ahem with a smaller dot and a thin underscore.
5. Every test links the section it tests (`<link rel="help">`) and states the rule (`<meta name="assert">`).
6. The tests were checked by mutation: a wrong behaviour was put into a copy of a test (`first` for `last`, `:left` for `:right`, a missing `:first` rule, `solid` for `dotted`, a footnote counter that is not reset), or a wrong rendering into a copy of its reference, and the copy must fail. The 121 mutations, written by the authors and by a separate review, are in [mutations.json](mutations.json), and [mutate.py](../mutate.py) runs them on any engine. All 121 fail on ReflowPDF, 120 on Prince 17 and 115 on WeasyPrint. Each that passes repeats what that engine already does: Prince evaluates `counter(page)` in a named string where the string is used, WeasyPrint does not draw `::footnote-marker::after`, and WeasyPrint's other five match its failures listed below.
7. A reference shows the expected margin-box content without margin boxes of its own: as a 40px block at the top or bottom of each page in normal flow, or, on single-page references, as absolutely positioned boxes, so that an engine without margin boxes fails these tests instead of rendering both sides without them. The exception is `wpt-ports/`: their text flows over pages as each engine breaks it, so their references keep a literal margin box, and an engine without margin boxes would pass them; the tests in `page/` show whether an engine has them.

The ports in `wpt-ports/` keep the original author and text. They add `rel=match`, a reference (written by the ReflowPDF authors), a note, and an explicit `@page { size: 8.5in 11in }` in test and reference, because the originals count pages ("this test has three pages") and engines have different default page sizes. They use the default font, in which digits differ.

## References

A reference is confirmed when Prince or WeasyPrint passes the test: 95 of the 100 are. Where one of them fails a test, the reason found by comparing its output with the reference is below.

| Engine | Tests | Why |
|---|---|---|
| Prince 17 | `footnotes/footnote-call-marker-custom-001` | Prince places the footnote marker with a property of its own, `footnote-style-position`, whose default is `outside`, and does not apply `list-style-position: inside`, which the test sets on `::footnote-marker` (css-gcpm-3 §2.7). With `footnote-style-position: inside` on the footnote element, Prince passes it. ReflowPDF and WeasyPrint place the marker inside whatever `list-style-position` says. |
| Prince 17 | `leaders/leader-dotted-001`, `leader-per-line-001`, `leader-wrap-001` | `leader(dotted)` repeats ". " (a dot and a space), as in the 2011 Working Draft of css-gcpm. css-content-3 §5.1: `dotted` is "Equivalent to leader(".")". |
| Prince 17 | `leaders/leader-line-edge-001` | The row of leader copies is laid from the start edge of the line, as in the 2011 Working Draft, where the string is repeated in the writing direction. css-content-3 §5.3 step 2 draws it "from the end edge to the start edge" of the line. |
| Prince 17 | `strings/string-set-content-before-001`, `string-set-content-after-001`, `wpt-ports/wpt-string-set-006`, `wpt-string-set-007` | `content(before)` and `content(after)` give an empty string. |
| Prince 17 | `strings/string-set-page-counter-001` | `counter(page)` in `string-set` takes the page where the string is used (iii on page 3), not where it is set (ii). css-gcpm-3 §1.1.1.1: "The content values of named strings are assigned at the point when the content box of the element is first created." |
| Prince 17 | `strings/string-start-001`, `string-start-first-element-001`, `wpt-ports/wpt-using-strings-003` | `string(name, start)` gives the entry value even when the element that sets the string is the first element on the page. css-gcpm-3 §1.1.2: "If the element is the first element on the page, the value of the first assignment is used." |
| Prince 17 | `xref/target-text-001`, `target-text-before-001` | `target-text()` is not supported; Prince has a function of its own, `target-content()`. |
| WeasyPrint 69.0 | `footnotes/footnote-counter-reset-page-001` | `@page { counter-reset: footnote }` does not reset the footnote counter on each page (css-gcpm-3 §2.5.2). |
| WeasyPrint 69.0 | `page/page-counter-reset-001`, `xref/target-counter-page-counter-reset-001` | With `counter-reset: page 7` on the first page, the pages are numbered from 7, not 8. css-page-3 §6.1: the page counter is "incremented by 1 on every page of the document", and CSS 2.1 §12.4: "If an element both resets and increments a counter, the counter is reset first and then incremented." |
| WeasyPrint 69.0 | `running/running-abspos-001` | An absolutely positioned box inside a running element is placed against the margin box, not the running element. css-position-3 §2: "Values other than static make the box a positioned box, and cause it to establish an absolute positioning containing block." |
| WeasyPrint 69.0 | `running/running-element-background-001` | The running element is only as wide as its text, so its background covers only the text. css-page-3 §5.3.2.2: boxes that are not generated have zero width, so a middle box generated alone receives all the free space. |
| WeasyPrint 69.0 | `strings/string-in-element-content-001` | `string()` in the `content` of an element, not of a margin box, is not supported. |
| WeasyPrint 69.0 | `wpt-ports/wpt-string-set-010`, `wpt-string-set-011` | `string-set` on an element with `display: none` is ignored, so no running head is shown. The property applies to all elements. |

Five references are confirmed by neither engine. Three of them are tentative tests (`footnote-max-height-001`, `leader-two-001`, `page-area-empty-001`). The other two:

- `page/margin-box-zero-size-001`: both engines center content that is taller than a margin box of zero height, so that it overflows above the box. css-page-3 §6: "On page-margin boxes, the vertical-align property behaves as specified for table cells", and css-align-3 §5.1.1 (block containers, including table cells) makes that alignment safe by default: "If no <overflow-position> is specified, and the block container is not a block-axis scroll container, then alignment is safe", so content that does not fit overflows at the end, below the box. Prince also places the top boxes of this test at the top edge of the page instead of 20 px below it, and ends the overflowing line of the zero-width top-right box at its end edge.
- `wpt-ports/wpt-string-set-011`: Prince keeps "Chapter One Title" on page 2, where a second `display: none` heading sets a new value. The reference follows the text of the WPT test.

## Running

```sh
cargo build --release --manifest-path ../compare/Cargo.toml
python3 ../run.py --engine prince .
python3 ../run.py --engine weasyprint footnotes
```

`/fonts/ahem.css` resolves to `conformance/fonts/` from any folder.

## License

The tests are under the MIT license of this repository, except `wpt-ports/`, which are derived from the web-platform-tests under the 3-Clause BSD License ([wpt-ports/LICENSE.md](wpt-ports/LICENSE.md)). `support/leaders.ttf` is derived from Ahem and is in the public domain, as Ahem is.
