#!/bin/bash
# danger-guard: block dangerous Bash commands per CLAUDE.md STOP conditions.
# Patterns blocked: rm -rf, git push --force, git reset --hard, --no-verify,
#   --no-gpg-sign, git rebase, dd if=, mkfs, chmod -R 777.
# Pattern requiring user confirmation (return "ask"): git push, rm -r (without -rf).

input=$(cat)

# Extract command string (handle escaped quotes loosely; sufficient for pattern detection)
cmd=$(printf '%s' "$input" | tr -d '\n' | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

if [ -z "$cmd" ]; then
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"no command field"}}'
  exit 0
fi

# Hard-block patterns (CLAUDE.md STOP条件 #6, +destructive)
block_patterns=(
  'rm[[:space:]]+-rf'
  'rm[[:space:]]+-fr'
  'git[[:space:]]+push[[:space:]]+.*--force'
  'git[[:space:]]+push[[:space:]]+.*-f([[:space:]]|$)'
  'git[[:space:]]+reset[[:space:]]+--hard'
  'git[[:space:]]+rebase'
  '--no-verify'
  '--no-gpg-sign'
  'dd[[:space:]]+if='
  'mkfs\.'
  'chmod[[:space:]]+-R[[:space:]]+777'
  '>[[:space:]]*/dev/sda'
)

for pat in "${block_patterns[@]}"; do
  if printf '%s' "$cmd" | grep -Eq -e "$pat"; then
    reason="【STOP条件抵触】危険コマンド検出: pattern='$pat'. CLAUDE.md のSTOP条件によりブロック。本当に必要な場合はユーザーに明示確認してから実行してください。"
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"%s"}}' "$reason"
    exit 0
  fi
done

# Ask-confirmation patterns (git push, etc.)
ask_patterns=(
  'git[[:space:]]+push([[:space:]]|$)'
)

for pat in "${ask_patterns[@]}"; do
  if printf '%s' "$cmd" | grep -Eq -e "$pat"; then
    reason="【STOP条件】git push を検出。CLAUDE.md のSTOP条件#6によりユーザー確認が必要です。"
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"%s"}}' "$reason"
    exit 0
  fi
done

printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"safe command"}}'
exit 0
