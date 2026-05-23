# Markdown 品質・Lint 標準

**参照元**: CLAUDE.md → `@docs/STANDARDS_MARKDOWN.md`

---

## 1. Lint ツール

- 主ツール: `markdownlint-cli2` (Node.js)
- 代替: `mdl`

## 2. セットアップ

```bash
npm install --save-dev markdownlint-cli2
```

## 3. 設定ファイル `.markdownlintrc.json`

| ルール | 設定 | 理由 |
| --- | --- | --- |
| MD003（見出しスタイル） | 無効化 | プロジェクト混在許容 |
| MD013（行長） | 無効化 | 日本語折り返し問題 |
| MD024（重複見出し） | siblings_only | 兄弟見出しのみチェック |
| MD034（裸URL） | 無効化 | 内部リンク許容 |

## 4. 実行コマンド

```bash
npm run lint:md          # チェックのみ
npm run lint:md:fix      # 自動修正
```

## 5. 禁止される警告（必ず修正）

- 不正な見出しレベル（H1複数使用）
- テーブル不整形
- 空行不足
- リスト形式エラー

## 6. 許可される警告

- ドメイン固有の慣例（プロジェクト特有書式）

## 7. CI/CD連携

`.github/workflows/lint.yml`:

```yaml
name: Markdown Lint
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v3
      - run: npm install --save-dev markdownlint-cli2
      - run: npm run lint:md
```

## 8. VSCode 拡張

- 拡張機能: `vscode-markdownlint` (David Anson)
- 効果: 保存時自動チェック、問題パネルにエラー表示
