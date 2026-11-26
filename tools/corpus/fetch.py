#!/usr/bin/env python3
"""Sample FineWeb-2 (vie_Latn) and FineWeb (sample-10BT) through the Hugging Face
datasets-server rows API and derive benchmark line files and token counts.

Both datasets are ODC-By. Output goes under .corpus/ (gitignored); never commit it.
Standard library only. Resumable: rerun to continue an interrupted fetch.
"""
import argparse, collections, hashlib, json, os, re, sys, time, unicodedata, urllib.error, urllib.parse, urllib.request

API = "https://datasets-server.huggingface.co/rows"
UA = "corpus-sampler/1.0"
PAGE = 100
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SOURCES = {
    "vi": ("HuggingFaceFW/fineweb-2", "vie_Latn"),
    "en": ("HuggingFaceFW/fineweb", "sample-10BT"),
}
TONES = {"̀", "́", "̃", "̉", "̣"}
MODS = {("a", "̂"), ("e", "̂"), ("o", "̂"), ("a", "̆"), ("o", "̛"), ("u", "̛")}


def is_vi_letter(c):
    """Non-ASCII Vietnamese letter (precomposed, NFC)."""
    if c in "đĐ":
        return True
    d = unicodedata.normalize("NFD", c)
    base, marks = d[0].lower(), d[1:]
    tones = [m for m in marks if m in TONES]
    mods = [m for m in marks if m not in TONES]
    return (base in "aeiouy" and 0 < len(marks) == len(tones) + len(mods) and len(tones) <= 1
            and len(mods) <= 1 and all((base, m) in MODS for m in mods))


VI_LETTERS = "".join(chr(i) for i in range(0xC0, 0x1F00) if is_vi_letter(chr(i)))
TOKEN_RE = re.compile("[A-Za-z" + VI_LETTERS + "]+")
VI_SET = set(VI_LETTERS)


def get(params, tries=8):
    url = API + "?" + urllib.parse.urlencode(params)
    for k in range(tries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=60) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            if e.code != 429 and e.code < 500:
                raise
            wait = float(e.headers.get("Retry-After") or 0) or min(2 ** (k + 1), 120)
            print(f"  HTTP {e.code}, retry in {wait:.0f}s", file=sys.stderr)
        except (urllib.error.URLError, TimeoutError, ConnectionError, json.JSONDecodeError) as e:
            wait = min(2 ** (k + 1), 120)
            print(f"  {e!r}, retry in {wait:.0f}s", file=sys.stderr)
        time.sleep(wait)
    raise RuntimeError(f"giving up on {url}")


