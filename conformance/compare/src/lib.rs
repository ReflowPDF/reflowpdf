//! Reference-test pixel comparator.
//!
//! A test and its reference are printed to PDF and rasterized at 96 dpi (`pdftoppm -r 96`). This crate
//! decides whether the two rasters agree. Two rasters of the same geometry reached through different
//! constructs (a border against a filled box, text against an image of it) land on fractionally
//! different subpixels, so a rasterizer's anti-aliasing fringe differs between them. The comparator
//! forgives a difference of 1/255 and that fringe, within the limits below; a changed colour or a
//! shifted edge fails.
//!
//! The rules:
//! - a channel difference of 1/255 is the rasterizer's noise floor;
//! - a pixel that differs by more is forgiven only if, in both images, its 3×3 neighbourhood spans an
//!   edge (some channel range above 16), each side's colour lies inside the other side's range (±2),
//!   and at least one side is a strictly intermediate blend (4 or more away from both ends of the
//!   range). A pure colour of the other side at an edge is a shift, not fringe, so most pixels of a
//!   shifted edge are not forgiven;
//! - if more than 1% of all pixels would be forgiven, nothing is forgiven: that is a real difference;
//! - a pair passes when, on every page, the largest remaining channel difference is at most 1, or
//!   both that difference and the page's count of differing pixels are within the upper bounds of the
//!   tolerance the test declares for this reference with `<meta name="fuzzy">` (read as WPT reads it,
//!   judged page by page as wptrunner does).
//!
//! One named departure from WPT: the lower bounds of a declared range are read but do not fail a
//! pair. In WPT a lower bound pins the difference a browser's own rasterizer leaves (wptrunner fails a
//! screenshot that comes out *closer* to the reference, so that the annotation is kept current); it
//! does not tell a right rendering from a wrong one. Here the anti-aliasing allowance removes exactly
//! that fringe, and pdftoppm's fringe is not Chrome's: `ttwf-transform-skewx-001` declares
//! `maxDifference=63-64;totalPixels=100-200` and an exact skew leaves 51 after the allowance. Seven
//! tests of the suite fail even an identical rendering under their lower bounds.
//!
//! The same pass that counts pixels can fill a class map ([`CLASS_EQUAL`] … [`CLASS_HARD`]), so a
//! picture of the difference can never disagree with the verdict.

use std::io::Read;
use std::path::Path;

/// One raster page: width, height and packed RGB bytes (3 per pixel, rows top to bottom).
pub type Page = (u32, u32, Vec<u8>);

/// One declared tolerance, as WPT reads it: the inclusive range `(lo, hi)` of the largest channel
/// difference and the inclusive range of the count of differing pixels on a page. A single value `n`
/// is the range `(n, n)`. The verdict reads the upper bounds (see the crate documentation).
pub type Fuzzy = ((u32, u32), (u64, u64));

/// A test's `<meta name="fuzzy">` declarations in source order: the reference each one is keyed to
/// (its `<url>:` prefix as written; `None` for an unprefixed one) and its ranges.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FuzzySpec(pub Vec<(Option<String>, Fuzzy)>);

impl FuzzySpec {
    /// The tolerance for one reference (wptrunner `get_fuzzy`): the declaration keyed to it —
    /// `names_ref` says whether a written prefix names this reference — else the unprefixed one,
    /// else none.
    pub fn for_ref(&self, names_ref: impl Fn(&str) -> bool) -> Option<Fuzzy> {
        self.0
            .iter()
            .find(|(k, _)| k.as_deref().is_some_and(&names_ref))
            .or_else(|| self.0.iter().find(|(k, _)| k.is_none()))
            .map(|(_, f)| *f)
    }
}

/// Comparison of two page sets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Differing pixels left after the anti-aliasing allowance (pixels at 1/255 included);
    /// `u64::MAX` when page counts or page sizes differ.
    pub hard: u64,
    /// The largest channel difference among them (255 on a size mismatch).
    pub maxd: u32,
    /// Pixels that passed as anti-aliasing fringe. When the allowance is withdrawn (more than 1%)
    /// they are counted in `hard` as well.
    pub excused: u64,
    /// Pixels that differ by exactly 1/255.
    pub delta1: u64,
    /// Among forgiven pixels, the smallest distance from a pure end of the neighbourhood's range
    /// (0..127); low values deserve a look.
    pub min_blend_depth: u32,
    /// Forgiven pixels with a blend depth below 8.
    pub near_pure: u64,
    /// `hard` and `maxd` page by page: the verdict judges each page on its own, as wptrunner does.
    pub pages: Vec<PageDiff>,
}

