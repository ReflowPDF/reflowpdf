use super::*;

const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];

fn page(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> Page {
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            rgb.extend_from_slice(&f(x, y));
        }
    }
    (w, h, rgb)
}

/// A black bar from column `left` to the right edge, with an optional grey fringe column just left
/// of it (partial coverage, as a rasterizer leaves at a subpixel edge).
fn bar(left: u32, fringe: Option<u8>) -> Page {
    page(200, 200, |x, _| match (x, fringe) {
        (x, _) if x >= left => BLACK,
        (x, Some(g)) if x + 1 == left => [g; 3],
        _ => WHITE,
    })
}

#[test]
fn equal_pages_have_no_difference() {
    let a = bar(100, None);
    let st = diff_stats(std::slice::from_ref(&a), std::slice::from_ref(&a));
    assert_eq!((st.hard, st.maxd, st.excused), (0, 0, 0));
    assert_eq!(st.pages, vec![PageDiff::default()]);
    assert!(pair_passes(&st.measure(), None));
}

#[test]
fn one_pixel_shift_fails() {
    // The other side's PURE colour at an edge is a shift, not fringe.
    let st = diff_stats(&[bar(100, None)], &[bar(101, None)]);
    assert_eq!(st.hard, 200, "{st:?}");
    assert_eq!(st.maxd, 255);
    assert_eq!(st.excused, 0, "nothing forgiven");
    assert!(!pair_passes(&st.measure(), None));
}

#[test]
fn anti_aliasing_fringe_is_forgiven() {
    // Same edge, different partial coverage of the fringe column.
    let st = diff_stats(&[bar(100, Some(120))], &[bar(100, Some(160))]);
    assert_eq!(st.hard, 0, "{st:?}");
    assert_eq!(st.excused, 200, "every fringe pixel forgiven");
    assert_eq!(
        st.min_blend_depth, 120,
        "blend depth: 120 from black, 135 from white"
    );
    assert!(pair_passes(&st.measure(), None));
}

#[test]
fn fringe_on_one_side_is_forgiven_a_pure_flip_is_not() {
    // Fringe on one side only, pure on the other: the fringe side is a blend inside the other
    // side's range, so the pixel is forgiven; a pure-against-pure flip is not.
    let st = diff_stats(&[bar(100, Some(120))], &[bar(100, None)]);
    assert_eq!(st.hard, 0, "{st:?}");
    let flip = page(200, 200, |x, _| if x >= 99 { BLACK } else { WHITE });
    let st = diff_stats(&[bar(100, None)], &[flip]);
    assert_eq!(st.hard, 200, "{st:?}");
}

#[test]
fn colour_change_fails() {
    let a = page(50, 50, |x, _| if x < 25 { [200, 30, 30] } else { WHITE });
    let b = page(50, 50, |x, _| if x < 25 { [30, 30, 200] } else { WHITE });
    let st = diff_stats(&[a], &[b]);
    assert!(st.hard >= 25 * 49, "{st:?}");
    assert_eq!(st.maxd, 170);
    assert!(!pair_passes(&st.measure(), None));
}

#[test]
fn one_level_is_noise() {
    let a = page(20, 20, |_, _| [128; 3]);
    let b = page(20, 20, |_, _| [129, 128, 128]);
    let st = diff_stats(&[a], &[b]);
    assert_eq!((st.hard, st.maxd, st.delta1), (400, 1, 400));
    assert!(pair_passes(&st.measure(), None));
}

#[test]
fn size_or_page_count_mismatch_fails() {
    let a = page(10, 10, |_, _| WHITE);
    let b = page(10, 11, |_, _| WHITE);
    assert_eq!(diff_stats(std::slice::from_ref(&a), &[b]).hard, u64::MAX);
    assert_eq!(
        diff_stats(std::slice::from_ref(&a), &[a.clone(), a.clone()]).hard,
        u64::MAX
    );
}

#[test]
fn allowance_withdrawn_above_one_percent() {
    // Fringe columns every 4 px: far more than 1% of the page would be forgiven.
    let stripes = |g: u8| {
        page(200, 200, |x, _| match x % 4 {
            0 | 1 => BLACK,
            2 => [g; 3],
            _ => WHITE,
        })
    };
    let mut maps = Vec::new();
    let st = diff_stats_paint(&[stripes(100)], &[stripes(160)], Some(&mut maps));
    assert!(st.excused > 400, "{st:?}");
    assert_eq!(st.hard, st.excused, "every would-be fringe pixel counts");
    // The withdrawn pixels' own difference is the real one: grey 100 against grey 160.
    assert_eq!(st.maxd, 60);
    assert_eq!(
        st.pages,
        vec![PageDiff {
            hard: st.hard,
            maxd: 60
        }]
    );
    assert!(
        maps[0].iter().all(|&c| c != CLASS_AA),
        "the map shows the withdrawal"
    );
}

