# コンテキストエンジニアリング再設計計画

**作成日**: 2026-05-24
**目的**: CLAUDE.mdの肥大化・指示逸脱・トークン浪費を解消し、指示の意図とAIの動作を一致させる
**ステータス**: 実施中

---

## 1. 背景：なぜ作り直すのか

### 観測された問題

- CLAUDE.md が 1062 行に肥大化し、毎ターン system prompt として注入される
- 会話が長くなるとCLAUDE.mdの中盤ルールが守られなくなる（Lost in the Middle 現象）
- 「必ず〜する」という命令はあっても、物理的に止めるhookが無く、暴走を防げない
- Claude Code 既定動作と憲法の細部が矛盾し、優先度判定が曖昧
- 「いつユーザーに確認すべきか」のSTOP条件が未定義で、勝手に進む余地が大きい

### 設計原則（先人ノウハウ）

| 出典 | 原則 | 本計画への反映 |
| --- | --- | --- |
| Anthropic "Effective Context Engineering" | 最小有効プロンプト、ツールはJust-in-time | CLAUDE.mdを100行以下、詳細はdocs/分割 |
| RFC 2119 | MUST/SHOULD/MAY で優先度を明示 | 全ルールに【MUST】【SHOULD】タグ付与 |
| "Lost in the Middle" (Liu et al.) | 重要情報は先頭と末尾に二重配置 | 核心3行を冒頭+末尾に配置 |
| Constitutional AI | 肯定形より否定形が守られやすい | STOP条件リスト（やらない場面の列挙） |
| Cursor/Cline rules best practice | 1ファイル200行ルール、肯定形より禁則明示 | docs/STANDARDS_*.md に200行制限 |

---

## 2. 全体構成（After）

```
e:\dev\comP\
├── CLAUDE.md                         # 憲法（100行以下、核心のみ）
├── .claude\
│   ├── CLAUDE.md                     # vexp/RTKツール規約（既存維持）
│   ├── settings.json                 # hooks 強制機構（拡張）
│   ├── hooks\                        # 物理強制スクリプト
│   │   ├── vexp-guard.sh             # 既存
│   │   ├── danger-guard.sh           # 新規：rm/reset/force系ブロック
│   │   ├── write-confirm.sh          # 新規：新規ファイル作成警告
│   │   ├── stop-checklist.sh         # 新規：完了時チェックリスト
│   │   └── context-inject.sh         # 新規：プロンプト時にSTOP条件再注入
│   └── skills\                       # スキル群
│       ├── skeleton-first\           # スケルトン+コメント生成
│       ├── tdd-flow\                 # テスト→実装→カバレッジ
│       ├── oss-init\                 # README/LICENSE/SBOM一式
│       ├── context-checkpoint\       # temp/への中間保存
│       └── pre-push-check\           # SAST/テスト/カバレッジ統合
└── docs\                              # 詳細リファレンス（必要時に参照）
    ├── CONTEXT_ENGINEERING_PLAN.md   # この文書
    ├── CONSTITUTION_DETAIL.md        # 旧CLAUDE.md全文（参照用）
    ├── STANDARDS_CODING.md           # コーディング・TDD詳細
    ├── STANDARDS_LOGGING.md          # ロギング規約（3言語例含む）
    ├── STANDARDS_ERROR_HANDLING.md   # 例外処理規約
    ├── STANDARDS_CICD.md             # CI/CD・SAST/DAST/SBOM
    ├── STANDARDS_OSS_INIT.md         # OSS作法・メタデータ
    └── STANDARDS_MARKDOWN.md         # Markdown Lint設定
```

---

## 3. 新CLAUDE.md の構造（100行以下）

| セクション | 行数目安 | 内容 |
| --- | --- | --- |
| 1. 核心3行 | 3 | 最重要原則（圧縮されても残す） |
| 2. 言語・対話 | 5 | 日本語、簡潔、確認重視 |
| 3. 【MUST】STOP条件 | 20 | 確認なしに進んではいけない場面リスト |
| 4. 【MUST】必須ツール | 10 | vexp、RTK、TDD |
| 5. 【MUST】禁止事項 | 15 | やってはいけない行為 |
| 6. 【SHOULD】実装フロー | 15 | スケルトン→テスト→実装の要約 |
| 7. 詳細リファレンス | 10 | docs/への@参照 |
| 8. 核心3行（再掲） | 3 | Lost in the Middle 対策 |

