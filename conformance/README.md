# Conformance

The code that judges the reftests of the ReflowPDF engine, a runner that applies it to any HTML-to-PDF engine, and a suite of 100 paged-media reftests with results for ReflowPDF, Prince and WeasyPrint.

The engine's results on the web-platform-tests (WPT) are published at [reflowpdf.com/conformance](https://reflowpdf.com/conformance), per specification and per test, with the [method](https://reflowpdf.com/conformance/methodology), the [tests left out](https://reflowpdf.com/conformance/not-judged) and the data as JSON. Those figures come from the engine's own test harness, which is closed source. This folder holds its comparator and a runner that judges reftests by the same rules, with the differences listed below.

| Path | What it is |
|---|---|
| [`compare/`](compare) | The pixel comparator, in Rust. The engine's harness judges every reftest with this code. |
| [`gcpm/`](gcpm) | 100 reftests for margin boxes, page counters, named strings, running elements, footnotes, leaders and cross-references. [Results](gcpm/RESULTS.md) for ReflowPDF, Prince and WeasyPrint. |
| [`run.py`](run.py) | Prints reftests with Prince, WeasyPrint, the ReflowPDF API or any command line, and judges them with `compare/`. |
| [`mutate.py`](mutate.py) | Puts the mistakes listed in `gcpm/mutations.json` into copies of the tests and checks, with `run.py`, that every copy fails. |
| [`fonts/`](fonts) | Ahem, the test font of WPT. |

## How a reftest is judged

A reftest is an HTML file and a reference that must look the same (`rel=match`) or different (`rel=mismatch`).

1. The test and the reference are printed to PDF by the same engine.
2. Both PDFs are rasterized by `pdftoppm` at 96 dpi.
3. The pages are compared pixel by pixel. A difference of 1/255 in a channel is the rasterizer's noise and is allowed. A larger difference is forgiven only as anti-aliasing at an edge: in both images the pixel's 3×3 neighbourhood crosses an edge, each side's colour lies within the other side's range, and at least one side is a partial blend rather than a solid colour. Most pixels of a shifted edge show a solid colour where the other image has a blend, so a shifted edge fails. If more than 1% of all pixels would be forgiven, none are.
4. A page passes when its largest remaining difference is at most 1/255, or when that difference and the page's number of differing pixels are both within the upper bounds of the tolerance the test declares with `<meta name="fuzzy">`. The test passes when every page does. A `rel=mismatch` test fails when the comparator finds no difference beyond forgiven anti-aliasing; a difference of 1/255 counts as a difference there and the declared tolerance does not apply, as in the engine's harness.

A test that declares `@page` is printed on its own pages, and all its pages are compared as one set: the 1% limit counts the pixels of all of them, and a different number of pages fails. Any other test is printed on one 600pt × 450pt page without margins (800 × 600 CSS px) and only its first page is compared. Tests that need a script, interaction or the network are not run, and neither are tests whose reference needs them.

The comparator reads `<meta name="fuzzy">` the way WPT does, from the test file only. A range is `lo-hi` or a single value `n` (the range `n-n`); ranges named `maxDifference=` and `totalPixels=` may come in either order, and unnamed ones take that order. A `<url>:` prefix keys a tolerance to one reference, by a URL relative to the test or, with a leading `/`, relative to `--root`; for any other reference the test's unprefixed tolerance applies, if it has one. One rule differs from WPT: lower bounds are read but do not fail a page. In WPT a lower bound records the difference that a browser's own rasterizer leaves, and a smaller difference fails. Here the anti-aliasing allowance removes that fringe, and pdftoppm draws a different one: `css/css-transforms/ttwf-transform-skewx-001.html` declares `maxDifference=63-64`, and an exact skew leaves 51 after the allowance.

The comparator's output gives the counts behind the verdict, for example `hard=40 maxd=1 excused=12 delta1=40 min_blend_depth=31 near_pure=0 verdict=PASS`: `hard` pixels differ after the allowance (those at 1/255 included), `excused` were forgiven as anti-aliasing. With `--map` it draws where the pages differ: magenta for differences that count, amber for forgiven anti-aliasing.

## What the engine's harness does beyond run.py

The published WPT figures cover more than reftests, and judge reftests with a few rules `run.py` does not have:

- Layout, parsing and computed-value tests are run against the engine directly. `run.py` skips them, as tests with a script.
- WPT print reftests (`-print` files) are printed on 5 × 3 inch pages with 0.5 inch margins, `reftest-pages` selects the pages to compare, and `<meta name="variant">` runs a test once per variant.
- A `rel=mismatch` pair that rasterizes the same at 96 dpi is rasterized again at 288 dpi, and if the pages differ there, the test passes: 96 dpi cannot show a difference of less than a pixel.
- A difference that disappears at 288 dpi, or when a second rasterizer draws both PDFs, is attributed to the rasterizer. Such a test still counts as failing; the second measurement only keeps it from being reported as a regression.

## Running

You need Rust (to build the comparator), Python 3.9 or later, and `pdftoppm` (poppler-utils).

```sh
cargo build --release --manifest-path compare/Cargo.toml
python3 run.py --engine prince gcpm
python3 run.py --engine weasyprint gcpm/footnotes --maps
python3 run.py --engine command --command 'mytool {input} -o {output}' gcpm
REFLOWPDF_API_KEY=rpdf_... python3 run.py --engine reflowpdf gcpm
```

Results go to `out/results.jsonl`, one line per test, and a summary per folder is printed. `--maps` writes a difference map for every failing `rel=match` pair to `out/maps/`. With `--engine command`, `{style}` in the command line stands for a style sheet that sets the 600pt × 450pt page; for tests with `@page` it is dropped, with an option such as `--style` directly before it. Without `{style}`, that page is set by a style element after the doctype. Local URLs in `src`, `srcset`, `href`, `data`, `poster` and `xlink:href` attributes and in `url()`, `image-set()` and `@import` become absolute `file:` URLs before printing; URLs that start with `/` resolve against `--root`, this folder by default. For WPT, pass folders of a WPT checkout and `--root` the checkout.

The ReflowPDF API reads no local files, so for `--engine reflowpdf` every local image, font and style sheet is sent inside the document as a `data:` URL; `--inline` does the same for any engine. An API key comes with a free account at [app.reflowpdf.com](https://app.reflowpdf.com) (account menu, API keys). The free plan prints a limited number of pages without the ReflowPDF mark, and a run of gcpm prints 342 pages (200 documents); once the allowance is used, `run.py` reports each marked print as an error instead of judging it.

## License

MIT, as the rest of this repository, except `gcpm/wpt-ports/` and `fonts/`, which come from the web-platform-tests ([gcpm/wpt-ports/LICENSE.md](gcpm/wpt-ports/LICENSE.md), [fonts/LICENSE.md](fonts/LICENSE.md)).
