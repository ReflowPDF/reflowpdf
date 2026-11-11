#!/usr/bin/env python3
"""Run reftests through an HTML-to-PDF engine and judge them with reflowpdf-compare.

    python3 run.py --engine prince gcpm
    python3 run.py --engine weasyprint gcpm/footnotes
    python3 run.py --engine command --command 'mytool {input} -o {output}' gcpm
    REFLOWPDF_API_KEY=rpdf_... python3 run.py --engine reflowpdf gcpm

A reftest is an HTML file with <link rel="match" href="..."> or <link rel="mismatch" href="...">.
The test and each reference are printed to PDF by the engine, rasterized by pdftoppm at 96 dpi, and
the page images are compared by reflowpdf-compare (compare/). The test passes when it matches one of
its match references and differs from all of its mismatch references.

A test that declares @page (or the `paged` flag) is printed on its own pages, and all its pages are
compared as one set. Any other test is printed on one 600pt x 450pt page (800 x 600 CSS px) with no
margin, and only the first page is compared.

URLs that start with "/" resolve against --root (this directory by default), in documents and in
style sheets alike, so /fonts/ahem.css is fonts/ahem.css here. Every local URL becomes a file: URL
before printing (a data: URL with --inline), so the engine reads the same files wherever the run
happens. Results go to
<out>/results.jsonl, one line per test.

Needs Python 3.9+, pdftoppm (poppler-utils) and the compare binary (cargo build --release in
compare/, or --compare / REFLOWPDF_COMPARE).
"""
import argparse
import base64
import json
import mimetypes
import os
import pathlib
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
VIEWPORT = "@page { size: 600pt 450pt; margin: 0 }\n"
API = "https://api.reflowpdf.com/v1/pdf"
TIMEOUT = 120
SKIP_FLAGS = ("interact", "animated", "http", "userstyle", "scroll")
XHTML = (".xht", ".xhtml")

LINK = re.compile(r"<link\b[^>]*>", re.I)
META = re.compile(r"<meta\b[^>]*>", re.I)
VOID = frozenset("area base br col embed hr img input link meta param source track wbr".split())
SELF_CLOSING = re.compile(r"<([a-zA-Z][a-zA-Z0-9:-]*)((?:[^<>'\"]|'[^']*'|\"[^\"]*\")*?)/>")
# What may stand before the doctype: a byte order mark (read as latin-1), white space, comments, an XML declaration.
DOCTYPE = re.compile(r"^(?:\xef\xbb\xbf)?(?:\s|<!--.*?-->|<\?xml[^>]*\?>)*<!doctype[^>]*>", re.I | re.S)
# A URL in a document: an attribute that loads something, url() and @import in style.
HTML_URL = re.compile(r"""(\b(?:src|href|xlink:href|data|poster)\s*=\s*)(?:(["'])(.*?)\2|([^\s"'>]+))""", re.I | re.S)
SRCSET = re.compile(r"""(\bsrcset\s*=\s*)(["'])(.*?)\2""", re.I | re.S)
IMAGE_SET = re.compile(r"(?:-webkit-)?image-set\(", re.I)
CSS_URL = re.compile(r"""(url\(\s*)(["']?)([^"')]*?)\2(\s*\))|(@import\s+)(["'])(.*?)\6""", re.I | re.S)


def attr(tag, name):
    m = re.search(r"\b" + name + r"""\s*=\s*("([^"]*)"|'([^']*)'|([^\s>]+))""", tag, re.I)
    if not m:
        return None
    return next(g for g in m.groups()[1:] if g is not None)


def read(path):
    """A file's text, byte for byte: every byte maps to one character and back (latin-1)."""
    with open(path, "rb") as f:
        return f.read().decode("latin-1")


def charset(text):
    """The document's own encoding, for an engine that is sent text rather than bytes."""
    if text.startswith("\xef\xbb\xbf"):
        return "utf-8"
    m = re.search(r"""<meta[^>]+charset\s*=\s*["']?([\w-]+)|<\?xml[^>]+encoding\s*=\s*["']([\w-]+)""", text[:4096], re.I)
    return (m.group(1) or m.group(2)) if m else "utf-8"