/// One page's difference after the anti-aliasing allowance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageDiff {
    /// Differing pixels on the page (see [`Stats::hard`]).
    pub hard: u64,
    /// The largest channel difference among them.
    pub maxd: u32,
}

impl Stats {
    /// The part of the comparison a verdict reads.
    pub fn measure(&self) -> Measure {
        Measure(self.pages.clone())
    }

    fn mismatch() -> Self {
        let page = PageDiff {
            hard: u64::MAX,
            maxd: 255,
        };
        Stats {
            hard: page.hard,
            maxd: page.maxd,
            pages: vec![page],
            ..Stats::default()
        }
    }
}

/// What a verdict reads of a comparison: the difference of each page after the anti-aliasing
/// allowance. A page-set mismatch is one page of `u64::MAX` pixels at 255.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Measure(pub Vec<PageDiff>);

impl Measure {
    /// All differing pixels (saturating).
    pub fn hard(&self) -> u64 {
        self.0.iter().fold(0u64, |a, p| a.saturating_add(p.hard))
    }

    /// The largest channel difference on any page.
    pub fn maxd(&self) -> u32 {
        self.0.iter().map(|p| p.maxd).max().unwrap_or(0)
    }
}

/// Class map value: the pixels are equal.
pub const CLASS_EQUAL: u8 = 0;
/// Class map value: the pixels differ by 1/255, the rasterizer's noise floor.
pub const CLASS_NOISE: u8 = 1;
/// Class map value: the difference was forgiven as anti-aliasing fringe.
pub const CLASS_AA: u8 = 2;
/// Class map value: a hard difference, the kind that fails a test.
pub const CLASS_HARD: u8 = 3;

/// Compares two page sets. See [`Stats`].
pub fn diff_stats(a: &[Page], b: &[Page]) -> Stats {
    diff_stats_paint(a, b, None)
}

