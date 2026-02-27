# rust-grep — プロジェクト仕様

## 概要
Dioxus 0.7 + Rust による GUIグレップ検索ツール。
高速・省メモリを重視したデスクトップアプリ。

## 技術スタック
- dioxus 0.7（desktop feature）
- regex, ignore, rayon, tokio, content_inspector, rfd

## 対象プラットフォーム
- Windows 10/11 (x86_64)
- macOS 12+ (x86_64 / aarch64)

## ディレクトリ構成
rust-grep/
├── CLAUDE.md
├── Cargo.toml
├── Dioxus.toml
├── assets/
│   └── icon.png
└── src/
    ├── main.rs
    ├── app.rs
    ├── state.rs
    ├── components/
    │   ├── toolbar.rs
    │   ├── options_bar.rs
    │   ├── result_panel.rs
    │   ├── tree_view.rs
    │   ├── flat_view.rs
    │   └── status_bar.rs
    └── search/
        ├── mod.rs
        ├── engine.rs
        ├── walker.rs
        └── matcher.rs

## データ構造

### UiSettings
```rust
enum ResultViewMode { Tree, Flat }
enum SearchTrigger { OnEnter, Realtime }

struct UiSettings {
    result_view_mode: ResultViewMode,
    search_trigger: SearchTrigger,
    debounce_ms: u64,  // Realtimeモード時（デフォルト300ms）
}
```

### SearchOptions
```rust
struct SearchOptions {
    use_regex: bool,
    case_sensitive: bool,
    include_binary: bool,
    max_file_size: u64,
}
```

### SearchResult
```rust
struct SearchResult {
    file_path: PathBuf,
    line_number: usize,
    line_content: String,
    match_ranges: Vec<Range<usize>>,
}
```

### SearchStatus
```rust
enum SearchStatus {
    Idle,
    Running { scanned: usize, matched: usize },
    Done { duration_ms: u64, total_matches: usize },
    Error(String),
}
```

## 実装ルール

### 高速・省メモリ
1. ファイルは行単位でストリーム処理（全体をメモリに乗せない）
2. バイナリ判定は content_inspector で先頭8KBのみ
3. 検索はバックグラウンドで逐次送信（結果が出次第UIに反映）
4. Realtimeモード時は CancellationToken で前回タスクをキャンセル
5. SearchResult はマッチ行のみ保持

### クロスプラットフォーム
6. パス操作は必ず std::path::Path / PathBuf を使用（文字列結合禁止）
7. ファイルダイアログは rfd クレートで統一
8. パス表示は p.display().to_string() を使用
9. Windows固有APIは cfg(windows) でガード

## UIコンポーネント構成
App
├── ToolBar
│   ├── PathSelector       # rfd でネイティブダイアログ
│   ├── SearchInput        # トリガーモードに応じた入力制御
│   └── SearchButton       # OnEnterモード時のみ活性
├── OptionsBar
│   ├── CheckBox: 正規表現
│   ├── CheckBox: 大文字小文字区別
│   ├── CheckBox: バイナリ含む
│   ├── ToggleButton: Tree / Flat
│   └── ToggleButton: Enter / Live
├── ResultPanel
│   ├── TreeView           # ファイル単位折りたたみ
│   └── FlatView           # ripgrep風フラットリスト
└── StatusBar              # 件数・ファイル数・経過時間

## 検索エンジン非同期フロー
UI Thread               Background Thread
─────────────────       ──────────────────────────
[Enter or debounce] ──→ spawn search task
                         ignore::WalkParallel
                         + rayon parallel match
                   ←──  tx.send(SearchResult)  ← 逐次
[Signal更新→再描画]  ←──  tx.send(SearchResult)
                   ←──  tx.send(Done { duration })

## バンドル
dx bundle --release --platform desktop
- Windows: .msi（WebView2が必要）
- macOS:   .dmg（自己完結型）