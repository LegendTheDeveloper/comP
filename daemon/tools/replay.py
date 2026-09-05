#!/usr/bin/env python3
"""Replay every recorded run_pipeline query against a daemon binary.

WHY: search quality can only be compared across daemon versions by asking both
binaries the same questions. The questions are already recorded: the daemon
writes every run_pipeline call to `search_history` in `.comp/index.db`. This
script reads them back, drives a daemon over stdio exactly as an MCP client
would, and stores the full responses so `compare.py` can diff two runs.

The daemon records the replayed calls too (search_history rows and
session-memory.json entries). Those are restored afterwards so a replay never
pollutes the logs it reads from: session-memory.json is copied aside and put
back, and search_history rows inserted during the run are deleted.

Usage:
  replay.py --exe C:/Users/me/.comp-bin/comp-daemon-win.exe \
            --workspace C:/path/to/repo --out baseline/lynium-0.9.6.json
  Options: --limit N (first N distinct queries), --params '{"max_pivots": 12}'
           (extra run_pipeline params for every query), --queries FILE (one
           query per line instead of search_history), --no-restore.
"""

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import time


def jsonrpc(proc, req_id, method, params=None):
    """Send one request and block until its response line arrives.

    Log lines never reach stdout (env_logger writes to stderr), so every stdout
    line is a JSON-RPC message; responses to other ids are not expected because
    requests are issued strictly one at a time.
    """
    req = {"jsonrpc": "2.0", "id": req_id, "method": method, "params": params or {}}
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    while True:
        line = proc.stdout.readline()
        if line == "":
            raise RuntimeError("daemon closed stdout (method=%s)" % method)
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("id") == req_id:
            if "error" in msg:
                raise RuntimeError("daemon error for %s: %s" % (method, msg["error"]))
            return msg.get("result")


def load_queries(db_path, limit=None):
    con = sqlite3.connect(db_path)
    con.execute("PRAGMA busy_timeout=5000")
    rows = con.execute(
        "SELECT query FROM search_history WHERE tool='run_pipeline' ORDER BY id"
    ).fetchall()
    con.close()
    seen = set()
    queries = []
    for (q,) in rows:
        if q in seen:
            continue
        seen.add(q)
        queries.append(q)
    return queries[:limit] if limit else queries


def wait_for_index(proc, next_id, timeout_s=1800):
    """Poll getStats until background indexing (and the TF-IDF build that ends
    it) has finished. Querying earlier would compare a daemon with an empty
    TF-IDF index against one with a full one."""
    started = time.time()
    last_log = 0
    while True:
        stats = jsonrpc(proc, next_id, "getStats")
        next_id += 1
        indexing = (stats or {}).get("indexing", {})
        if not indexing.get("is_indexing", True):
            return stats, next_id
        if time.time() - last_log > 15:
            print(
                "  indexing... repo=%s files=%s" % (indexing.get("current_repo"), stats.get("total_files")),
                file=sys.stderr,
            )
            last_log = time.time()
        if time.time() - started > timeout_s:
            raise RuntimeError("indexing did not finish within %ds" % timeout_s)
        time.sleep(2)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--exe", required=True)
    ap.add_argument("--workspace", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--limit", type=int)
    ap.add_argument("--params", default="{}", help="JSON object merged into every run_pipeline call")
    ap.add_argument("--queries", help="text file with one query per line (overrides search_history)")
    ap.add_argument("--no-restore", action="store_true", help="leave the replayed calls in the daemon logs")
    ap.add_argument("--stderr", help="file to capture daemon stderr (default: discard)")
    args = ap.parse_args()

    workspace = os.path.abspath(args.workspace)
    comp_dir = os.path.join(workspace, ".comp")
    db_path = os.path.join(comp_dir, "index.db")
    memory_path = os.path.join(comp_dir, "session-memory.json")
    extra_params = json.loads(args.params)

    if args.queries:
        with open(args.queries, encoding="utf-8") as f:
            queries = [l.strip() for l in f if l.strip()]
        if args.limit:
            queries = queries[: args.limit]
    else:
        queries = load_queries(db_path, args.limit)
    print("replaying %d queries against %s" % (len(queries), args.exe), file=sys.stderr)

    # Snapshot what the daemon is about to append to.
    memory_backup = None
    if os.path.exists(memory_path):
        memory_backup = memory_path + ".replay-bak"
        shutil.copy2(memory_path, memory_backup)
    con = sqlite3.connect(db_path)
    con.execute("PRAGMA busy_timeout=5000")
    max_id_before = con.execute("SELECT COALESCE(MAX(id), 0) FROM search_history").fetchone()[0]
    con.close()
    start_ts = int(time.time())

    env = dict(os.environ)
    env["COMP_WORKSPACE_ROOT"] = workspace
    env.setdefault("RUST_LOG", "warn")
    stderr_target = open(args.stderr, "w", encoding="utf-8") if args.stderr else subprocess.DEVNULL
    proc = subprocess.Popen(
        [args.exe],
        cwd=workspace,
        env=env,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=stderr_target,
        text=True,
        encoding="utf-8",
        bufsize=1,
    )

    results = []
    daemon_version = None
    try:
        init = jsonrpc(proc, 1, "initialize", {"protocolVersion": "2024-11-05", "capabilities": {}})
        daemon_version = (init or {}).get("serverInfo", {}).get("version")
        stats, next_id = wait_for_index(proc, 2)
        print(
            "index ready: version=%s files=%s nodes=%s edges=%s"
            % (daemon_version, stats.get("total_files"), stats.get("total_nodes"), stats.get("total_edges")),
            file=sys.stderr,
        )
        for i, q in enumerate(queries):
            params = dict(extra_params)
            params["task"] = q
            t0 = time.perf_counter()
            result = jsonrpc(proc, next_id, "run_pipeline", params)
            next_id += 1
            ms = int((time.perf_counter() - t0) * 1000)
            payload = json.dumps(result, separators=(",", ":"), ensure_ascii=False)
            results.append({"query": q, "params": params, "ms": ms, "bytes": len(payload.encode("utf-8")), "result": result})
            if (i + 1) % 10 == 0 or i + 1 == len(queries):
                print("  %d/%d" % (i + 1, len(queries)), file=sys.stderr)
        final_stats = jsonrpc(proc, next_id, "getStats")
    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        try:
            proc.wait(timeout=20)
        except Exception:
            proc.kill()
        if args.stderr:
            stderr_target.close()
        if not args.no_restore:
            if memory_backup:
                shutil.move(memory_backup, memory_path)
            con = sqlite3.connect(db_path)
            con.execute("PRAGMA busy_timeout=5000")
            deleted = con.execute(
                "DELETE FROM search_history WHERE id > ? AND timestamp >= ?", (max_id_before, start_ts)
            ).rowcount
            con.commit()
            con.close()
            print("restored logs: %d search_history rows removed" % deleted, file=sys.stderr)

    out = {
        "exe": args.exe,
        "workspace": workspace,
        "daemon_version": daemon_version,
        "started": start_ts,
        "index": {k: final_stats.get(k) for k in ("total_files", "total_nodes", "total_edges", "repos")},
        "extra_params": extra_params,
        "n": len(results),
        "results": results,
    }
    os.makedirs(os.path.dirname(os.path.abspath(args.out)) or ".", exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
    print("wrote %s (%d results)" % (args.out, len(results)), file=sys.stderr)


if __name__ == "__main__":
    main()
