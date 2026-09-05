#!/usr/bin/env python3
"""One-time cleanup of a workspace's .comp/session-memory.json.

WHY: daemons before 0.9.7 recorded every raw hit of every search channel in
each call's `files` list (119 to 1 672 paths per call, most of them Unity
.meta twins), and rewrote the whole pretty-printed file on every search. The
file reached 3.2 MB on Lynium and 19.6 MB on GameLauncherCloud. 0.9.7 records
only the pivots it returned, but the history already written stays bloated
until it is rewritten once.

For each call, `files` becomes the pivots the daemon actually returned for
that query, read back from search_history.top_pivots in .comp/index.db
(matched by query text, closest timestamp). When no row exists the list is
filtered instead: skipped extensions and data files go, the first 20 stay.
Symbols are capped at 20. The result is written compactly, newest 300 calls
kept, with a .bak of the original beside it.

Usage: shrink_session_memory.py <workspace-root> [--keep 300] [--dry-run]
"""

import argparse
import json
import os
import shutil
import sqlite3
import sys

SKIP_EXT = {
    "meta", "prefab", "unity", "asset", "mat", "anim", "controller", "mixer", "lighting",
    "rendertexture", "terrainlayer", "physicmaterial", "cubemap", "preset", "spriteatlas",
    "guiskin", "flare", "signal", "playable", "overridecontroller", "xml", "json", "pdf",
    "docx", "pptx", "xlsx", "csv", "yaml", "yml", "jsonl", "_",
}


def ext_of(path):
    name = path.rsplit("/", 1)[-1]
    return name.rsplit(".", 1)[-1].lower() if "." in name else ""


def load_history(db_path):
    """query -> [(timestamp_seconds, [pivot paths])]"""
    by_query = {}
    if not os.path.exists(db_path):
        return by_query
    con = sqlite3.connect(db_path)
    con.execute("PRAGMA busy_timeout=5000")
    try:
        rows = con.execute(
            "SELECT query, timestamp, top_pivots FROM search_history WHERE tool='run_pipeline'"
        ).fetchall()
    except sqlite3.OperationalError:
        rows = []
    con.close()
    for query, ts, top in rows:
        try:
            pivots = [p["path"] for p in json.loads(top or "[]") if isinstance(p, dict) and "path" in p]
        except json.JSONDecodeError:
            pivots = []
        by_query.setdefault(query, []).append((ts or 0, pivots))
    return by_query


def shrink_call(call, history):
    query = call.get("query") or call.get("request") or ""
    files = call.get("files") or []
    ts = (call.get("timestamp") or 0) / 1000.0
    candidates = history.get(query)
    if candidates:
        _, pivots = min(candidates, key=lambda c: abs(c[0] - ts))
        new_files = pivots
        source = "history"
    else:
        new_files = [f for f in files if ext_of(f) not in SKIP_EXT][:20]
        source = "filtered"
    call["files"] = new_files
    if isinstance(call.get("symbols"), list):
        call["symbols"] = call["symbols"][:20]
    return source


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("workspace")
    ap.add_argument("--keep", type=int, default=300)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    comp = os.path.join(os.path.abspath(args.workspace), ".comp")
    memory_path = os.path.join(comp, "session-memory.json")
    if not os.path.exists(memory_path):
        print("no session-memory.json under", comp)
        return 1
    before = os.path.getsize(memory_path)
    with open(memory_path, encoding="utf-8") as f:
        memory = json.load(f)
    history = load_history(os.path.join(comp, "index.db"))

    sessions = memory.get("sessions", [])
    sources = {"history": 0, "filtered": 0}
    files_before = 0
    files_after = 0
    for s in sessions:
        for c in s.get("calls", []):
            files_before += len(c.get("files") or [])
            sources[shrink_call(c, history)] += 1
            files_after += len(c["files"])

    # Keep the newest calls; sessions are chronological, oldest first.
    total = sum(len(s.get("calls", [])) for s in sessions)
    excess = max(0, total - args.keep)
    for s in sessions:
        if excess == 0:
            break
        take = min(excess, len(s.get("calls", [])))
        s["calls"] = s["calls"][take:]
        excess -= take
    memory["sessions"] = [s for s in sessions if s.get("calls")]

    payload = json.dumps(memory, ensure_ascii=False, separators=(",", ":"))
    print(
        "%s: %d calls (%d from history, %d filtered), files %d -> %d, bytes %d -> %d"
        % (memory_path, total, sources["history"], sources["filtered"], files_before, files_after, before, len(payload.encode("utf-8")))
    )
    if args.dry_run:
        return 0
    shutil.copy2(memory_path, memory_path + ".bak")
    tmp = memory_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(payload)
    os.replace(tmp, memory_path)
    print("written; original kept as session-memory.json.bak")
    return 0


if __name__ == "__main__":
    sys.exit(main())
