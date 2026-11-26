#!/usr/bin/env python3
"""Build crates/triokey-engine/data/freq_{vi,en}.tsv from the token counts that fetch.py
writes under .corpus/counts/ (counts documents only, never the benchmark documents).

Only counts are stored, no text. Standard library only.
"""
import os, sys, unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SPECS = {  # lang: (min count, dataset line)
    "vi": (5, "HuggingFaceFW/fineweb-2, config vie_Latn, split train"),
    "en": (3, "HuggingFaceFW/fineweb, config sample-10BT, split train"),
}
TONES = {"̀", "́", "̃", "̉", "̣"}
MODS = {("a", "̂"), ("e", "̂"), ("o", "̂"), ("a", "̆"), ("o", "̛"), ("u", "̛")}
MAX_LEN = 24


def letter_ok(c):
    """ASCII letter or Vietnamese letter (lower case, NFC)."""
    if "a" <= c <= "z" or c == "đ":
        return True
    d = unicodedata.normalize("NFD", c)
    base, marks = d[0], d[1:]
    tones = [m for m in marks if m in TONES]
    mods = [m for m in marks if m not in TONES]
    return (base in "aeiouy" and marks != "" and len(tones) <= 1 and len(mods) <= 1
            and len(tones) + len(mods) == len(marks) and all((base, m) in MODS for m in mods))


def token_ok(t):
    return 0 < len(t) <= MAX_LEN and t == t.lower() and all(letter_ok(c) for c in t)


def read_counts(path):
    total = docs = None
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.startswith("# total tokens:"):
                total = int(line.split(":")[1])
            elif line.startswith("# documents:"):
                docs = int(line.split(":")[1])
            elif not line.startswith("#") and line.strip():
                w, k = line.rstrip("\n").split("\t")
                rows.append((w, int(k)))
    if total is None or docs is None:
        sys.exit(f"{path}: missing '# total tokens' or '# documents' header; rerun fetch.py")
    return total, docs, rows


def build(lang, counts_dir, out_dir):
    min_count, dataset = SPECS[lang]
    total, docs, rows = read_counts(os.path.join(counts_dir, f"{lang}_tokens.tsv"))
    keep = sorted(((w, k) for w, k in rows if k >= min_count and token_ok(w)), key=lambda r: (-r[1], r[0]))
    path = os.path.join(out_dir, f"freq_{lang}.tsv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(f"# Token frequencies: lower-cased letter-run token counts (token<TAB>count), count >= {min_count},\n"
                f"# most frequent first; tokens over {MAX_LEN} chars or with a char that is not an ASCII or\n"
                f"# Vietnamese letter are dropped. Built by tools/corpus/build_freq.py.\n"
                f"# total: {total}\n"
                f"# source: {dataset} (Hugging Face), licence ODC-By 1.0\n"
                f"# documents counted: {docs} (the benchmark documents are excluded)\n"
                f"# Only counts are stored, no text.\n")
        f.writelines(f"{w}\t{k}\n" for w, k in keep)
    print(f"{path}: total {total}, {len(keep)} rows, {os.path.getsize(path)} bytes")


def selftest():
    assert token_ok("đường") and token_ok("test") and token_ok("ừ")
    assert not token_ok("naïve") and not token_ok("ñ") and not token_ok("Test") and not token_ok("a1")
    assert not token_ok("a" * 25) and token_ok("a" * 24)


def main():
    selftest()
    counts_dir = os.path.join(ROOT, ".corpus", "counts")
    out_dir = os.path.join(ROOT, "crates", "triokey-engine", "data")
    for lang in SPECS:
        build(lang, counts_dir, out_dir)


if __name__ == "__main__":
    main()