def meta(src, name):
    for tag in META.findall(src):
        if (attr(tag, "name") or "").lower() == name:
            return attr(tag, "content") or ""
    return None


def not_run(src):
    """Why a file cannot be judged on a printed page, or None."""
    low = src.lower()
    if "<script" in low or re.search(r"\bon(load|readystatechange)\s*=", low):
        return "script"
    if "reftest-wait" in low:
        return "reftest-wait"
    for flag in (meta(src, "flags") or "").lower().split():
        if flag in SKIP_FLAGS:
            return "flag:" + flag
    return None


class Resolver:
    """Local URLs made absolute: as file: URLs for an engine that reads files, or as data: URLs for one
    that does not (the HTTP API). Style sheets are rewritten the same way, recursively."""

    def __init__(self, root, work, inline):
        self.root, self.work, self.inline = root, work, inline
        self.sheets = {}

    def local(self, ref, here):
        ref = ref.strip()
        if not ref or re.match(r"^(?:[a-z][a-z0-9+.-]*:|#|//)", ref, re.I):
            return None
        path = urllib.parse.unquote(ref.split("#", 1)[0].split("?", 1)[0])
        path = os.path.join(self.root, path.lstrip("/")) if path.startswith("/") else os.path.join(here, path)
        return os.path.normpath(path)

    def url(self, ref, here):
        path = self.local(ref, here)
        if path is None or not os.path.isfile(path):
            return ref
        fragment = "#" + ref.split("#", 1)[1] if "#" in ref else ""
        if path.lower().endswith(".css"):
            return self.sheet(path) + fragment
        if self.inline:
            kind = mimetypes.guess_type(path)[0] or "application/octet-stream"
            with open(path, "rb") as f:
                return f"data:{kind};base64,{base64.b64encode(f.read()).decode()}" + fragment
        return pathlib.Path(path).as_uri() + fragment

    def image_sets(self, text, here):
        """Strings inside image-set(): each names an image, as url() does."""
        out, at = [], 0
        for m in IMAGE_SET.finditer(text):
            if m.start() < at:
                continue
            depth, i = 1, m.end()
            while i < len(text) and depth:
                depth += {"(": 1, ")": -1}.get(text[i], 0)
                i += 1
            inner = re.sub(r"""(["'])(.*?)\1""", lambda q: q.group(1) + self.url(q.group(2), here) + q.group(1), text[m.end():i - 1])
            out.append(text[at:m.end()] + inner + ")")
            at = i
        return "".join(out) + text[at:]

    def css(self, text, here):
        text = self.image_sets(text, here)
        def one(m):
            if m.group(1):
                return m.group(1) + m.group(2) + self.url(m.group(3), here) + m.group(2) + m.group(4)
            return m.group(5) + m.group(6) + self.url(m.group(7), here) + m.group(6)
        return CSS_URL.sub(one, text)

    def sheet(self, path):
        if path not in self.sheets:
            self.sheets[path] = ""  # an @import cycle ends here
            text = self.css(read(path), os.path.dirname(path))
            if self.inline:
                self.sheets[path] = "data:text/css;base64," + base64.b64encode(text.encode("latin-1")).decode()
            else:
                copy = os.path.join(self.work, uuid.uuid4().hex + ".css")
                with open(copy, "wb") as f:
                    f.write(text.encode("latin-1"))
                self.sheets[path] = pathlib.Path(copy).as_uri()
        return self.sheets[path]

    def document(self, text, here):
        def one(m):
            if m.group(2):
                return m.group(1) + m.group(2) + self.url(m.group(3), here) + m.group(2)
            return m.group(1) + '"' + self.url(m.group(4), here) + '"'
        text = HTML_URL.sub(one, text)
        def srcset(m):
            candidates = []
            for part in m.group(3).split(","):
                bits = part.strip().split(None, 1)
                if bits:
                    candidates.append(" ".join([self.url(bits[0], here)] + bits[1:]))
            return m.group(1) + m.group(2) + ", ".join(candidates) + m.group(2)
        text = SRCSET.sub(srcset, text)
        return self.css(text, here)


