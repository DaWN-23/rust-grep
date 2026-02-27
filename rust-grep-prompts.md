# rust-grep — Claude Code プロンプト集

> Dioxus 0.7 + Rust | Windows / macOS | 全7フェーズ

各フェーズを順番に実行し、**完了条件を確認してから次のフェーズに進んでください。**

---

## フェーズ一覧

| フェーズ | 内容 | 完了条件 |
|---------|------|---------|
| 1 | プロジェクト初期化 | `cargo check` が通ること |
| 2 | state.rs データ構造定義 | `cargo check` が通ること |
| 3 | search/ モジュール実装 | `cargo test` が通ること |
| 4 | Dioxus UI 骨格 | `dx serve` で起動確認 |
| 5 | ResultPanel 実装 | Tree/Flat 切り替え動作確認 |
| 6 | 検索エンジン接続 + トリガー切り替え | Enter/Live 切り替え動作確認 |
| 7 | バンドル・最終確認 | `dx bundle` で成果物生成 |

---

## フェーズ 1 — プロジェクト初期化

**🎯 ゴール:** `cargo check` がエラーなく通るプロジェクト雛形を作る

### タスク

- `cargo new rust-grep` でプロジェクト作成
- Cargo.toml を指定の依存関係に更新
- Dioxus.toml を作成
- CLAUDE.md をプロジェクトルートに配置
- `assets/` ディレクトリと icon.png プレースホルダーを作成
- `src/` の各ディレクトリ・空ファイルを作成

### Claude Code プロンプト

```
以下の仕様に従って rust-grep プロジェクトを初期化してください。

## Cargo.toml

[package]
name = "rust-grep"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { version = "0.7", features = ["desktop"] }
regex = "1"
ignore = "0.4"
rayon = "1"
tokio = { version = "1", features = ["full"] }
content_inspector = "0.2"
rfd = "0.15"
serde = { version = "1", features = ["derive"] }

## Dioxus.toml

[application]
name = "rust-grep"
default_platform = "desktop"

[bundle]
identifier = "com.rust-grep.app"
publisher = "rust-grep"
icon = ["assets/icon.png"]

## 作成するディレクトリ/ファイル構成

rust-grep/
├── CLAUDE.md        （仕様書、内容は別途指示）
├── Cargo.toml
├── Dioxus.toml
├── assets/
│   └── icon.png     （32x32 の空PNGで可）
└── src/
    ├── main.rs
    ├── app.rs
    ├── state.rs
    ├── components/
    │   ├── mod.rs
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

## 完了条件
- cargo check がエラーなく通ること
```

### 完了条件

- `cargo check` がエラーなく通ること
- CLAUDE.md がプロジェクトルートに存在すること
- `src/` の全ファイルが作成されていること

---

## フェーズ 2 — state.rs データ構造定義

**🎯 ゴール:** アプリ全体の状態・オプション・結果を型定義する

### タスク

- `ResultViewMode` / `SearchTrigger` enum を定義
- `UiSettings` 構造体を定義
- `SearchOptions` 構造体を定義
- `SearchResult` 構造体を定義
- `SearchStatus` enum を定義
- `AppState` 構造体（Signal でラップ）を定義

### Claude Code プロンプト

```
state.rs に以下のデータ構造をすべて実装してください。
Dioxus の Signal を使った AppState も定義してください。

## 定義する型

// 表示モード切り替え
#[derive(Clone, PartialEq)]
pub enum ResultViewMode { Tree, Flat }

// 検索トリガー切り替え
#[derive(Clone, PartialEq)]
pub enum SearchTrigger { OnEnter, Realtime }

pub struct UiSettings {
    pub result_view_mode: ResultViewMode,
    pub search_trigger: SearchTrigger,
    pub debounce_ms: u64,  // デフォルト 300
}

pub struct SearchOptions {
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub include_binary: bool,
    pub max_file_size: u64,  // バイト単位、デフォルト 100MB
}

pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_ranges: Vec<Range<usize>>,
}

pub enum SearchStatus {
    Idle,
    Running { scanned: usize, matched: usize },
    Done { duration_ms: u64, total_matches: usize },
    Error(String),
}

## 完了条件
- cargo check がエラーなく通ること
```

### 完了条件

- `cargo check` がエラーなく通ること
- 全型に `Clone` / `PartialEq` / `Debug` が適切に derive されていること
- `AppState` が Dioxus Signal でラップされていること

---

## フェーズ 3 — search/ モジュール実装

**🎯 ゴール:** UI なしで検索ロジック単体をテストできる状態にする

### タスク

