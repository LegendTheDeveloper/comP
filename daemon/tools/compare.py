#!/usr/bin/env python3
"""Compare two replay runs (see replay.py) and print the quality metrics.

WHY these metrics: the complaint about run_pipeline was "too much context",
and the transcripts showed the cause was precision, not bytes. So the table
measures what the agent actually suffers from: pivots that are asset-store or
package code (vendor share), pivots that have no symbols at all (.meta,
prefabs, doc XML), whether the files it later edited were among the pivots
(recall against a ground-truth file from mine_transcripts.py), and only then
size and latency.

Vendor classification for runs whose daemon did not emit a `vendor` flag comes
from --vendor-prefixes: a text file, one path prefix or glob per line, matched
against the repo-qualified pivot path. A run that carries `vendor: true` flags
is also counted with them (column "vendor flag").

Usage:
  compare.py baseline.json candidate.json [--groundtruth gt.json]
             [--vendor-prefixes vendor-prefixes.txt] [--diff N]
"""

import argparse
import fnmatch
import json
import statistics
from collections import Counter

DATA_EXT = {
    "meta", "prefab", "unity", "asset", "mat", "anim", "controller", "mixer", "lighting",
    "rendertexture", "terrainlayer", "physicmaterial", "cubemap", "preset", "spriteatlas",
    "guiskin", "flare", "signal", "playable", "overridecontroller", "xml", "json", "txt",
    "pdf", "docx", "pptx", "xlsx", "csv", "yaml", "yml", "jsonl", "_",
}


def ext_of(path):
    name = path.rsplit("/", 1)[-1]
    return name.rsplit(".", 1)[-1].lower() if "." in name else ""


def load_prefixes(path):
    if not path:
        return []
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            s = line.strip()
            if s and not s.startswith("#"):
                out.append(s.replace("\\", "/"))
    return out


def is_vendor_path(path, prefixes):
    p = path.replace("\\", "/")
    for pre in prefixes:
        if any(ch in pre for ch in "*?["):
            if fnmatch.fnmatch(p, pre) or fnmatch.fnmatch(p, pre.rstrip("/") + "/*"):
                return True
        elif p.startswith(pre) or ("/" + pre) in p:
            return True
    return False


def summarize(run, prefixes, groundtruth):
    results = run["results"]
    pivots_per = []
    vendor_by_path = 0
    vendor_flag = 0
    data_pivots = 0
    doc_pivots = 0
    total_pivots = 0
    conf = Counter()
    weak = 0
    bytes_ = []
    ms = []
    top1_vendor = 0
    with_matched_symbols = 0
    for r in results:
        res = r["result"] or {}
        pivots = res.get("pivot_files", [])
        pivots_per.append(len(pivots))
        bytes_.append(r["bytes"])
        ms.append(r["ms"])
        conf[res.get("confidence")] += 1
        weak += 1 if res.get("weak_results") else 0
        for i, p in enumerate(pivots):
            total_pivots += 1
            path = p["path"]
            v = is_vendor_path(path, prefixes)
            vendor_by_path += 1 if v else 0
            vendor_flag += 1 if p.get("vendor") else 0
            e = ext_of(path)
            if e in DATA_EXT:
                data_pivots += 1
            if e == "md":
                doc_pivots += 1
            if i == 0 and v:
                top1_vendor += 1
            if p.get("matched_symbols"):
                with_matched_symbols += 1

    recall = None
    if groundtruth:
        by_query = {}
        for r in results:
            by_query.setdefault(r["query"], [p["path"] for p in (r["result"] or {}).get("pivot_files", [])])
        calls = 0
        calls_hit = 0
        files = 0
        files_hit = 0
        misses = []
        for e in groundtruth["entries"]:
            pivots = by_query.get(e["query"])
            if pivots is None:
                continue
            calls += 1
            hits = [f for f in e["edited"] if f in pivots]
            files += len(e["edited"])
            files_hit += len(hits)
            if hits:
                calls_hit += 1
            else:
                misses.append((e["query"], e["edited"][:3]))
        recall = {"calls": calls, "calls_hit": calls_hit, "files": files, "files_hit": files_hit, "misses": misses}

    n = len(results) or 1
    return {
        "n": len(results),
        "version": run.get("daemon_version"),
        "index_files": (run.get("index") or {}).get("total_files"),
        "index_nodes": (run.get("index") or {}).get("total_nodes"),
        "pivots_mean": statistics.mean(pivots_per) if pivots_per else 0,
        "total_pivots": total_pivots,
        "vendor_by_path": vendor_by_path,
        "vendor_flag": vendor_flag,
        "top1_vendor": top1_vendor,
        "data_pivots": data_pivots,
        "doc_pivots": doc_pivots,
        "with_matched_symbols": with_matched_symbols,
        "conf": conf,
        "weak": weak,
        "bytes_mean": statistics.mean(bytes_) if bytes_ else 0,
        "bytes_max": max(bytes_) if bytes_ else 0,
        "ms_mean": statistics.mean(ms) if ms else 0,
        "recall": recall,
    }


