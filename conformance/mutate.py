#!/usr/bin/env python3
"""Check that the gcpm tests can see the mistakes they guard against.

    python3 mutate.py --engine weasyprint
    python3 mutate.py --engine prince --prince /path/to/prince

Each entry of gcpm/mutations.json puts a wrong behaviour into a copy of a test ("target": "test"), or a wrong
rendering into a copy of its reference ("target": "reference"). Every copy should fail. The copies are made in a
temporary folder and run with run.py; any option after the first one is passed to run.py unchanged. A copy that
passes is a mistake the test cannot see on that engine, or one the engine already makes in the original test.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))


def find_test(root, name):
    for ext in (".html", ".tentative.html"):
        if os.path.isfile(os.path.join(root, "gcpm", name + ext)):
            return name + ext
    raise SystemExit(f"no test {name} in gcpm/")


def main():
    mutations = json.load(open(os.path.join(HERE, "gcpm", "mutations.json"), encoding="utf-8"))
    work = tempfile.mkdtemp(prefix="mutate-")
    try:
        for d in ("fonts", "gcpm"):
            shutil.copytree(os.path.join(HERE, d), os.path.join(work, d))
        made = {}
        for i, m in enumerate(mutations, 1):
            test = find_test(work, m["test"])
            path = os.path.join(work, "gcpm", test)
            src = open(path, encoding="utf-8").read()
            ref = re.search(r'<link rel="match" href="([^"]+)">', src).group(1)
            ref_path = os.path.join(os.path.dirname(path), ref)
            text = src if m["target"] == "test" else open(ref_path, encoding="utf-8").read()
            for old, new in m["edits"]:
                if old not in text:
                    raise SystemExit(f"mutation {i} ({m['test']}): text not found: {old!r}")
                text = text.replace(old, new)
            stem = f"{os.path.basename(m['test'])}--m{i:03d}"
            out = os.path.join(os.path.dirname(path), stem + ".html")
            if m["target"] == "test":
                open(out, "w", encoding="utf-8").write(text)
            else:
                open(os.path.join(os.path.dirname(path), stem + "-ref.html"), "w", encoding="utf-8").write(text)
                open(out, "w", encoding="utf-8").write(src.replace(f'href="{ref}"', f'href="{stem}-ref.html"'))
            made[os.path.relpath(out, work)] = m
        result = os.path.join(work, "out")
        subprocess.run([sys.executable, os.path.join(HERE, "run.py"), "--root", work, "--out", result,
                        *sys.argv[1:], *[os.path.join(work, p) for p in made]], check=True,
                       stdout=subprocess.DEVNULL)
        rows = [json.loads(line) for line in open(os.path.join(result, "results.jsonl"))]
        missed = [r for r in rows if r["status"] != "fail"]
        print(f"{len(rows) - len(missed)} of {len(rows)} mutations fail")
        for r in missed:
            m = made[r["test"]]
            print(f"  {r['status']}: {m['test']} ({m['target']}): {m['note']}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