def refs(src, path, kind, root):
    out = []
    for tag in LINK.findall(src):
        if (attr(tag, "rel") or "").lower() == kind and attr(tag, "href"):
            href = attr(tag, "href")
            base = root if href.startswith("/") else os.path.dirname(path)
            out.append(os.path.normpath(os.path.join(base, href.lstrip("/"))))
    return out


def discover(paths, root):
    cases = []
    for top in paths:
        top = os.path.abspath(top)
        files = [top] if os.path.isfile(top) else sorted(
            os.path.join(d, n) for d, _, names in os.walk(top) for n in names)
        for f in files:
            name = os.path.basename(f)
            stem = os.path.splitext(name)[0]
            inside = "/" + os.path.relpath(f, top if os.path.isdir(top) else os.path.dirname(top)) + "/"
            if name.startswith(".") or not name.endswith((".html", ".htm") + XHTML):
                continue
            if any(d in inside for d in ("/reference/", "/support/", "/withdrawn/")) or stem.endswith("-ref") or "-ref-" in stem:
                continue
            src = read(f)
            match, mismatch = refs(src, f, "match", root), refs(src, f, "mismatch", root)
            if not match and not mismatch:
                continue
            skip = not_run(src)
            for r in match + mismatch:
                if skip:
                    break
                if not os.path.isfile(r):
                    skip = "reference missing: " + os.path.relpath(r, root)
                elif not_run(read(r)):
                    skip = "reference " + not_run(read(r))
            flags = (meta(src, "flags") or "").lower().split()
            cases.append({
                "test": os.path.relpath(f, root), "path": f, "match": match, "mismatch": mismatch,
                "paged": "@page" in src or "paged" in flags, "skip": skip,
            })
    return cases


