#!/bin/bash
# write-confirm: warn (ask) when Write tool creates a NEW file (STOP条件 #1).
# Existing-file edits pass through (Edit tool is preferred for those anyway).

input=$(cat)

# Extract file_path field
file_path=$(printf '%s' "$input" | tr -d '\n' | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

if [ -z "$file_path" ]; then
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"no file_path field"}}'
  exit 0
fi

# Normalize Windows-style backslashes to forward slashes for test -f
normalized=$(printf '%s' "$file_path" | sed 's|\\\\|/|g')

if [ -f "$normalized" ]; then
  # Existing file: allow (it's an overwrite, but content edit is the intent)
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"existing file edit"}}'
  exit 0
fi

# New file creation → STOP条件 #1 抵触 → ask
reason="【STOP条件#1】新規ファイル作成を検出: $file_path. CLAUDE.md のSTOP条件によりユーザー確認が必要です。"
printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"%s"}}' "$reason"
exit 0