/// Compares two page sets and, when `paint` is given, fills it with one class map per page of `a`
/// (values [`CLASS_EQUAL`] … [`CLASS_HARD`]). The map is written by the same pass that produces the
/// numbers, so the two cannot disagree.
// Casts: coordinates are checked to be inside the page before indexing, and pages are bounded by
// u32 dimensions. The function is one pass and reads best whole.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::too_many_lines
)]
pub fn diff_stats_paint(a: &[Page], b: &[Page], mut paint: Option<&mut Vec<Vec<u8>>>) -> Stats {
    if let Some(p) = paint.as_deref_mut() {
        p.clear();
        for (w, h, _) in a {
            p.push(vec![CLASS_EQUAL; (*w as usize) * (*h as usize)]);
        }
    }
    if a.len() != b.len() {
        return Stats::mismatch();
    }
    let px = |img: &Vec<u8>, w: u32, x: i64, y: i64| -> [u8; 3] {
        let i = (y as usize * w as usize + x as usize) * 3;
        [img[i], img[i + 1], img[i + 2]]
    };
    // Per-channel range over the 3×3 neighbourhood.
    let hood_range = |img: &Vec<u8>, w: u32, h: u32, x: i64, y: i64| -> ([u8; 3], [u8; 3]) {
        let (mut lo, mut hi) = ([255u8; 3], [0u8; 3]);
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= i64::from(w) || ny >= i64::from(h) {
                    continue;
                }
                let c = px(img, w, nx, ny);
                for k in 0..3 {
                    lo[k] = lo[k].min(c[k]);
                    hi[k] = hi[k].max(c[k]);
                }
            }
        }
        (lo, hi)
    };
    let on_edge = |lo: [u8; 3], hi: [u8; 3]| -> bool { (0..3).any(|k| hi[k] - lo[k] > 16) };
    let blend_member = |c: [u8; 3], lo: [u8; 3], hi: [u8; 3]| -> bool {
        (0..3).all(|k| c[k] >= lo[k].saturating_sub(2) && c[k] <= hi[k].saturating_add(2))
    };
    // Strictly intermediate blend: the pixel must stand off BOTH ends of the range, at least in a
    // channel that carries the edge. Otherwise an honest one-pixel geometry shift (a column of the
    // other side's PURE colour at the edge) would pass as anti-aliasing, and rasterizer fringe is
    // always partial coverage, never a pure colour.
    let intermediate = |c: [u8; 3], lo: [u8; 3], hi: [u8; 3]| -> bool {
        (0..3).any(|k| {
            hi[k] - lo[k] > 16 && c[k] >= lo[k].saturating_add(4) && c[k] <= hi[k].saturating_sub(4)
        })
    };
    let (mut count, mut maxd) = (0u64, 0u32);
    let mut aa_count = 0u64;
    // Page by page: (hard, maxd, forgiven, largest forgiven difference).
    let mut per_page: Vec<(u64, u32, u64, u32)> = vec![(0, 0, 0, 0); a.len()];
    let mut d1_count = 0u64;
    let mut min_depth = u32::MAX;
    let mut near_pure = 0u64;
    let mut total_px = 0u64;
    for (pi, (pa, pb)) in a.iter().zip(b).enumerate() {
        if pa.0 != pb.0 || pa.1 != pb.1 {
            return Stats::mismatch();
        }
        let (w, h) = (pa.0, pa.1);
        total_px += u64::from(w) * u64::from(h);
        for (i, (ca, cb)) in pa.2.chunks_exact(3).zip(pb.2.chunks_exact(3)).enumerate() {
            let d = ca
                .iter()
                .zip(cb)
                .map(|(x, y)| u32::from(x.abs_diff(*y)))
                .max()
                .unwrap_or(0);
            if d == 0 {
                continue;
            }
            if d == 1 {
                d1_count += 1;
                if let Some(p) = paint.as_deref_mut() {
                    p[pi][i] = CLASS_NOISE;
                }
            }
            if d > 1 {
                let (x, y) = ((i as u32 % w) as i64, (i as u32 / w) as i64);
                let (alo, ahi) = hood_range(&pa.2, w, h, x, y);
                let (blo, bhi) = hood_range(&pb.2, w, h, x, y);
                let ta = [ca[0], ca[1], ca[2]];
                let tb = [cb[0], cb[1], cb[2]];
                if on_edge(alo, ahi)
                    && on_edge(blo, bhi)
                    && blend_member(ta, blo, bhi)
                    && blend_member(tb, alo, ahi)
                    // At least one side is a true PARTIAL blend (fringe), not the other side's
                    // pure colour (a shift).
                    && (intermediate(ta, blo, bhi) || intermediate(tb, alo, ahi))
                {
                    aa_count += 1;
                    per_page[pi].2 += 1;
                    per_page[pi].3 = per_page[pi].3.max(d);
                    // Blend depth: how far the forgiven pixel is from a PURE end of the range, in
                    // the strongest edge channel of either side.
                    let mut depth = 0u32;
                    for (c, lo, hi) in [(ta, blo, bhi), (tb, alo, ahi)] {
                        for k in 0..3 {
                            if hi[k] - lo[k] > 16 {
                                // Saturating: a debug build must give a verdict, not panic on u8
                                // overflow.
                                let dd = u32::from(
                                    c[k].saturating_sub(lo[k]).min(hi[k].saturating_sub(c[k])),
                                );
                                depth = depth.max(dd);
                            }
                        }
                    }
                    min_depth = min_depth.min(depth);
                    if depth < 8 {
                        near_pure += 1;
                    }
                    if let Some(p) = paint.as_deref_mut() {
                        p[pi][i] = CLASS_AA;
                    }
                    continue; // rasterizer fringe: not counted in maxd
                }
            }
            count += 1;
            maxd = maxd.max(d);
            per_page[pi].0 += 1;
            per_page[pi].1 = per_page[pi].1.max(d);
            if d > 1 {
                if let Some(p) = paint.as_deref_mut() {
                    p[pi][i] = CLASS_HARD;
                }
            }
        }
    }
    if aa_count > total_px / 100 {
        // Too much "anti-aliasing": not fringe, an honest difference. The allowance is withdrawn
        // as a whole, and the map must show that: pixels marked as forgiven during the pass in fact
        // failed the test.
        if let Some(p) = paint {
            for page in p.iter_mut() {
                for c in page.iter_mut() {
                    if *c == CLASS_AA {
                        *c = CLASS_HARD;
                    }
                }
            }
        }
        // The withdrawn pixels carry their own differences: the largest of them is the real one.
        let pages: Vec<PageDiff> = per_page
            .iter()
            .map(|&(h, m, ah, am)| PageDiff {
                hard: h + ah,
                maxd: m.max(am),
            })
            .collect();
        return Stats {
            hard: count + aa_count,
            maxd: pages.iter().map(|p| p.maxd).max().unwrap_or(0),
            excused: aa_count,
            delta1: d1_count,
            min_blend_depth: 0,
            near_pure,
            pages,
        };
    }
    Stats {
        hard: count,
        maxd,
        excused: aa_count,
        delta1: d1_count,
        min_blend_depth: if min_depth == u32::MAX { 0 } else { min_depth },
        near_pure,
        pages: per_page
            .iter()
            .map(|&(h, m, ..)| PageDiff { hard: h, maxd: m })
            .collect(),
    }
}