#[test]
fn class_map_matches_the_numbers() {
    let a = bar(100, Some(120));
    let mut b = bar(100, Some(160));
    b.2[0] = 254; // one noise pixel at the corner
    let i = (150 * 200 + 50) * 3; // one hard pixel in the white area
    b.2[i..i + 3].copy_from_slice(&[0, 0, 255]);
    let mut maps = Vec::new();
    let st = diff_stats_paint(&[a], &[b.clone()], Some(&mut maps));
    let n = |c: u8| {
        maps[0]
            .iter()
            .filter(|&&v| v == c)
            .map(|_| 1u64)
            .sum::<u64>()
    };
    assert_eq!(n(CLASS_AA), st.excused);
    assert_eq!(n(CLASS_NOISE), st.delta1);
    assert_eq!(n(CLASS_HARD) + n(CLASS_NOISE), st.hard);
    assert_eq!(
        st.pages,
        vec![PageDiff {
            hard: st.hard,
            maxd: st.maxd
        }]
    );
    let png = class_map_png(&b, &maps[0]).expect("png");
    assert!(png.starts_with(b"\x89PNG"));
}

#[test]
fn fuzzy_declarations() {
    let one = |c: &str| -> Option<(Option<String>, Fuzzy)> {
        let spec = fuzzy(&format!(
            "<html><head><meta name=\"fuzzy\" content=\"{c}\"></head>"
        ));
        assert!(spec.0.len() <= 1);
        spec.0.into_iter().next()
    };
    let r =
        |d: (u32, u32), p: (u64, u64)| -> Option<(Option<String>, Fuzzy)> { Some((None, (d, p))) };
    assert_eq!(one("0-5;0-375"), r((0, 5), (0, 375)));
    assert_eq!(
        one("maxDifference=0-3;totalPixels=0-100"),
        r((0, 3), (0, 100))
    );
    // Named in the other order: WPT assigns by name, not by position.
    assert_eq!(one("totalPixels=0-2;maxDifference=0-1"), r((0, 1), (0, 2)));
    assert_eq!(
        one("maxDifference=1; totalPixels=0-5000"),
        r((1, 1), (0, 5000)),
        "a single value is n-n"
    );
    assert_eq!(one("10-20;300-400"), r((10, 20), (300, 400)));
    assert_eq!(
        one("totalPixels=5-6;7-8"),
        r((7, 8), (5, 6)),
        "an unnamed range fills the rest"
    );
    assert_eq!(
        one("transform3d-matrix3d-001-ref.html:0-100;0-980"),
        Some((
            Some("transform3d-matrix3d-001-ref.html".to_string()),
            ((0, 100), (0, 980))
        )),
        "the prefix keeps its spelling"
    );
    for bad in [
        "0-5",
        "0-5;0-5;0-5",
        "x;1",
        "1-2-3;4",
        "maxDifference=1;maxDifference=2",
        "speed=1;2",
    ] {
        assert_eq!(one(bad), None, "{bad}");
    }
    assert_eq!(
        fuzzy("<meta name=fuzzy content=\"0-5;0-375\">").for_ref(|_| false),
        Some(((0, 5), (0, 375))),
        "unquoted name"
    );
    assert_eq!(
        fuzzy("<META NAME='fuzzy' CONTENT='0-2;0-9'>").for_ref(|_| false),
        Some(((0, 2), (0, 9)))
    );
    // Names are case-sensitive, as in WPT, and an empty prefix names no reference.
    assert_eq!(
        fuzzy("<meta name='Fuzzy' content='0-2;0-9'>"),
        FuzzySpec::default()
    );
    assert_eq!(one("maxdifference=0-2;0-9"), None);
    assert_eq!(one(":0-2;0-9"), None);
    assert_eq!(
        fuzzy("<meta name=\"assert\" content=\"x\">"),
        FuzzySpec::default()
    );
    assert_eq!(fuzzy("<p>no meta</p>"), FuzzySpec::default());
}