def pct(a, b):
    return "%.0f%%" % (100.0 * a / b) if b else "-"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("baseline")
    ap.add_argument("candidate")
    ap.add_argument("--groundtruth")
    ap.add_argument("--vendor-prefixes")
    ap.add_argument("--diff", type=int, default=25, help="how many top-1 changes to list")
    args = ap.parse_args()

    base = json.load(open(args.baseline, encoding="utf-8"))
    cand = json.load(open(args.candidate, encoding="utf-8"))
    gt = json.load(open(args.groundtruth, encoding="utf-8")) if args.groundtruth else None
    prefixes = load_prefixes(args.vendor_prefixes)

    b = summarize(base, prefixes, gt)
    c = summarize(cand, prefixes, gt)

    rows = [
        ("daemon version", b["version"], c["version"]),
        ("indexed files", b["index_files"], c["index_files"]),
        ("indexed symbols", b["index_nodes"], c["index_nodes"]),
        ("queries", b["n"], c["n"]),
        ("pivots per query (mean)", "%.1f" % b["pivots_mean"], "%.1f" % c["pivots_mean"]),
        ("vendor pivots (by path)", "%d (%s)" % (b["vendor_by_path"], pct(b["vendor_by_path"], b["total_pivots"])), "%d (%s)" % (c["vendor_by_path"], pct(c["vendor_by_path"], c["total_pivots"]))),
        ("vendor pivots (flagged by daemon)", b["vendor_flag"], c["vendor_flag"]),
        ("queries whose top-1 is vendor", "%d (%s)" % (b["top1_vendor"], pct(b["top1_vendor"], b["n"])), "%d (%s)" % (c["top1_vendor"], pct(c["top1_vendor"], c["n"]))),
        ("symbol-less pivots (.meta, prefab, xml...)", "%d (%s)" % (b["data_pivots"], pct(b["data_pivots"], b["total_pivots"])), "%d (%s)" % (c["data_pivots"], pct(c["data_pivots"], c["total_pivots"]))),
        ("markdown pivots", b["doc_pivots"], c["doc_pivots"]),
        ("pivots carrying matched_symbols", b["with_matched_symbols"], c["with_matched_symbols"]),
        ("confidence high / medium / low", "%d / %d / %d" % (b["conf"]["high"], b["conf"]["medium"], b["conf"]["low"]), "%d / %d / %d" % (c["conf"]["high"], c["conf"]["medium"], c["conf"]["low"])),
        ("weak_results", b["weak"], c["weak"]),
        ("response bytes (mean / max)", "%d / %d" % (b["bytes_mean"], b["bytes_max"]), "%d / %d" % (c["bytes_mean"], c["bytes_max"])),
        ("latency ms (mean)", "%d" % b["ms_mean"], "%d" % c["ms_mean"]),
    ]
    if b["recall"] and c["recall"]:
        rb, rc = b["recall"], c["recall"]
        rows.append(("ground-truth calls with >=1 edited file among pivots", "%d / %d (%s)" % (rb["calls_hit"], rb["calls"], pct(rb["calls_hit"], rb["calls"])), "%d / %d (%s)" % (rc["calls_hit"], rc["calls"], pct(rc["calls_hit"], rc["calls"]))))
        rows.append(("edited files found among pivots", "%d / %d (%s)" % (rb["files_hit"], rb["files"], pct(rb["files_hit"], rb["files"])), "%d / %d (%s)" % (rc["files_hit"], rc["files"], pct(rc["files_hit"], rc["files"]))))

    print("| Metric | Baseline | Candidate |")
    print("| --- | --- | --- |")
    for name, x, y in rows:
        print("| %s | %s | %s |" % (name, x, y))

    # Per-query top-1 changes, for eyeballing what moved.
    base_by = {r["query"]: r for r in base["results"]}
    changed = []
    for r in cand["results"]:
        br = base_by.get(r["query"])
        if not br:
            continue
        bp = [p["path"] for p in (br["result"] or {}).get("pivot_files", [])]
        cp = [p["path"] for p in (r["result"] or {}).get("pivot_files", [])]
        if (bp[:1] or [None]) != (cp[:1] or [None]):
            changed.append((r["query"], bp[:1], cp[:1]))
    print("\ntop-1 changed in %d of %d queries" % (len(changed), len(cand["results"])))
    for q, bp, cp in changed[: args.diff]:
        print("- %s\n    was: %s\n    now: %s" % (q[:90], bp[0] if bp else "-", cp[0] if cp else "-"))

    if c["recall"] and c["recall"]["misses"]:
        print("\nstill missed (candidate), first %d:" % min(15, len(c["recall"]["misses"])))
        for q, ed in c["recall"]["misses"][:15]:
            print("- %s -> %s" % (q[:80], [e.rsplit('/', 1)[-1] for e in ed]))


if __name__ == "__main__":
    main()
