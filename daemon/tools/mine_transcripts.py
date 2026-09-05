#!/usr/bin/env python3
"""Build a ground-truth set from Claude Code transcripts.

WHY: the only honest measure of a code search is whether it pointed at the
files the agent then had to edit. Claude Code keeps every session as JSONL
under ~/.claude/projects/<project>/, with tool calls and results inline, so the
pairing "this run_pipeline call ... these files were edited afterwards" can be
reconstructed without anyone labelling anything.

Rules:
- A run_pipeline call owns the Edit/MultiEdit calls that follow it in the same
  session, up to the next run_pipeline call.
- Only Edit targets count: an Edit proves the file existed when the search ran.
  Write targets are new files (migrations, new scripts), which no index could
  have returned, so they are excluded.
- Markdown, scratchpad and .claude/ files are excluded: they are notes, not the
  code the search was about.
- Paths are qualified as "<repo>/<relative>" the way the daemon reports them,
  using the repo folder names under the GitHub directory.

Usage:
  mine_transcripts.py --project "C:/Users/me/.claude/projects/c--...-Lynium" \
                      --repos-root "C:/Users/me/OneDrive/Documents/GitHub" \
                      --out baseline/lynium-groundtruth.json
"""

import argparse
import glob
import json
import os
import re


def make_qualifier(repos_root, default_repo):
    repos_root_norm = os.path.abspath(repos_root).replace("\\", "/").lower()

    def qualify(path):
        p = path.replace("\\", "/")
        low = p.lower()
        if low.startswith(repos_root_norm + "/"):
            rest = p[len(repos_root_norm) + 1 :]
            repo, _, rel = rest.partition("/")
            return "%s/%s" % (repo, rel) if rel else None
        if ":" not in p and not p.startswith("/"):
            return "%s/%s" % (default_repo, p)
        return None

    return qualify


def is_noise(qualified):
    low = qualified.lower()
    return (
        low.endswith(".md")
        or "/.claude/" in low
        or "scratchpad" in low
        or "/temp/claude/" in low
    )


def mine(project_dir, qualify):
    entries = []
    for fn in sorted(glob.glob(os.path.join(project_dir, "*.jsonl")), key=os.path.getmtime):
        msgs = []
        with open(fn, encoding="utf-8", errors="replace") as f:
            for line in f:
                try:
                    o = json.loads(line)
                except json.JSONDecodeError:
                    continue
                content = (o.get("message") or {}).get("content")
                if isinstance(content, list):
                    msgs.append((o.get("timestamp"), content))

        pending = {}
        calls = []  # (msg_index, input, pivots, timestamp)
        edits = []  # (msg_index, qualified path)
        for i, (ts, content) in enumerate(msgs):
            for block in content:
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "tool_use":
                    name = block.get("name")
                    if name == "mcp__comp__run_pipeline":
                        pending[block["id"]] = (i, block.get("input") or {}, ts)
                    elif name in ("Edit", "MultiEdit"):
                        p = (block.get("input") or {}).get("file_path")
                        q = qualify(p) if p else None
                        if q and not is_noise(q):
                            edits.append((i, q))
                elif block.get("type") == "tool_result" and block.get("tool_use_id") in pending:
                    i0, inp, ts0 = pending.pop(block["tool_use_id"])
                    raw = block.get("content")
                    text = "".join(x.get("text", "") for x in raw if isinstance(x, dict)) if isinstance(raw, list) else str(raw)
                    try:
                        pivots = [p["path"] for p in json.loads(text).get("pivot_files", [])]
                    except (json.JSONDecodeError, AttributeError, TypeError):
                        pivots = []
                    calls.append((i, inp, pivots, ts0))

        for idx, (i, inp, pivots, ts) in enumerate(calls):
            nxt = calls[idx + 1][0] if idx + 1 < len(calls) else len(msgs)
            edited = sorted({q for (j, q) in edits if i < j < nxt})
            if not edited:
                continue
            entries.append(
                {
                    "query": inp.get("task", ""),
                    "params": {k: v for k, v in inp.items() if k != "task"},
                    "session": os.path.basename(fn),
                    "timestamp": ts,
                    "observed_pivots": pivots,
                    "edited": edited,
                }
            )
    return entries


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--project", required=True, help="~/.claude/projects/<project> directory")
    ap.add_argument("--repos-root", required=True, help="directory holding the repo folders (aliases)")
    ap.add_argument("--default-repo", default=None, help="alias for relative paths (default: last segment of the project dir name)")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    default_repo = args.default_repo
    if not default_repo:
        default_repo = re.split(r"[-]", os.path.basename(args.project.rstrip("/\\")))[-1]
    qualify = make_qualifier(args.repos_root, default_repo)
    entries = mine(args.project, qualify)

    total_edited = sum(len(e["edited"]) for e in entries)
    observed_hits = sum(len(set(e["edited"]) & set(e["observed_pivots"])) for e in entries)
    print(
        "%d searches with edits afterwards, %d edited files, %d of them among the observed pivots"
        % (len(entries), total_edited, observed_hits)
    )
    os.makedirs(os.path.dirname(os.path.abspath(args.out)) or ".", exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump({"project": args.project, "entries": entries}, f, ensure_ascii=False, indent=1)
    print("wrote", args.out)


if __name__ == "__main__":
    main()