def fetch(lang, n, out):
    dataset, config = SOURCES[lang]
    base = {"dataset": dataset, "config": config, "split": "train"}
    path = os.path.join(out, "raw", f"{lang}.jsonl")
    done_path = path + ".offsets"  # completed page offsets, for resume
    done = set()
    if os.path.exists(done_path):
        done = {int(x) for x in open(done_path, encoding="utf-8").read().split()}
    total = get({**base, "offset": 0, "length": 1})["num_rows_total"]
    pages = -(-n // PAGE)
    offsets = [i * (total - PAGE) // max(pages - 1, 1) for i in range(pages)]
    # pages kept from a run with another n count toward n: drop the nearest planned page for each
    for o in sorted(done - set(offsets)):
        todo = [x for x in offsets if x not in done]
        if todo:
            offsets.remove(min(todo, key=lambda x: abs(x - o)))
    print(f"{lang}: {total} rows in split, {pages} pages, {len(done)} already on disk")
    with open(path, "a", encoding="utf-8") as f, open(done_path, "a", encoding="utf-8") as d:
        for i, o in enumerate(offsets):
            if o in done:
                continue
            time.sleep(1.0)
            rows = get({**base, "offset": o, "length": PAGE})["rows"]
            for r in rows:
                row = r["row"]
                text = unicodedata.normalize("NFC", row["text"])
                f.write(json.dumps({"id": row["id"], "text": text}, ensure_ascii=False) + "\n")
            f.flush()
            d.write(f"{o}\n")
            d.flush()
            print(f"  {lang} page {i + 1}/{pages} offset {o}: {len(rows)} rows")


def group(doc_id):
    """Document-level split, fixed by id: one in five documents is benchmark text, never counted."""
    return "bench" if int(hashlib.sha1(doc_id.encode("utf-8")).hexdigest(), 16) % 5 == 0 else "counts"


def docs(out, lang, grp):
    seen = set()
    with open(os.path.join(out, "raw", f"{lang}.jsonl"), encoding="utf-8") as f:
        for line in f:
            try:
                doc = json.loads(line)
            except json.JSONDecodeError:  # truncated last line from an interrupted run
                continue
            if doc["id"] not in seen:  # resume after a crash mid-page can repeat a page
                seen.add(doc["id"])
                if group(doc["id"]) == grp:
                    yield doc["text"]


def good_line(line, need_vi):
    toks = line.split()
    if not 5 <= len(toks) <= 40:
        return False
    chars = "".join(toks)
    if not all(" " <= c <= "~" or c in VI_SET for c in chars):
        return False
    letters = sum(c.isalpha() for c in chars)
    if letters < 0.7 * len(chars):
        return False
    return not need_vi or any(c in VI_SET for c in chars)


def unaccent(s):
    s = unicodedata.normalize("NFD", s.lower().replace("đ", "d"))
    return "".join(c for c in s if not unicodedata.combining(c))


def word_list(rel):
    with open(os.path.join(ROOT, rel), encoding="utf-8") as f:
        return {w.strip() for w in f if w.strip() and not w.startswith("#")}


def bench(out, lang, cap, mixed_cap=0):
    need_vi = lang == "vi"
    lines, seen = [], set()
    mixed, mixed_seen = [], set()
    if mixed_cap:
        vi_plain = {unaccent(s) for s in word_list("crates/triokey-engine/tests/data/vi_attested.txt")}
        en = {w for w in word_list("crates/triokey-engine/data/en_words.txt") if len(w) >= 3} - vi_plain
        ascii_word = re.compile("^[a-z]{3,}$")
    for text in docs(out, lang, "bench"):
        per_doc = 0
        for raw in text.split("\n"):
            line = " ".join(raw.split())
            if not good_line(line, need_vi):
                continue
            if per_doc < 5 and len(lines) < cap and line not in seen:
                seen.add(line)
                lines.append(line)
                per_doc += 1
            if mixed_cap and len(mixed) < mixed_cap and line not in mixed_seen:
                if any(ascii_word.match(t) and t in en for t in (t.strip(".,;:!?\"'()[]") for t in line.split())):
                    mixed_seen.add(line)
                    mixed.append(line)
    os.makedirs(os.path.join(out, "bench"), exist_ok=True)
    write(os.path.join(out, "bench", f"web_{lang}.txt"), lines)
    if mixed_cap:
        write(os.path.join(out, "bench", "web_vi_mixed.txt"), mixed)


def counts(out, lang):
    c = collections.Counter()
    n = 0
    for text in docs(out, lang, "counts"):
        c.update(t.lower() for t in TOKEN_RE.findall(text))
        n += 1
    os.makedirs(os.path.join(out, "counts"), exist_ok=True)
    with open(os.path.join(out, "counts", f"{lang}_tokens.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write(f"# total tokens: {sum(c.values())}\n# documents: {n}\n")
        for w, k in c.most_common():
            f.write(f"{w}\t{k}\n")


def check_split(out, lang):
    ids = {"bench": set(), "counts": set()}
    with open(os.path.join(out, "raw", f"{lang}.jsonl"), encoding="utf-8") as f:
        for line in f:
            try:
                i = json.loads(line)["id"]
            except json.JSONDecodeError:
                continue
            ids[group(i)].add(i)
    assert not ids["bench"] & ids["counts"]
    print(f"{lang}: {len(ids['bench'])} bench documents, {len(ids['counts'])} counts documents, none in both")


def write(path, lines):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("".join(l + "\n" for l in lines))
    print(f"{path}: {len(lines)} lines, sha256 {hashlib.sha256(open(path, 'rb').read()).hexdigest()[:12]}")


def selftest():
    assert is_vi_letter("ệ") and is_vi_letter("đ") and is_vi_letter("Ữ") and is_vi_letter("é")
    assert not is_vi_letter("ñ") and not is_vi_letter("ü") and not is_vi_letter("ŷ") and not is_vi_letter("ç")
    assert len(VI_LETTERS) == 134, len(VI_LETTERS)  # 67 lowercase (12 vowels x 5 tones + 6 marked bare + đ) x 2 cases
    assert good_line("Tôi đang viết code cho dự án", True)
    assert not good_line("Tôi đang viết code", True)  # 4 tokens
    assert not good_line("this is plain english text here", True)
    assert good_line("this is plain english text here", False)
    assert not good_line("こんにちは is not latin text at all", False)
    assert not good_line("1234 5678 9012 3456 7890 abc", False)  # under 70% letters
    assert unaccent("Đường") == "duong"
    assert group("<urn:uuid:0>") in ("bench", "counts")
    assert TOKEN_RE.findall("Xin chào, world-wide 2024!") == ["Xin", "chào", "world", "wide"]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--vi-rows", type=int, default=16000)
    ap.add_argument("--en-rows", type=int, default=10000)
    ap.add_argument("--out", default=os.path.join(ROOT, ".corpus"))
    a = ap.parse_args()
    selftest()
    os.makedirs(os.path.join(a.out, "raw"), exist_ok=True)
    fetch("vi", a.vi_rows, a.out)
    fetch("en", a.en_rows, a.out)
    check_split(a.out, "vi")
    check_split(a.out, "en")
    bench(a.out, "vi", 30000, mixed_cap=15000)
    bench(a.out, "en", 15000)
    counts(a.out, "vi")
    counts(a.out, "en")


if __name__ == "__main__":
    main()
