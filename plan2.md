# plan2 — インデックス除外バグ（.venv 混入・forceReindex タイムアウト）修正計画

作成日: 2026-06-13
症状: 別リポジトリで forceReindex が約 4,417 ファイル（うち .venv 約 4,200）を再解析しようとしてタイムアウト。

---

## 1. 調査結果（根本原因）

### Bug 1【最重要】walker が除外ディレクトリの配下を枝刈りしない

[daemon/src/indexer/walker.rs:118-124](daemon/src/indexer/walker.rs#L118-L124)

```rust
for entry in WalkDir::new(&self.workspace_root)
    .into_iter()
    .filter_map(|e| e.ok())
{
    if self.should_skip_entry(&entry) {
        continue;   // ← そのエントリ1件を飛ばすだけ。配下には降りていく
    }
```

- `skip_hidden=true` により `.venv` ディレクトリ**自体**は名前が `.` 始まりなのでスキップされる
- しかし `WalkDir` は構わず配下に降りる。`.venv/Lib/site-packages/six.py` の
  `file_name()` は `six.py` で隠しファイルではなく、`skip_patterns`
  （node_modules / .git / .comp / target / dist / build）にも `.venv` が無いため**素通りしてインデックスされる**
- 証拠: 本リポジトリ自身のインデックスにも `.venv/Lib/site-packages/pandas/...` や
  `.claude/CLAUDE.md`（隠しdir配下）が大量に入っている（run_pipeline の pivot_files で確認済み）

### Bug 2: 既定 skip_patterns に Python 系が無い + 部分文字列マッチの誤爆

[daemon/src/indexer/walker.rs:60-67](daemon/src/indexer/walker.rs#L60-L67), [walker.rs:187-192](daemon/src/indexer/walker.rs#L187-L192)

- `.venv` / `venv` / `__pycache__` / `.pytest_cache` などPythonの定番が既定に無い
- `path.to_string_lossy().contains(pattern)` の**部分一致**は誤爆する:
  - `src/builder.rs` → "build" にマッチして除外される
  - `targets.rs` / `retargeting.ts` → "target" にマッチ
  - `.gitignore` / `.github` → ".git" にマッチ
  - **フルパス**判定のため、ワークスペースの親フォルダ名に "build" 等が含まれると全ファイル除外

### Bug 3: .gitignore 非対応（ドキュメントと実装の乖離）

- [walker.rs:1](daemon/src/indexer/walker.rs#L1) ヘッダは「Filesystem walker with .gitignore support」と謳うが、実装は TODO のまま（[walker.rs:107](daemon/src/indexer/walker.rs#L107)）
- [docs/user/CONFIGURATION.md:116](docs/user/CONFIGURATION.md#L116) は「comP respects `.gitignore`」と**虚偽の記載**。さらに `files.exclude` を案内しているが、daemon は独自walkするため VS Code 設定は無効
- 実際に機能するのは `.comp/ignore` のみ（[daemon/src/indexer/mod.rs:49-57](daemon/src/indexer/mod.rs#L49-L57)）だが、CONFIGURATION.md に説明が無い
- 注: 受領したアドバイス中の `.compignore` や `comp.exclude` 設定は**存在しない**（正しくは `.comp/ignore`）

### Bug 4: FileSystemWatcher / index_file に除外判定が無い

- [src/extension.ts:215-216](src/extension.ts#L215-L216) — `**/*.{ts,...}` 全体を監視、除外なし。
  `pip install` 等で `.venv` 配下が変化すると大量の `indexFile` 要求が daemon に飛ぶ
- daemon 側 `handle_index_file` → [Indexer::index_file](daemon/src/indexer/mod.rs#L274) も skip 判定を一切通さず、来たパスをそのままインデックスする

### Bug 5: forceReindex は同期ブロッキング・タイムアウト120秒

- [src/daemon/DaemonManager.ts:244](src/daemon/DaemonManager.ts#L244) — `forceReindex: 120_000`
- [daemon/src/mcp/mod.rs:1014-1026](daemon/src/mcp/mod.rs#L1014-L1026) — clear → 全ファイル再パースを同期実行。
  4,400 ファイルの tree-sitter パース＋SHA256 で 120 秒を超過 → 今回のタイムアウトの直接原因
  （除外が直れば件数 123 まで減り実質解消するが、巨大モノレポでは再発しうる）

---

## 2. 即時ワークアラウンド（コード修正不要・該当リポジトリで今すぐ可能）

`.comp/ignore` を作成して再インデックス:

```text
.venv
venv
__pycache__
.pytest_cache
```

`matches_ignore_pattern` はパスを `/` で分割した**セグメント完全一致**なので、
`.venv/Lib/.../six.py` のような配下ファイルも現行コードで除外できる
（走査とディレクトリ降下自体は残るが、ハッシュ計算・パースはスキップされるため実用上十分）。

---

## 3. 修正計画

### Fix 1: `filter_entry` によるディレクトリ枝刈り【最優先】

対象: [daemon/src/indexer/walker.rs](daemon/src/indexer/walker.rs)

```rust
for entry in WalkDir::new(&self.workspace_root)
    .into_iter()
    .filter_entry(|e| e.depth() == 0 || !self.should_skip_entry(e))
    .filter_map(|e| e.ok())
```

- `filter_entry` はディレクトリが false ならその**サブツリーごと降りない**（walkdir の標準機能）
- `depth() == 0`（ルート自身）は常に許可 — ワークスペースのフォルダ名が `.` 始まり等でも全スキップにならないようにする
- 副次効果: `.venv` / node_modules への物理走査・ハッシュ計算が消え、走査自体が大幅に高速化

### Fix 2: 既定 skip_patterns 拡充 + セグメント完全一致化

- `should_skip_entry` の `contains()` 部分一致を廃止し、**ワークスペース相対パス**の
  セグメント完全一致に統一（既存 `matches_ignore_pattern` と同じロジックに寄せる）
- 既定パターン追加（非隠し名のみ必須。`.` 始まりは skip_hidden + Fix 1 で枝刈りされる）:
  `venv`, `__pycache__`, `coverage`, `vendor`, `out`
  （`.venv`, `.pytest_cache`, `.mypy_cache`, `.tox` 等は隠しdirなので Fix 1 で自動除外）
- 回帰防止: `src/builder.rs`・`targets.rs`・`.gitignore` が除外**されない**ことをテストで固定

### Fix 3: .gitignore 対応（2案・要選択）

- **案A（推奨）**: [`ignore` クレート](https://crates.io/crates/ignore)（ripgrep の walker）へ置き換え。
  gitignore 文法完全対応（否定 `!`・`**`・ネストした .gitignore）、並列walkで高速。
  ⚠️ **依存パッケージ追加 = STOP条件 → 実装前にユーザー承認必要**
- 案B: 依存追加なし。ルートの `.gitignore` を読んで `ignore_patterns` にマージする簡易実装。
  否定パターン・`**`・アンカー非対応の制限をドキュメントに明記

### Fix 4: incremental 経路の防御

- daemon 側 `Indexer::index_file`（[mod.rs:274](daemon/src/indexer/mod.rs#L274)）の先頭で
  walker と同じ skip 判定を通し、除外対象なら no-op で return
  （防御を daemon に一元化 — watcher 以外の呼び出し元にも効く）
- （任意）[extension.ts:216](src/extension.ts#L216) の watcher コールバックでも
  `.venv`/`node_modules` 等を弾いて IPC 自体を削減

### Fix 5: ドキュメント修正

- [docs/user/CONFIGURATION.md:116](docs/user/CONFIGURATION.md#L116) の「respects `.gitignore`」
  「`files.exclude` で除外」を実態に合わせて修正。`.comp/ignore` の書式・例を追記
- [walker.rs:1](daemon/src/indexer/walker.rs#L1) ヘッダコメントも Fix 3 の実装に合わせて更新
- README / GETTING_STARTED に「Python プロジェクトでの注意」(.venv 除外）を追記

### テスト計画（TDD: スケルトン→テスト→実装）

| テスト | 検証内容 |
| --- | --- |
| `test_walk_prunes_hidden_dirs` | `.venv/sub/a.py` を作成 → files に含まれない |
| `test_walk_prunes_skip_pattern_dirs` | `venv/`, `__pycache__/` 配下が除外される |
| `test_no_substring_false_positive` | `src/builder.rs`, `targets.rs` が**除外されない** |
| `test_gitignore_patterns_applied` | .gitignore の `.venv/` 記載で除外される（Fix 3） |
| `test_index_file_skips_excluded_path` | index_file に `.venv/...` を渡すと no-op（Fix 4） |
| 既存テスト | walker / indexer の既存テスト全维持 |

### 変更ファイル一覧（STOP条件: 3ファイル以上 → 実装開始前に承認を得る）

1. `daemon/src/indexer/walker.rs`（Fix 1, 2, 3）
2. `daemon/src/indexer/mod.rs`（Fix 3 の .gitignore 読み込み, Fix 4）
3. `daemon/src/mcp/mod.rs`（Fix 4 の経路確認）
4. `src/extension.ts`（Fix 4 任意分）
5. `docs/user/CONFIGURATION.md` ほかドキュメント（Fix 5）
6. テストファイル
7. （案A採用時）`daemon/Cargo.toml` — 依存追加

---

## 4. その他の修正推奨事項（今回の直接原因ではないが指摘）

1. **forceReindex の非同期ジョブ化**: 除外修正後も巨大モノレポでは 120 秒超の可能性。
   リクエストを即 ack して進捗を notification で返す方式、最低限タイムアウトの設定化を推奨
   （[DaemonManager.ts:244](src/daemon/DaemonManager.ts#L244)）
   → **見送り（4-1）**: ファイル数激減で実質解消。大規模化時に再検討。
2. **ファイルサイズ上限が無い**: `calculate_file_hash` / `read_to_string` が無制限に全読み込み。
   閾値（例: 5MB）超のファイルはスキップ＋ログを推奨（[walker.rs:230-235](daemon/src/indexer/walker.rs#L230-L235)）
   → **実装済み（§4-2）**: §3 Fix 2 相当として実装完了。
3. **max_nodes チェックが事後**: インデックス完了後に超過判定している（[mod.rs:139-159](daemon/src/indexer/mod.rs#L139-L159)）。
   walk 直後のファイル数で「多すぎる」事前警告を出すと今回のような事故に早く気づける
   （例: 2,000 ファイル超で警告 + 上位ディレクトリ別の内訳を表示）
   → **実装済み（§4-3）**: 2,000 ファイル超で warning ログ実装済み。
4. **forceReindex の workspace_root が env 変数頼み**（[mcp/mod.rs:1018-1020](daemon/src/mcp/mod.rs#L1018-L1020)）:
   起動時の root と乖離しうる。daemon の state に保持した root を使うべき
   → **✅ 実装完了（4-4）**: `AppState` に `workspace_root: String` を追加し、全ハンドラで
     `self.state.workspace_root` から取得するよう統一。env var は `main()` 起動時のみ使用。
5. **`comp.exclude` 的な VS Code 設定が無い**: package.json の contributes.configuration に
   除外設定を追加し、daemon に渡す（または `.comp/ignore` 生成を促す）と、
   今回もらったアドバイスのような UX が実現できる
   → **✅ 実装完了（4-5）**: `comp.exclude` 設定を追加し、`.comp/config.json` の `exclude` に
     同期する `syncExcludeToConfig()` を実装。daemon の `Indexer::new` で `load_exclude_patterns`
     を読み込んで `extra_skip_names` に追記。
6. **Fix 1 適用後は隠しdir配下が一切入らなくなる**: 現在 `.claude/CLAUDE.md` が
   インデックスされているのは Bug 1 の副作用。意図的に含めたい隠しパスがあるなら
   allowlist（negate パターン）が必要 — Fix 3 案A なら .gitignore の `!` で自然に解決
   → **見送り（4-6）**: 隠しdir allowlist は設計と衝突するため今回スコープ外。

---

## 5. 既知テスト不具合ロードマップ（いつか直す）

### KnownBug-1: `test_handle_compress_file` — llvm-cov 実行時 "outside workspace" エラー

- **症状**: `cargo llvm-cov` 実行時のみ失敗。通常の `cargo test` では単独実行で PASS、並列実行でも概ね PASS（稀にフレーキー）。
- **原因**: `mcp::tests::test_handle_compress_file` が一時ディレクトリ（`C:\...\Temp\comP_test_compress_file\`）にファイルを作成し、それを daemon の `handle_compress_file` に渡している。llvm-cov は `llvm-cov-target` 配下を workspace root として設定するため、一時ディレクトリがワークスペース外と判定され `Access denied: outside workspace` エラーになる。
- **修正方針**: テスト内でワークスペースルートを一時ディレクトリに合わせるか、`compress_file` のパス検証を workspace root から `AppState::workspace_root` 経由で取得するよう修正する。
- **該当箇所**: [daemon/src/mcp/mod.rs](daemon/src/mcp/mod.rs) の `test_handle_compress_file` と `handle_compress_file`

### KnownBug-2: `test_session_recall` — 並列実行時のフレーキー

- **症状**: `cargo test`（全テスト並列実行）で稀に失敗。`assertion failed: result.as_str().unwrap().contains("test task")` — 単独実行では常に PASS。
- **原因**: SQLite の書き込みと session_recall の読み込みの間に競合が発生している可能性。各テストが共有の一時 DB を使っているか、テスト間の状態汚染が疑われる。
- **修正方針**: テストごとに独立した TempDir + GraphDB を使うよう修正し、共有状態を排除する。
- **該当箇所**: [daemon/src/mcp/mod.rs](daemon/src/mcp/mod.rs) の `test_session_recall`
