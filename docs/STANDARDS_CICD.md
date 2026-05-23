# CI/CD・セキュリティテスト標準

**参照元**: CLAUDE.md → `@docs/STANDARDS_CICD.md`
**詳細**: [CONSTITUTION_DETAIL.md#cicdセキュリティテスト要件](CONSTITUTION_DETAIL.md)

---

## 1. PUSH時の必須チェック

### 単体テスト

| 言語 | コマンド | カバレッジ |
| --- | --- | --- |
| Python | `pytest --cov=src --cov-report=term-missing` | 80%以上 |
| Node.js | `npm test -- --coverage` | 80%以上 |
| Rust | `cargo test --all` | 80%以上 |

### SAST（静的セキュリティ）

| 言語 | ツール |
| --- | --- |
| Python | `bandit -r src/` + `semgrep --config p/security-audit src/` |
| Node.js | `npm audit --production` + `semgrep` |
| Rust | `cargo audit` |

**脆弱性レベル**:
- Critical/High → PUSHブロック
- Medium → 警告のみ

### DAST（動的セキュリティ／該当時のみ）

- HTTPS使用確認
- セキュリティヘッダ（CSP, HSTS, X-Frame-Options）
- 認証/認可動作確認

## 2. GitHub Actions ワークフロー

`.github/workflows/ci.yml` 雛形は [CONSTITUTION_DETAIL.md](CONSTITUTION_DETAIL.md) 参照。

主要ジョブ：

- **test**: テスト実行 + カバレッジレポート + 依存関係セキュリティチェック
- **sbom** (タグリリース時): CycloneDX形式 SBOM 生成 → Release添付 + コミット

## 3. ローカル検証コマンド集

```bash
# テスト + カバレッジ
pytest --cov=src --cov-report=term-missing

# SAST
bandit -r src/
semgrep --config p/security-audit src/

# SBOM
cyclonedx-py -o SBOM.json requirements.txt
```

## 4. チェックリスト

- [ ] 全テスト 🟢 カバレッジ 80% 以上
- [ ] SAST Critical/High なし
- [ ] DAST セキュリティヘッダ確認（該当時）
- [ ] GitHub Actions 結果確認

## 5. SBOM（ソフトウェア部品表）

- **形式**: CycloneDX JSON
- **生成タイミング**: タグ付きリリース時
- **配布先**: GitHub Release添付 + リポジトリコミット
