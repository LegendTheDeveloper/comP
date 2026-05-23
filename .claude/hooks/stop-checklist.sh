#!/bin/bash
# stop-checklist: at end-of-turn (Stop event), inject a completion checklist
# so the assistant verifies CLAUDE.md compliance before declaring done.
# Outputs additionalContext to Stop hook, visible to model on next thought.

cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"Stop","additionalContext":"【完了前チェックリスト】\n- [ ] テスト実行・全てPASS\n- [ ] カバレッジ80%以上\n- [ ] ドキュメント更新（API/使用例/制約）\n- [ ] スケルトンコメント↔実装の一貫性\n- [ ] 関連ファイル整合性\n- [ ] STOP条件抵触の有無を再確認\n未完了項目があれば、ユーザーに報告してから完了宣言してください。"}}
EOF
exit 0