/// The verdict for a pair (`fuzzy` — the tolerance the test declares for this reference): every page
/// passes on its own (wptrunner `check_pass` per page), by the 1/255 noise floor or within the upper
/// bounds of the tolerance. Lower bounds do not fail a page (see the crate documentation).
pub fn pair_passes(m: &Measure, fuzzy: Option<Fuzzy>) -> bool {
    m.0.iter().all(|p| {
        p.maxd <= 1 || fuzzy.is_some_and(|((_, dhi), (_, phi))| p.maxd <= dhi && p.hard <= phi)
    })
}

/// The tolerances a test declares with `<meta name="fuzzy" content="[<url>:]<range>;<range>">` (WPT
/// `SourceFile.fuzzy`): the optional prefix — up to the last colon — keys a declaration to one
/// reference; each range is `lo-hi` or a single value `n` (the range `n-n`), optionally named
/// `maxDifference=` or `totalPixels=` in either order, unnamed ones filling the rest in that order.
/// A malformed declaration (WPT rejects the test for it) declares nothing. Names are case-sensitive,
/// as in WPT.
pub fn fuzzy(src: &str) -> FuzzySpec {
    let l = src.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = l[i..].find("<meta") {
        let start = i + p;
        let end = l[start..].find('>').map_or(l.len(), |e| start + e);
        // The lowercased tag finds the attributes; the source keeps the reference's own spelling.
        let (tl, tag) = (&l[start..end], &src[start..end]);
        if attr(tl, tag, "name").as_deref() == Some("fuzzy") {
            if let Some(d) = attr(tl, tag, "content")
                .as_deref()
                .and_then(fuzzy_declaration)
            {
                out.push(d);
            }
        }
        i = end;
    }
    FuzzySpec(out)
}

fn fuzzy_declaration(content: &str) -> Option<(Option<String>, Fuzzy)> {
    let (key, value) = match content.rfind(':') {
        // An empty prefix names no reference: WPT rejects it.
        Some(c) => (
            Some(content[..c].trim())
                .filter(|k| !k.is_empty())
                .map(Some)?,
            &content[c + 1..],
        ),
        None => (None, content),
    };
    let parts: Vec<&str> = value.split(';').collect();
    if parts.len() != 2 {
        return None;
    }
    let (mut max_difference, mut total_pixels, mut positional) = (None, None, Vec::new());
    for part in parts {
        let (name, v) = match part.split_once('=') {
            Some((n, v)) => (Some(n.trim()), v),
            None => (None, part),
        };
        let range: (u64, u64) = if let Some((lo, hi)) = v.split_once('-') {
            (lo.trim().parse().ok()?, hi.trim().parse().ok()?)
        } else {
            let n = v.trim().parse().ok()?;
            (n, n)
        };
        let slot = match name {
            None => {
                positional.push(range);
                continue;
            }
            Some("maxDifference") => &mut max_difference,
            Some("totalPixels") => &mut total_pixels,
            Some(_) => return None,
        };
        if slot.replace(range).is_some() {
            return None;
        }
    }
    let mut rest = positional.into_iter();
    let md = max_difference.or_else(|| rest.next())?;
    let tp = total_pixels.or_else(|| rest.next())?;
    if rest.next().is_some() {
        return None;
    }
    let md = (u32::try_from(md.0).ok()?, u32::try_from(md.1).ok()?);
    Some((key.map(str::to_string), (md, tp)))
}

/// Whether a fuzzy prefix written in `test` names the reference at `reference`: the prefix is a URL
/// relative to the test, or rooted (`/…`) at the suite root `root`, compared by its normalized path. A
/// rooted prefix names nothing when the root is not known.
pub fn prefix_names_ref(test: &Path, prefix: &str, reference: &Path, root: Option<&Path>) -> bool {
    let norm = |p: &Path| -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for c in p.components() {
            match c {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    out.pop();
                }
                c => out.push(c.as_os_str().to_string_lossy().into_owned()),
            }
        }
        out
    };
    let want = norm(reference);
    match prefix.strip_prefix('/') {
        Some(rooted) => root.is_some_and(|r| norm(&r.join(rooted)) == want),
        None => test
            .parent()
            .is_some_and(|dir| norm(&dir.join(prefix)) == want),
    }
}