def run_command(cmd, timeout=TIMEOUT):
    r = subprocess.run(cmd, capture_output=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError((r.stderr or r.stdout or b"failed").decode(errors="replace").strip()[:200])


class Runner:
    def __init__(self, args):
        self.args, self.root = args, os.path.abspath(args.root)
        self.work = tempfile.mkdtemp(prefix="reftest-")
        self.viewport = os.path.join(self.work, "viewport.css")
        with open(self.viewport, "w") as f:
            f.write(VIEWPORT)
        self.api = args.engine == "reflowpdf"
        if self.api:
            self.key = os.environ.get("REFLOWPDF_API_KEY")
            if not self.key:
                sys.exit("--engine reflowpdf needs REFLOWPDF_API_KEY (app.reflowpdf.com: account menu, API keys)")

    def print_pdf(self, path, pdf, paged):
        here = os.path.dirname(path)
        xhtml = path.endswith(XHTML)
        resolver = Resolver(self.root, self.work, inline=self.api or self.args.inline)
        text = read(path)
        if self.api:
            body = {"html": resolver.document(text, here).encode("latin-1").decode(charset(text), errors="replace"),
                    "filename": "test.pdf", "options": {
                        **({"input_format": "xhtml"} if xhtml else {}), **({} if paged else {"base_css": VIEWPORT})}}
            self.call_api(body, pdf)
            return
        if xhtml:  # an HTML parser would leave <div/> open: the rest of the document would nest in it
            text = text.replace("<![CDATA[", "").replace("]]>", "")
            text = SELF_CLOSING.sub(
                lambda m: m.group(0) if m.group(1).lower() in VOID else f"<{m.group(1)}{m.group(2)}></{m.group(1)}>", text)
        text = resolver.document(text, here)
        style = None if paged else self.viewport
        engine = self.args.engine
        if engine == "command" and "{style}" not in self.args.command and style:
            # After the doctype, so that the document stays in standards mode.
            m = DOCTYPE.match(text)
            at = m.end() if m else (3 if text.startswith("\xef\xbb\xbf") else 0)
            text = text[:at] + "<style>" + VIEWPORT + "</style>" + text[at:]
        html = os.path.join(self.work, uuid.uuid4().hex + ".html")
        with open(html, "wb") as f:
            f.write(text.encode("latin-1"))
        try:
            if engine == "prince":
                run_command([self.args.prince, *(["--style", style] if style else []), html, "-o", pdf])
            elif engine == "weasyprint":
                run_command([self.args.weasyprint, *(["-s", style] if style else []), html, pdf])
            else:
                args = shlex.split(self.args.command)
                if not style and "{style}" in args:  # no viewport page: drop {style} and the option that introduces it
                    i = args.index("{style}")
                    del args[i - 1 if i and args[i - 1].startswith("-") else i:i + 1]
                cmd = [a.replace("{input}", html).replace("{output}", pdf).replace("{style}", style or "") for a in args]
                run_command(cmd)
        finally:
            os.unlink(html)

    def call_api(self, body, pdf):
        data = json.dumps(body).encode()
        for attempt in range(6):
            req = urllib.request.Request(API, data=data, method="POST", headers={
                "Authorization": "Bearer " + self.key, "Content-Type": "application/json", "Accept": "application/pdf"})
            try:
                with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
                    mark = r.headers.get("X-ReflowPDF-Mark", "none")
                    if mark != "none":
                        raise RuntimeError(f"the API marked this print ({mark}): the account's plan has no unmarked pages left")
                    with open(pdf, "wb") as out:
                        shutil.copyfileobj(r, out)
                return
            except urllib.error.HTTPError as e:
                if e.code in (429, 503) and attempt < 5:
                    time.sleep(float(e.headers.get("Retry-After") or 2 ** attempt))
                    continue
                raise RuntimeError(f"API {e.code}: {e.read().decode(errors='replace')[:200]}")

    def pages(self, path, paged):
        """Print one file and rasterize it: the paths of its page images, in order."""
        tag = uuid.uuid4().hex
        pdf = os.path.join(self.work, tag + ".pdf")
        prefix = os.path.join(self.work, tag)
        try:
            self.print_pdf(path, pdf, paged)
            run_command(["pdftoppm", "-r", "96", *([] if paged else ["-f", "1", "-l", "1"]), pdf, prefix])
        finally:
            if os.path.exists(pdf):
                os.unlink(pdf)
        out = sorted(os.path.join(self.work, n) for n in os.listdir(self.work)
                     if n.startswith(tag + "-") and n.endswith(".ppm"))
        if not out:
            raise RuntimeError("no pages")
        return out

    def compare(self, test_pages, ref_pages, test_file, ref_file, maps=None):
        # --ref and --root let the comparator pick a tolerance the test keys to this reference.
        cmd = [self.args.compare, *test_pages, "--", *ref_pages,
               "--test=" + test_file, "--ref=" + ref_file, "--root=" + self.root]
        if maps:
            cmd.append("--map=" + maps)
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT)
        if r.returncode not in (0, 1):
            raise RuntimeError("compare: " + (r.stderr or r.stdout).strip()[:200])
        f = dict(kv.split("=", 1) for kv in r.stdout.split())
        return int(f["hard"]), int(f["maxd"]), r.returncode == 0

    def map_name(self, case, i):
        name = case["test"].replace("/", "__") + (f"__ref{i + 1}" if len(case["match"]) > 1 else "")
        os.makedirs(os.path.join(self.args.out, "maps"), exist_ok=True)
        return os.path.join(self.args.out, "maps", name)

    def judge(self, case):
        result = {"test": case["test"], "status": "skip", "reason": case["skip"] or "", "hard": 0, "maxd": 0}
        if case["skip"]:
            return result
        drop = lambda ps: [os.unlink(p) for p in ps]
        try:
            test = self.pages(case["path"], case["paged"])
            try:
                passed, best = not case["match"], None
                for i, ref in enumerate(case["match"]):
                    pages = self.pages(ref, case["paged"])
                    try:
                        hard, maxd, ok = self.compare(test, pages, case["path"], ref)
                        if not ok and self.args.maps:
                            self.compare(test, pages, case["path"], ref, maps=self.map_name(case, i))
                    finally:
                        drop(pages)
                    if best is None or ok or hard < best[0]:
                        best = (hard, maxd)
                    if ok:
                        passed = True
                        break
                distinct = True
                for ref in case["mismatch"]:
                    pages = self.pages(ref, case["paged"])
                    try:
                        # The same as the reference: no difference beyond forgiven anti-aliasing (hard=0).
                        distinct = self.compare(test, pages, case["path"], ref)[0] != 0
                    finally:
                        drop(pages)
                    if not distinct:
                        break
            finally:
                drop(test)
            hard, maxd = best or (0, 0)
            reason = "" if distinct else "renders the same as its mismatch reference"
            if hard == 2 ** 64 - 1:
                reason = reason or "page count or page size differs"
            result.update(status="pass" if passed and distinct else "fail", reason=reason, hard=hard, maxd=maxd)
        except Exception as e:  # one test's failure must not stop the run
            result.update(status="error", reason=str(e)[:200])
        return result


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("paths", nargs="+", help="test files or directories")
    ap.add_argument("--engine", required=True, choices=["prince", "weasyprint", "reflowpdf", "command"])
    ap.add_argument("--command", help="for --engine command: a command line with {input} and {output}; {style}, if present, is a style sheet that sets the viewport page of tests without @page (an option directly before it, such as --style, is dropped with it for tests with @page)")
    ap.add_argument("--prince", default="prince", help="the prince executable")
    ap.add_argument("--weasyprint", default="weasyprint", help="the weasyprint executable")
    ap.add_argument("--compare", default=os.environ.get("REFLOWPDF_COMPARE", os.path.join(
        HERE, "compare", "target", "release", "reflowpdf-compare")), help="the reflowpdf-compare binary")
    ap.add_argument("--root", default=HERE, help='where URLs starting with "/" resolve (default: this directory)')
    ap.add_argument("--out", default="out", help="where results.jsonl goes (default: out)")
    ap.add_argument("--maps", action="store_true", help="write a difference map (PNG) for every failing rel=match pair")
    ap.add_argument("--inline", action="store_true",
                    help="send local images, fonts and style sheets inside the document as data: URLs (always on for --engine reflowpdf)")
    ap.add_argument("--workers", type=int, default=os.cpu_count() or 4)
    args = ap.parse_args()
    if args.engine == "command" and not args.command:
        ap.error("--engine command needs --command")
    if not (os.path.isfile(args.compare) and os.access(args.compare, os.X_OK)):
        ap.error(f"no compare binary at {args.compare}: cargo build --release in compare/, or pass --compare")
    if not shutil.which("pdftoppm"):
        ap.error("pdftoppm is not installed (poppler-utils)")
    args.out = os.path.abspath(args.out)
    os.makedirs(args.out, exist_ok=True)

    runner = Runner(args)
    cases = discover(args.paths, runner.root)
    print(f"{len(cases)} reftests", file=sys.stderr)
    pool = ThreadPoolExecutor(args.workers)
    try:
        results = sorted(pool.map(runner.judge, cases), key=lambda r: r["test"])
        pool.shutdown()
    except KeyboardInterrupt:
        pool.shutdown(wait=False, cancel_futures=True)
        print("stopped", file=sys.stderr)
        sys.exit(130)
    finally:
        shutil.rmtree(runner.work, ignore_errors=True)

    with open(os.path.join(args.out, "results.jsonl"), "w") as f:
        for r in results:
            f.write(json.dumps(r) + "\n")
    by_dir = defaultdict(Counter)
    for r in results:
        by_dir[os.path.dirname(r["test"])][r["status"]] += 1
    total = Counter()
    print(f"{'directory':<32} {'pass':>5} {'fail':>5} {'error':>5} {'skip':>5}")
    for d in sorted(by_dir):
        c = by_dir[d]
        total.update(c)
        print(f"{d:<32} {c['pass']:>5} {c['fail']:>5} {c['error']:>5} {c['skip']:>5}")
    print(f"{'total':<32} {total['pass']:>5} {total['fail']:>5} {total['error']:>5} {total['skip']:>5}")


if __name__ == "__main__":
    main()
