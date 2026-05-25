# OSS作法・メタデータファイル標準

**参照元**: CLAUDE.md → `@docs/STANDARDS_OSS_INIT.md`
**詳細テンプレート**: [CONSTITUTION_DETAIL.md#oss作法メタデータファイル生成](CONSTITUTION_DETAIL.md)

---

## 1. プロジェクト初期化時の必須ファイル

| ファイル | 説明 | 参考規格 |
| --- | --- | --- |
| `README.md` | 英語、初心者向け詳細ガイド | - |
| `README_ja.md` | 日本語版 | - |
| `LICENSE` | ライセンス宣言（推奨: MIT/Apache2.0） | [SPDX](https://spdx.org/licenses/) |
| `CHANGELOG.md` | 変更履歴 | [Keep a Changelog](https://keepachangelog.com/) + [Semver](https://semver.org/) |
| `SBOM.json` | ソフトウェア部品表 | [CycloneDX](https://cyclonedx.org/) |
| `.gitignore` | Git除外ルール | GitHub template |
| `CONTRIBUTING.md` | 貢献ガイド | - |

## 2. README.md（英語）必須セクション

1. **What It Does** - 3行以内の機能説明 + ユースケース + デモ画像/GIF
2. **Prerequisites** - OS、言語バージョン
3. **Installation** - ステップバイステップ + エラー対応
4. **Quick Start** - 5分以内で動く最小例
5. **Usage** - ユースケース 3〜5個 + コマンド例
6. **Output** - 効果・パフォーマンス数値
7. **Troubleshooting** - FAQ
8. **License** - SPDX identifier
9. **Contributing** - CONTRIBUTING.md リンク

## 3. README_ja.md（日本語）

英語版と同構成：できること / 必要な環境 / インストール / クイックスタート / 使い方 / 効果 / トラブル対応 / ライセンス / 貢献方法

## 4. SBOM 生成

```bash
# Python
pip install cyclonedx-bom
cyclonedx-py -o SBOM.json requirements.txt

# Node.js
npm install -g @cyclonedx/npm
cyclonedx-npm -o SBOM.json
```

## 5. Python環境管理（uv必須）

```bash
uv venv                                  # .venv作成
uv pip install -r requirements.txt       # 依存インストール
```

**チェック**:
- [ ] uv インストール済み
- [ ] `.venv` が `.gitignore` 記載
- [ ] `requirements.txt` が Git管理
- [ ] README に `uv venv` 起動手順記載
