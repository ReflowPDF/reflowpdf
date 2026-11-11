//! Compares the pages of a test with the pages of its reference.
//!
//! ```text
//! reflowpdf-compare <test.ppm> <reference.ppm> [map.png] [--test=<test.html> [--ref=<reference.html>] [--root=<dir>]]
//! reflowpdf-compare <test-1.ppm>… -- <reference-1.ppm>… [--map=<prefix>] [--test=<test.html> [--ref=<reference.html>] [--root=<dir>]]
//! ```
//!
//! Pages are P6 PPM files, as `pdftoppm -r 96` writes them, given in page order. All pages are
//! judged together, as one set: the anti-aliasing allowance is a share of all their pixels, and a
//! different number of pages fails. The program prints one line:
//! `hard=… maxd=… excused=… delta1=… min_blend_depth=… near_pure=… verdict=PASS|FAIL`.
//!
//! With `--test`, the test's own `<meta name="fuzzy">` tolerance applies: the one keyed to the
//! reference named by `--ref` when the test keys one to it, else its unprefixed one. A key is a URL
//! relative to the test, or rooted (`/…`) at the suite folder given by `--root`; without `--root` a
//! rooted key names no reference. A map of the difference is written as PNG: to the third path in
//! the first form, to `<prefix>-<page>.png` with `--map`.
//! Exit status: 0 PASS, 1 FAIL, 2 usage or input error.

use std::path::Path;
use std::process::ExitCode;

use reflowpdf_compare::{
    class_map_png, diff_stats_paint, fuzzy, pair_passes, prefix_names_ref, read_ppm, Page,
};

const USAGE: &str =
    "usage: reflowpdf-compare <test.ppm> <reference.ppm> [map.png] [--test=<test.html> [--ref=<reference.html>] [--root=<dir>]]
       reflowpdf-compare <test-1.ppm>… -- <reference-1.ppm>… [--map=<prefix>] [--test=<test.html> [--ref=<reference.html>] [--root=<dir>]]";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("reflowpdf-compare: {e}");
            ExitCode::from(2)
        }
    }
}

/// Where the class maps go: one file for a single pair, `<prefix>-<page>.png` for a set.
enum Map {
    None,
    File(String),
    Prefix(String),
}

fn read_all(paths: &[String]) -> Result<Vec<Page>, String> {
    paths
        .iter()
        .map(|p| read_ppm(Path::new(p)).map_err(|e| format!("{p}: {e}")))
        .collect()
}

fn run() -> Result<bool, String> {
    let mut test = None;
    let mut reference = None;
    let mut root = None;
    let mut prefix = None;
    let mut paths = Vec::new();
    for a in std::env::args().skip(1) {
        if let Some(p) = a.strip_prefix("--test=") {
            test = Some(p.to_string());
        } else if let Some(p) = a.strip_prefix("--ref=") {
            reference = Some(p.to_string());
        } else if let Some(p) = a.strip_prefix("--root=") {
            root = Some(p.to_string());
        } else if let Some(p) = a.strip_prefix("--map=") {
            prefix = Some(p.to_string());
        } else {
            paths.push(a);
        }
    }
    let (ours, theirs, map) = match paths.iter().position(|p| p == "--") {
        Some(i) if i > 0 && i + 1 < paths.len() => (
            paths[..i].to_vec(),
            paths[i + 1..].to_vec(),
            prefix.map_or(Map::None, Map::Prefix),
        ),
        None if (paths.len() == 2 || paths.len() == 3) && prefix.is_none() => (
            vec![paths[0].clone()],
            vec![paths[1].clone()],
            paths.get(2).cloned().map_or(Map::None, Map::File),
        ),
        _ => return Err(USAGE.into()),
    };
    let a = read_all(&ours)?;
    let b = read_all(&theirs)?;
    let mut classes: Vec<Vec<u8>> = Vec::new();
    let paint = if matches!(map, Map::None) {
        None
    } else {
        Some(&mut classes)
    };
    let st = diff_stats_paint(&a, &b, paint);
    let hard = st.hard;
    // A map is drawn over the reference page; when the page sets differ in count or size there is
    // no map to draw, and the verdict says so.
    if hard != u64::MAX {
        for (i, (page, cls)) in b.iter().zip(&classes).enumerate() {
            let out = match &map {
                Map::None => break,
                Map::File(f) => f.clone(),
                Map::Prefix(p) => format!("{p}-{}.png", i + 1),
            };
            std::fs::write(&out, class_map_png(page, cls)?).map_err(|e| format!("{out}: {e}"))?;
        }
    }
    let fz = match &test {
        Some(p) => {
            fuzzy(&std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?).for_ref(|k| {
                reference.as_deref().is_some_and(|r| {
                    prefix_names_ref(
                        Path::new(p),
                        k,
                        Path::new(r),
                        root.as_deref().map(Path::new),
                    )
                })
            })
        }
        None if reference.is_some() || root.is_some() => return Err(USAGE.into()),
        None => None,
    };
    let pass = pair_passes(&st.measure(), fz);
    let verdict = if pass { "PASS" } else { "FAIL" };
    println!(
        "hard={hard} maxd={} excused={} delta1={} min_blend_depth={} near_pure={} verdict={verdict}",
        st.maxd, st.excused, st.delta1, st.min_blend_depth, st.near_pure
    );
    Ok(pass)
}