/// The value of `name="…"` inside a tag. `tl` is the tag lowercased (where the name is searched),
/// `tag` the original (where an unquoted value is read).
pub fn attr(tl: &str, tag: &str, name: &str) -> Option<String> {
    let at = tl.find(&format!("{name}="))? + name.len() + 1;
    let quote = tl.as_bytes().get(at).copied()?;
    // An unquoted value is legal HTML and reftests use it (`<meta name=fuzzy content="0-5;0-375">`).
    // Refusing it would drop a test's declared tolerance and judge it exactly.
    if quote != b'"' && quote != b'\'' {
        let rest = &tag[at..];
        // Whitespace or `>` ends it, not a bare `/`: that lives inside every relative href
        // (`href=../reference/x.xht`). Only a TRAILING `/` is the self-closing slash.
        let end = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '>')
            .unwrap_or(rest.len());
        let v = rest[..end].trim_end_matches('/');
        return (!v.is_empty()).then(|| v.to_string());
    }
    let rest = &tag[at + 1..];
    let end = rest.find(quote as char)?;
    Some(rest[..end].to_string())
}

/// Reads a binary P6 PPM file (`pdftoppm`'s native output).
///
/// # Errors
/// The file cannot be read or is not a complete binary PPM.
pub fn read_ppm(p: &Path) -> Result<Page, String> {
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    f.read_to_end(&mut data).map_err(|e| e.to_string())?;
    parse_ppm(&data)
}

/// Parses binary P6 PPM bytes (maxval 255).
///
/// # Errors
/// The bytes are not a P6 header followed by width × height × 3 bytes of samples.
pub fn parse_ppm(data: &[u8]) -> Result<Page, String> {
    if !data.starts_with(b"P6") {
        return Err("not P6".into());
    }
    // Header = three whitespace-separated ints after magic (comments start with '#').
    let mut pos = 2;
    let mut vals = [0u32; 3];
    for v in &mut vals {
        while pos < data.len() && (data[pos].is_ascii_whitespace() || data[pos] == b'#') {
            if data[pos] == b'#' {
                while pos < data.len() && data[pos] != b'\n' {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }
        let start = pos;
        while pos < data.len() && data[pos].is_ascii_digit() {
            pos += 1;
        }
        *v = std::str::from_utf8(&data[start..pos])
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or("bad header")?;
    }
    pos += 1; // single whitespace after maxval
    let (w, h) = (vals[0], vals[1]);
    let need = (w * h * 3) as usize;
    if data.len() < pos + need {
        return Err("truncated ppm".into());
    }
    Ok((w, h, data[pos..pos + need].to_vec()))
}

/// Encodes an RGB image as PNG, at the best compression (maps are written once and viewed often).
///
/// # Errors
/// The encoder refuses the image (`rgb` is not `w × h × 3` bytes).
pub fn png_bytes(w: u32, h: u32, rgb: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Best);
        enc.set_filter(png::FilterType::Paeth);
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        wr.write_image_data(rgb).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// A class map as PNG: the reference page lightened underneath, so the difference shows on what it
/// sits; magenta for hard differences, amber for forgiven fringe, pale amber for 1/255 noise.
///
/// # Errors
/// As [`png_bytes`].
pub fn class_map_png(ref_page: &Page, classes: &[u8]) -> Result<Vec<u8>, String> {
    let (w, h) = (ref_page.0, ref_page.1);
    let n = (w as usize) * (h as usize);
    let mut rgb = vec![0u8; n * 3];
    for i in 0..n {
        let out = match classes.get(i).copied().unwrap_or(CLASS_EQUAL) {
            CLASS_HARD => [0xC2, 0x18, 0x5B],
            CLASS_AA => [0xB0, 0x70, 0x0A],
            CLASS_NOISE => [0xD9, 0xB3, 0x66],
            _ => {
                // The backdrop is quantized to steps of 16: anti-aliased edges of the reference
                // otherwise add noise deflate cannot compress, and the backdrop would cost more
                // than the map.
                let p = &ref_page.2[i * 3..i * 3 + 3];
                let q = |v: u8| 200 + (v / 5 / 16) * 16;
                [q(p[0]), q(p[1]), q(p[2])]
            }
        };
        rgb[i * 3..i * 3 + 3].copy_from_slice(&out);
    }
    png_bytes(w, h, &rgb)
}

#[cfg(test)]
mod tests;