- `matcher.rs`: Regex / リテラル切り替えマッチャーを実装
- `walker.rs`: `ignore::WalkParallel` ラッパーを実装
- `engine.rs`: tokio + rayon の非同期検索エンジンを実装
- バイナリ判定ロジック（`content_inspector`）を実装
- ユニットテストを各モジュールに追加

### Claude Code プロンプト

```
search/ モジュールを以下の仕様で実装してください。

## matcher.rs
- SearchOptions.use_regex が true なら Regex マッチ
- false ならリテラル検索（case_sensitive も反映）
- マッチした Range<usize> の Vec を返す find_matches() を実装

## walker.rs
- ignore::WalkParallel でディレクトリを並列走査
- content_inspector で先頭 8KB を読みバイナリ判定
- SearchOptions.include_binary が false ならバイナリをスキップ
- SearchOptions.max_file_size を超えるファイルをスキップ

## engine.rs
- 検索は tokio::spawn でバックグラウンド実行
- rayon で各ファイルを並列処理
- 結果は mpsc::Sender<SearchResult> で逐次送信
- CancellationToken で前回タスクをキャンセル可能にする
- 完了時に SearchStatus::Done を送信

## テスト
- リテラル検索のユニットテスト
- 正規表現検索のユニットテスト
- バイナリファイルスキップのテスト

## 完了条件
- cargo test がすべて通ること
```

### 完了条件

- `cargo test` がすべて通ること
- リテラル / 正規表現 / バイナリスキップの各テストが存在すること
- `CancellationToken` によるキャンセルが実装されていること

---

## フェーズ 4 — Dioxus UI 骨格

**🎯 ゴール:** `dx serve` でウィンドウが開き、ToolBar と StatusBar が表示される

### タスク

- `main.rs`: `dioxus::launch(App)` のエントリポイント実装
- `app.rs`: AppState 初期化 + 全コンポーネントの配置
- `toolbar.rs`: PathSelector（rfd）+ SearchInput + SearchButton
- `options_bar.rs`: 各チェックボックスとトグルボタン
- `status_bar.rs`: SearchStatus の表示
- `result_panel.rs`: モードに応じた切り替え（中身は仮）

### Claude Code プロンプト

```
Dioxus 0.7 で以下の UI 骨格を実装してください。
検索ロジックとの接続はフェーズ6で行うため、
このフェーズでは UI の表示と基本的な状態更新のみ実装します。

## main.rs
fn main() { dioxus::launch(App); }

## app.rs
- use_context_provider で AppState をグローバルに提供
- ToolBar / OptionsBar / ResultPanel / StatusBar を縦に配置

## toolbar.rs
- PathSelector: rfd::FileDialog でディレクトリ選択
- SearchInput: onkeydown で Enter 検知（OnEnter モード時）
- SearchButton: OnEnter モード時のみ活性

## options_bar.rs
- checkbox: 正規表現 / 大文字小文字区別 / バイナリ含む
- toggle: Tree | Flat（ResultViewMode 切り替え）
- toggle: Enter | Live（SearchTrigger 切り替え）

## status_bar.rs
- SearchStatus::Idle    → 「待機中」
- SearchStatus::Running → 「検索中... X件」
- SearchStatus::Done    → 「X件マッチ / Yファイル / Zms」
- SearchStatus::Error   → エラーメッセージを赤字で表示

## スタイル
- Tailwind CSS を使用（Dioxus 0.7 の auto tailwind 機能）
- ダークテーマを基本とする

## 完了条件
- dx serve でウィンドウが開くこと
- ToolBar / OptionsBar / StatusBar が表示されること
- パス選択ダイアログが開くこと
```

### 完了条件

- `dx serve` でウィンドウが開くこと
- ToolBar / OptionsBar / StatusBar が表示されること
- rfd のパス選択ダイアログが開くこと
- Tree/Flat トグルと Enter/Live トグルが表示されること

---

## フェーズ 5 — ResultPanel 実装

**🎯 ゴール:** Tree / Flat 表示の切り替えが動作し、ダミーデータで表示確認できる

### タスク

- `tree_view.rs`: ファイル単位折りたたみ + マッチ行表示
- `flat_view.rs`: ripgrep 風フラットリスト表示
- `result_panel.rs`: ResultViewMode に応じた切り替え
- マッチ箇所のハイライト表示（`match_ranges` を使用）
- ダミーデータで表示を確認

### Claude Code プロンプト