#[test]
fn a_keyed_tolerance_applies_to_its_reference_only() {
    let spec = fuzzy(
        "<meta name=fuzzy content='a-ref.html:0-9;0-90'><meta name=fuzzy content='0-1;0-10'>\
         <meta name=fuzzy content='../b/b-ref.html:0-4;0-40'>",
    );
    let t = Path::new("/w/css/x/test.html");
    let pick = |r: &str| spec.for_ref(|k| prefix_names_ref(t, k, Path::new(r), None));
    assert_eq!(pick("/w/css/x/a-ref.html"), Some(((0, 9), (0, 90))));
    assert_eq!(
        pick("/w/css/x/./a-ref.html"),
        Some(((0, 9), (0, 90))),
        "compared as paths"
    );
    assert_eq!(pick("/w/css/b/b-ref.html"), Some(((0, 4), (0, 40))));
    assert_eq!(
        pick("/w/css/x/c-ref.html"),
        Some(((0, 1), (0, 10))),
        "others take the unprefixed one"
    );
    let w = Some(Path::new("/w"));
    assert!(
        prefix_names_ref(t, "/css/x/a-ref.html", Path::new("/w/css/x/a-ref.html"), w),
        "rooted at the suite"
    );
    assert!(
        !prefix_names_ref(t, "/a-ref.html", Path::new("/w/css/x/a-ref.html"), w),
        "not by the path's tail"
    );
    assert!(
        !prefix_names_ref(
            t,
            "/css/x/a-ref.html",
            Path::new("/w/css/x/a-ref.html"),
            None
        ),
        "no root, no rooted name"
    );
    assert_eq!(
        fuzzy("<meta name=fuzzy content='a-ref.html:0-9;0-90'>").for_ref(|_| false),
        None
    );
}

#[test]
fn verdict_respects_declared_tolerance() {
    let m = |hard: u64, maxd: u32| Measure(vec![PageDiff { hard, maxd }]);
    let f = Some(((0, 5), (0, 375)));
    assert!(pair_passes(&m(375, 5), f));
    assert!(!pair_passes(&m(376, 5), f));
    assert!(!pair_passes(&m(375, 6), f));
    assert!(pair_passes(&m(10_000, 1), None), "noise floor");
    assert!(!pair_passes(&m(1, 2), None));
    assert!(
        !pair_passes(&m(1, 2), Some(((0, 0), (0, 0)))),
        "0;0 is no tolerance"
    );
}

#[test]
fn lower_bounds_do_not_fail_a_pair() {
    let m = |hard: u64, maxd: u32| Measure(vec![PageDiff { hard, maxd }]);
    // `ttwf-transform-skewx-001`: Chrome's fringe pinned at 63-64; an exact skew here leaves 51.
    let f = Some(((63, 64), (100, 200)));
    assert!(pair_passes(&m(100, 51), f));
    assert!(pair_passes(&m(0, 0), f), "identity passes");
    assert!(!pair_passes(&m(100, 65), f), "the upper bounds still hold");
    assert!(!pair_passes(&m(201, 60), f));
}

#[test]
fn pages_are_judged_one_by_one() {
    // Two pages of 150 differing pixels under `0-200`: each page is within it, the sum is not.
    let two = Measure(vec![
        PageDiff {
            hard: 150,
            maxd: 30,
        },
        PageDiff {
            hard: 150,
            maxd: 30,
        },
    ]);
    assert!(pair_passes(&two, Some(((0, 50), (0, 200)))));
    assert_eq!((two.hard(), two.maxd()), (300, 30));
    let st = diff_stats(
        &[bar(100, None), bar(100, None)],
        &[bar(100, None), bar(101, None)],
    );
    assert_eq!(
        st.pages,
        vec![
            PageDiff::default(),
            PageDiff {
                hard: 200,
                maxd: 255
            }
        ]
    );
    assert!(!pair_passes(&st.measure(), None));
}

#[test]
fn attr_reads_unquoted_href_with_slashes() {
    let t = "<link rel=match href=../reference/x.xht/>";
    assert_eq!(attr(t, t, "href").as_deref(), Some("../reference/x.xht"));
    assert_eq!(attr(t, t, "rel").as_deref(), Some("match"));
    assert_eq!(attr(t, t, "type"), None);
}

#[test]
fn ppm_header_with_comment() {
    let mut data = b"P6\n# pdftoppm\n2 1\n255\n".to_vec();
    data.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(parse_ppm(&data), Ok((2, 1, vec![1, 2, 3, 4, 5, 6])));
    assert!(parse_ppm(b"P6 2 1 255 \x01").is_err(), "truncated");
    assert!(parse_ppm(b"P3 1 1 255 0 0 0").is_err(), "not binary");
}
