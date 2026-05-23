# comP プロジェクト憲法

<!--
バージョン: 2.0 (2026-05-24 スリム化)
旧版（1062行詳細）: docs/CONSTITUTION_DETAIL.md
本ファイルは100行以下を維持すること。詳細は docs/STANDARDS_*.md 参照。
-->

## 核心3原則【必ず守る】

1. **【MUST】日本語で対話する。**
2. **【MUST】STOP条件に該当したら、実行前に必ずユーザーに確認する。**（後述）
3. **【MUST】vexp `run_pipeline` を最初に呼ぶ。grep/glob/Read は原則禁止。**

---

## 【MUST】STOP条件（実行前に必ずユーザー確認）

以下のいずれかに該当する場面では、**作業を止めて確認**してから進める：

1. 新規ファイル・新規フォルダの作成
2. 3ファイル以上の同時変更
3. 既存テストの削除・スキップ化・期待値変更
4. 依存パッケージの追加・削除・メジャー更新
5. CI/CD設定ファイル（`.github/`, `.gitlab-ci.yml`等）の変更
6. `git push`、`git push --force`、`git reset --hard`、`git rebase`
7. データベース・マイグレーションファイルの変更
8. 認証/認可/暗号化に関するコード変更
9. スコープ外の「ついでにリファクタ」（指示外の変更）
10. `CLAUDE.md`、`.claude/settings.json`、`.claude/hooks/` の変更

確認文言例：「〇〇を実施する前に確認です。これは△△の理由で実行しますが、進めてよいですか？」

---

## 【MUST】禁止事項

- ❌ vexpを使わずに grep/glob で探索する
- ❌ コードのみ修正してテスト・ドキュメント未更新で完了とする
- ❌ 既存テストを「失敗するから」という理由で削除・スキップ化する
- ❌ コンテキスト枯渇時に憲法を「省略してよい」と判断する
- ❌ `--no-verify`、`--no-gpg-sign` 等のフック・署名スキップ
- ❌ 「処理の説明」「used by X」等の冗長コメント

---

## 【MUST】必須ツール

| ツール | 用途 | 詳細 |
| --- | --- | --- |
| vexp `run_pipeline` | コード探索・影響分析・メモリ | [.claude/CLAUDE.md](.claude/CLAUDE.md) |
| RTK (`rtk gain`, `rtk discover`) | トークン圧縮・統計 | `~/.claude/RTK.md` |
| `temp/log/checkpoint_*.md` | セッション復帰・進捗保存 | `docs/CONSTITUTION_DETAIL.md` |

---

## 【SHOULD】実装フロー（要約）

```text
[1] スケルトン作成（クラス/関数/分岐/変数 + WHYコメント）
       ↓
[2] 単体テスト（正常系/異常系/境界値）
       ↓
[3] テストが通る最小実装
       ↓
[4] リファクタ + カバレッジ80%以上確認
       ↓
[5] ドキュメント・関連ファイル同時更新
```

詳細: [docs/STANDARDS_CODING.md](docs/STANDARDS_CODING.md)

---

## 【SHOULD】コメント原則

| 書く（WHY） | 書かない（WHAT） |
| --- | --- |
| なぜこの分岐が必要か | コード自体が説明できること |
| 入出力の制約・前提 | 関数・変数の一覧 |
| 非自明なアルゴリズム | 「used by X」「added for Y」 |
| 境界値の扱い | 過去タスクの参照 |

---

## 詳細リファレンス（必要時のみ参照）

- [docs/CONTEXT_ENGINEERING_PLAN.md](docs/CONTEXT_ENGINEERING_PLAN.md) — 本憲法の設計思想
- [docs/STANDARDS_CODING.md](docs/STANDARDS_CODING.md) — TDD・スケルトン・カバレッジ
- [docs/STANDARDS_LOGGING.md](docs/STANDARDS_LOGGING.md) — ログレベル・出力先
- [docs/STANDARDS_ERROR_HANDLING.md](docs/STANDARDS_ERROR_HANDLING.md) — 例外処理パターン
- [docs/STANDARDS_CICD.md](docs/STANDARDS_CICD.md) — テスト・SAST/DAST・SBOM
- [docs/STANDARDS_OSS_INIT.md](docs/STANDARDS_OSS_INIT.md) — README/LICENSE/SBOM
- [docs/STANDARDS_MARKDOWN.md](docs/STANDARDS_MARKDOWN.md) — Markdown Lint
- [docs/CONSTITUTION_DETAIL.md](docs/CONSTITUTION_DETAIL.md) — 旧版全文（参照用）

---

## 核心3原則【再掲・必ず守る】

1. **【MUST】日本語で対話する。**
2. **【MUST】STOP条件に該当したら、実行前に必ずユーザーに確認する。**
3. **【MUST】vexp `run_pipeline` を最初に呼ぶ。grep/glob/Read は原則禁止。**

---

## 修正ログ

| 日付 | 修正内容 |
| --- | --- |
| 2026-05-21 | 初版（1062行） |
| 2026-05-24 | v2.0 スリム化（100行以下、STOP条件導入、詳細をdocs/分離） |