---

## 4. 【MUST】STOP条件リスト（暴走防止の核）

以下の場面では、**実行前に必ずユーザーに確認**する：

1. 新規ファイル/フォルダの作成（既存ファイル編集は除く）
2. 3ファイル以上の同時変更
3. 既存テストの削除・スキップ化・期待値変更
4. 依存パッケージの追加・削除・メジャー更新
5. CI/CD設定ファイル（.github/, .gitlab-ci.yml 等）の変更
6. `git push`, `git push --force`, `git reset --hard`, `git rebase`
7. データベース・マイグレーションファイルの変更
8. 認証/認可/暗号化に関するコード変更
9. スコープ外の「ついでにリファクタ」
10. CLAUDE.md, settings.json, hooks の変更

---

## 5. hooks 設定（settings.json拡張）

| イベント | matcher | 動作 |
| --- | --- | --- |
| PreToolUse | `Grep\|Glob\|Regex` | vexp daemon稼働中はブロック（既存） |
| PreToolUse | `Write` | 新規ファイル作成警告（既存ファイル編集ならスルー） |
| PreToolUse | `Bash` | `rm -rf`, `git reset --hard`, `git push --force`, `--no-verify`, `rm -r` を含むコマンドをブロック |
| Stop | * | 完了時チェックリストを表示（テスト・カバレッジ・ドキュメント） |
| UserPromptSubmit | * | タスク開始時に「核心3行 + STOP条件」を再注入 |

---

## 6. スキル群

| スキル名 | トリガー | 内容 |
| --- | --- | --- |
| skeleton-first | 新規実装開始時 | クラス/関数/分岐/変数の骨格+コメント生成 |
| tdd-flow | テスト関連タスク | 正常/異常/境界値テスト→実装→カバレッジ確認 |
| oss-init | プロジェクト初期化 | README/LICENSE/SBOM/CONTRIBUTING一式生成 |
| context-checkpoint | 長時間タスク中 | temp/log/checkpoint_*.md への進捗保存 |
| pre-push-check | push前 | テスト/SAST/カバレッジ統合チェック |

---

## 7. 実施フェーズ

| Phase | 内容 | リスク | 状態 |
| --- | --- | --- | --- |
| 1 | 計画書作成（本文書） | 低 | 進行中 |
| 2 | 旧CLAUDE.mdを docs/CONSTITUTION_DETAIL.md として保存 | 低（追加のみ） | 未着手 |
| 3 | docs/STANDARDS_*.md に分割 | 低 | 未着手 |
| 4 | 新CLAUDE.md ドラフト作成 | 中（既存上書き） | 未着手 |
| 5 | hooks 追加 | 中（誤発動リスク） | 未着手 |
| 6 | skills 作成 | 低 | 未着手 |
| 7 | ユーザー確認・微調整 | - | 未着手 |

---

## 8. 成功指標

- [ ] CLAUDE.md が 100 行以下
- [ ] STOP条件 10 項目以上が hooks で物理的に強制される
- [ ] スキル化により CLAUDE.md 本文から重複指示が消える
- [ ] 「指示通り動かない」発生率の体感が低下
- [ ] `rtk gain` でトークン使用量が前比 30% 以上削減

---

## 9. ロールバック手順

問題発生時：

```bash
# CLAUDE.md を元に戻す
cp docs/CONSTITUTION_DETAIL.md CLAUDE.md

# settings.json を元に戻す
git checkout .claude/settings.json

# 追加した hooks を無効化
rm .claude/hooks/{danger-guard,write-confirm,stop-checklist,context-inject}.sh
```

---

## 10. 修正ログ

| 日付 | 修正内容 |
| --- | --- |
| 2026-05-24 | 初版作成 |
