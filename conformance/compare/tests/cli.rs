//! The command line: both forms, page sets, maps and exit status.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh directory under the build's own target directory (`CARGO_TARGET_TMPDIR`), so that
/// nothing is left in the system's temporary directory.
fn dir(name: &str) -> PathBuf {
    let d =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

/// A 40×40 white page with a black square whose left edge is at `left`.
fn page(d: &Path, name: &str, left: usize) -> String {
    let mut data = b"P6\n40 40\n255\n".to_vec();
    for _y in 0..40 {
        for x in 0..40 {
            data.extend_from_slice(if (left..left + 10).contains(&x) {
                &[0, 0, 0]
            } else {
                &[255, 255, 255]
            });
        }
    }
    let p = d.join(name);
    std::fs::write(&p, data).expect("ppm");
    p.to_string_lossy().into_owned()
}

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_reflowpdf-compare"))
        .args(args)
        .output()
        .expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn single_pair_and_map() {
    let d = dir("pair");
    let (a, b) = (page(&d, "a.ppm", 10), page(&d, "b.ppm", 11));
    let map = d.join("map.png").to_string_lossy().into_owned();
    assert_eq!(
        run(&[&a, &a]),
        (
            0,
            "hard=0 maxd=0 excused=0 delta1=0 min_blend_depth=0 near_pure=0 verdict=PASS\n".into()
        )
    );
    let (code, line) = run(&[&a, &b, &map]);
    assert_eq!(code, 1, "{line}");
    assert!(line.starts_with("hard=80 maxd=255 "), "{line}");
    assert!(std::fs::read(&map).expect("map").starts_with(b"\x89PNG"));
}

#[test]
fn page_sets_are_judged_together() {
    let d = dir("set");
    let (a1, a2, b2) = (
        page(&d, "a1.ppm", 10),
        page(&d, "a2.ppm", 20),
        page(&d, "b2.ppm", 21),
    );
    assert_eq!(run(&[&a1, &a2, "--", &a1, &a2]).0, 0);
    let prefix = d.join("m").to_string_lossy().into_owned();
    let (code, line) = run(&[&a1, &a2, "--", &a1, &b2, &format!("--map={prefix}")]);
    assert_eq!(code, 1, "{line}");
    assert!(d.join("m-1.png").exists() && d.join("m-2.png").exists());
    let (code, line) = run(&[&a1, &a2, "--", &a1]);
    assert_eq!(code, 1, "a missing page fails: {line}");
    assert!(line.starts_with(&format!("hard={} ", u64::MAX)), "{line}");
}

#[test]
fn declared_tolerance_and_usage() {
    let d = dir("fuzzy");
    let (a, b) = (page(&d, "a.ppm", 10), page(&d, "b.ppm", 11));
    let t = d.join("t.html");
    std::fs::write(&t, "<meta name=fuzzy content=\"0-255;0-80\">").expect("html");
    assert_eq!(run(&[&a, &b, &format!("--test={}", t.display())]).0, 0);
    assert_eq!(run(&[&a]).0, 2);
    assert_eq!(run(&[&a, "--"]).0, 2);
    assert_eq!(run(&[&a, &d.join("none.ppm").to_string_lossy()]).0, 2);
}

#[test]
fn a_tolerance_keyed_to_a_reference_needs_ref() {
    let d = dir("keyed");
    std::fs::create_dir_all(d.join("sub/reference")).expect("dirs");
    let (a, b) = (page(&d, "a.ppm", 10), page(&d, "b.ppm", 11));
    let t = d.join("sub/t.html");
    std::fs::write(
        &t,
        "<meta name=fuzzy content=\"reference/r.html:0-255;0-80\">",
    )
    .expect("html");
    let test = format!("--test={}", t.display());
    assert_eq!(
        run(&[&a, &b, &test]).0,
        1,
        "keyed to a reference not named: exact"
    );
    let r = format!("--ref={}", d.join("sub/reference/r.html").display());
    assert_eq!(run(&[&a, &b, &test, &r]).0, 0);
    let other = format!("--ref={}", d.join("sub/reference/other.html").display());
    assert_eq!(run(&[&a, &b, &test, &other]).0, 1);
    std::fs::write(
        &t,
        "<meta name=fuzzy content=\"/sub/reference/r.html:0-255;0-80\">",
    )
    .expect("html");
    assert_eq!(run(&[&a, &b, &test, &r]).0, 1, "a rooted key needs --root");
    let root = format!("--root={}", d.display());
    assert_eq!(run(&[&a, &b, &test, &r, &root]).0, 0);
    assert_eq!(run(&[&a, &b, &r]).0, 2, "--ref without --test");
}
