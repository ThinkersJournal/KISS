# SPDX-License-Identifier: MIT OR Apache-2.0
"""Corpus sha256 pins (ROADMAP R5.21).

`conformance/corpus/PROVENANCE.md`, `REUSE.toml` and `tools/kiss_spdx.py` say the corpus JSON is
sha256-pinned. The pins live in ONE file, `conformance/corpus/PINS.sha256` (`<sha256>  <name>`,
sorted by name, LF). This tool recomputes them:

  (default) / --check   every `conformance/corpus/*.json` must be listed, every listed file must
                        exist, and every recomputed hash must equal its pin. Any difference exits 1
                        and names the file and the kind of difference (changed / unpinned / missing).
  --write               rewrite PINS.sha256 from the files on disk. Run it after a deliberate
                        regeneration (`emit_*` / `kiss_mint`), and review the diff: a pin that moves
                        is the corpus moving.

A hash is taken over the file's bytes with CRLF folded to LF, so a checkout with `core.autocrlf`
cannot report a difference that is not one (`.gitattributes` forces LF on the corpus regardless).
The pin file is enumerated from the directory listing of `conformance/corpus` only, never a walk.
"""
import argparse
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
CORPUS = os.path.join(ROOT, "conformance", "corpus")
PINS = "PINS.sha256"


def digest(path):
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read().replace(b"\r\n", b"\n")).hexdigest()


def corpus_files(corpus):
    return sorted(n for n in os.listdir(corpus) if n.endswith(".json"))


def read_pins(corpus):
    pins = {}
    with open(os.path.join(corpus, PINS), "rb") as fh:
        for raw in fh.read().decode("utf-8").splitlines():
            if not raw.strip():
                continue
            h, sep, name = raw.partition("  ")
            if not sep or len(h) != 64 or any(c not in "0123456789abcdef" for c in h):
                raise ValueError("malformed pin line: %r" % raw)
            if name in pins:
                raise ValueError("duplicate pin for %s" % name)
            pins[name] = h
    return pins


def check(corpus=CORPUS):
    """Return a list of (kind, name) differences; empty means CLEAN."""
    pins = read_pins(corpus)
    on_disk = corpus_files(corpus)
    out = []
    for name in on_disk:
        if name not in pins:
            out.append(("unpinned", name))
        elif digest(os.path.join(corpus, name)) != pins[name]:
            out.append(("changed", name))
    for name in sorted(pins):
        if name not in on_disk:
            out.append(("missing", name))
    return out


def write(corpus=CORPUS):
    lines = ["%s  %s\n" % (digest(os.path.join(corpus, n)), n) for n in corpus_files(corpus)]
    with open(os.path.join(corpus, PINS), "wb") as fh:
        fh.write("".join(lines).encode("utf-8"))
    return len(lines)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--write", action="store_true")
    a = ap.parse_args(argv)
    if a.write:
        print("wrote %d pins to conformance/corpus/%s" % (write(), PINS))
        return 0
    diffs = check()
    n = len(read_pins(CORPUS))
    for kind, name in diffs:
        print("PIN %s: %s" % (kind.upper(), name))
    print("corpus pins: %d pinned, %d on disk, %s" % (n, len(corpus_files(CORPUS)),
                                                      "DRIFT" if diffs else "CLEAN"))
    return 1 if diffs else 0


if __name__ == "__main__":
    sys.exit(main())
