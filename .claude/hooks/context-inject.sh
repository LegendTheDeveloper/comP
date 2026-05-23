#!/bin/bash
# context-inject: at every UserPromptSubmit, re-inject the 3 core rules + STOP条件 summary.
# This fights "Lost in the Middle" — even when CLAUDE.md is compressed,
# the core constraints arrive fresh in each turn.

cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"【憲法・核心3原則の再注入】\n1. 日本語で対話\n2. STOP条件該当時は実行前に必ずユーザー確認\n3. vexp run_pipeline を最初に呼ぶ（grep/glob/Read原則禁止）\n\n【STOP条件サマリ】新規ファイル作成 / 3ファイル以上の同時変更 / 既存テスト削除 / 依存追加削除 / CI設定変更 / git push系 / DB変更 / 認証コード変更 / 指示外リファクタ / CLAUDE.md・settings.json変更 → 必ず確認。"}}
EOF
exit 0