```
ResultPanel を以下の仕様で実装してください。

## tree_view.rs
- ファイルごとにグループ化して表示
- ファイル名クリックで折りたたみ / 展開トグル
- マッチ件数をファイル名の横に表示（例: src/main.rs (3件)）
- 各行は「行番号: 内容」形式で表示
- match_ranges を使いマッチ箇所を背景色でハイライト

## flat_view.rs
- 全マッチをフラットに表示
- 各行は「ファイルパス:行番号: 内容」形式
- match_ranges を使いマッチ箇所をハイライト

## result_panel.rs
- ResultViewMode::Tree → TreeView を表示
- ResultViewMode::Flat → FlatView を表示
- 結果が 0 件のとき「マッチなし」を表示

## 動作確認用ダミーデータ
- AppState に仮の SearchResult を 5件程度セットして表示確認

## 完了条件
- Tree / Flat の切り替えが動作すること
- マッチ箇所がハイライトされること
```

### 完了条件

- Tree / Flat 切り替えが動作すること
- ファイルツリーの折りたたみ / 展開が動作すること
- マッチ箇所がハイライト表示されること
- ダミーデータで両モードの表示が確認できること

---

## フェーズ 6 — 検索エンジン接続 + トリガー切り替え

**🎯 ゴール:** 実際のファイル検索が動作し、Enter / Live 切り替えが機能する

### タスク

- UI と `search/engine.rs` を接続
- OnEnter モード: Enter キー / ボタン押下で検索開始
- Realtime モード: 300ms デバウンスで自動検索
- `CancellationToken` で前回タスクをキャンセル
- 検索結果を逐次 Signal に反映
- SearchStatus を Running → Done / Error に更新

### Claude Code プロンプト

```
UI と検索エンジンを接続し、トリガーモードを実装してください。

## 接続方法
- use_coroutine または use_future で検索タスクを管理
- mpsc::channel で engine.rs から結果を受信
- 受信のたびに results Signal に push して再描画

## OnEnter モード
- SearchInput の onkeydown で Enter を検知して検索開始
- SearchButton の onclick でも検索開始
- 検索開始時に results をクリアして SearchStatus::Running にセット

## Realtime モード
- query が変化して 300ms 後に検索開始（debounce）
- 新しい入力が来たら CancellationToken でキャンセルして再実行
- SearchButton は非活性

## エラーハンドリング
- 不正な正規表現の場合は SearchStatus::Error に設定
- ディレクトリアクセスエラーも同様に処理

## 完了条件
- 実際のディレクトリで検索が動作すること
- Enter / Live 切り替えが動作すること
- 結果が逐次表示されること
```

### 完了条件

- 実際のディレクトリで検索が動作すること
- Enter / Live 切り替えが機能すること
- 結果が逐次（出次第）表示されること
- 不正な正規表現でエラー表示されること
- SearchStatus が正しく遷移すること

---

## フェーズ 7 — バンドル・最終確認

**🎯 ゴール:** Windows / macOS 向けにリリースバイナリを生成できる

### タスク

- `dx bundle --release --platform desktop` を実行
- Windows: .msi の生成確認
- macOS: .dmg の生成確認
- アイコン設定（`assets/icon.png` の差し替え）
- Dioxus.toml のバンドル設定を最終調整

### Claude Code プロンプト

```
以下の手順でリリースバンドルを生成・確認してください。

## ビルド前チェック
- dx doctor で依存関係に問題がないか確認
- cargo test で全テストが通ることを確認

## バンドル生成
dx bundle --release --platform desktop

## 確認事項
- dist/ ディレクトリに成果物が生成されること
- macOS: .app および .dmg が生成されること
- Windows: .msi が生成されること（Windows 環境の場合）
- アプリ名が「rust-grep」になっていること

## Dioxus.toml 最終確認
- アイコンパスが正しく設定されていること
- identifier が com.rust-grep.app になっていること

## 完了条件
- dx bundle --release が成功すること
- 生成された .app / .dmg を起動して動作確認できること
```

### 完了条件

- `dx bundle --release --platform desktop` が成功すること
- 成果物が `dist/` に生成されること
- 生成されたアプリが起動して検索が動作すること

---

## Claude Code 活用 Tips

**効果的な使い方**

- 各フェーズは独立して実行 — 前フェーズの完了条件を確認してから次へ
- エラーが出たらエラーメッセージをそのまま Claude Code に貼り付けて修正依頼
- 大きな変更の前に `git commit` しておくと安全
- `dx serve --hotpatch` で Rust コードをホットリロードしながら開発可能

**よくある問題と対処**

- WebView2 エラー（Windows）: Microsoft から WebView2 Runtime をインストール
- `cargo check` は通るが `dx serve` が失敗: `dx doctor` で依存関係を確認
- Tailwind クラスが反映されない: Dioxus 0.7 の auto tailwind 機能を確認
- rfd ダイアログが開かない（macOS）: Info.plist の権限設定を確認
