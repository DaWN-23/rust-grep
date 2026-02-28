# rust-grep — Claude Code プロンプト集

> Dioxus 0.7 + Rust | Windows / macOS | 全19フェーズ（8〜19は追加要件）

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
| **8** | **検索結果の中断機能** | **中断ボタンで即時停止できること** |
| **9** | **Enter/Live 切替フリーズ対策** | **切替時に UI がフリーズしないこと** |
| **10** | **長時間検索の待ち表示** | **5秒超過でスピナー表示されること** |
| **11** | **CSV/TSV エクスポート** | **ファイル出力の動作確認** |
| **12** | **Enter完了後のLive切替フリーズ解消** | **全状態からの切替が安全に動作すること** |
| **13** | **切替時リセット＆再検索による確実なフリーズ解消** | **全状態・全パターンでフリーズしないこと** |
| **14** | **Live切替後の中断ボタンフリーズ解消** | **中断→再検索→中断の繰り返しが安定すること** |
| **15** | **フリーズの包括的解消（SearchEngine集約）** | **全シナリオでフリーズしないこと** |
| **16** | **Live検索機能の廃止** | **Enter検索のみのシンプルな構成に整理されること** |
| **17** | **検索結果のクリア機能** | **クエリ・結果・状態が全消去されること** |
| **18** | **検索の中止機能** | **クエリを残して結果リセット・再検索できること** |
| **19** | **ステータスバーのリアルタイム性向上** | **件数・時間が50ms以下で更新されること** |

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

## フェーズ 8 — 検索結果の中断機能

**🎯 ゴール:** 検索中に「中断」ボタンで検索を即時停止できる

### 追加・変更対象ファイル

- `src/state.rs` — `SearchStatus` に `Cancelled` を追加
- `src/search/engine.rs` — `CancellationToken` を外部から操作可能にする
- `src/components/toolbar.rs` — SearchButton を状態に応じて「検索」/「中断」に切り替え

### Claude Code プロンプト

```
フェーズ6の成果物に対して、検索中断機能を追加実装してください。
既存のコードを上書き修正する形で対応してください。

## state.rs の変更
SearchStatus に Cancelled バリアントを追加する。

pub enum SearchStatus {
    Idle,
    Running { scanned: usize, matched: usize, elapsed_ms: u64 },
    Done { duration_ms: u64, total_matches: usize },
    Cancelled { matched: usize },   // ← 追加
    Error(String),
}

## search/engine.rs の変更
- AppState に Arc<CancellationToken> を保持させ、外部から cancel() を呼べるようにする
- キャンセル時は SearchStatus::Cancelled { matched: 現在件数 } をセットして終了する

## components/toolbar.rs の変更
SearchButton をステータスに応じて切り替える。
- SearchStatus::Idle / Done / Cancelled / Error → 「🔍 検索」ボタン（活性）
- SearchStatus::Running → 「⏹ 中断」ボタン（クリックで cancel() を呼ぶ）

## status_bar.rs の変更
- SearchStatus::Cancelled → 「中断しました（X件マッチ済み）」をオレンジ色で表示

## 完了条件
- 検索中に「中断」ボタンが表示されること
- クリックで検索が即時停止すること
- 中断後に「中断しました（X件マッチ済み）」が表示されること
- 中断後に再度「検索」ボタンに戻ること
```

### 完了条件

- 検索中に「⏹ 中断」ボタンが表示されること
- クリックで検索が即時停止すること
- ステータスバーに中断済み件数が表示されること
- 中断後に再検索できること

---

## フェーズ 9 — 検索方式切替時のフリーズ対策

**🎯 ゴール:** Enter/Live トグル切替時に UI がフリーズしない

### 追加・変更対象ファイル

- `src/search/engine.rs` — 切替時の安全なタスク終了処理
- `src/components/options_bar.rs` — トグル切替ハンドラの非同期化
- `src/app.rs` — トリガーモード変更の副作用処理

### Claude Code プロンプト

```
フェーズ6の成果物に対して、Enter/Live 切替時のフリーズ対策を実装してください。
既存のコードを上書き修正する形で対応してください。

## 問題の原因
切替時に前回の検索タスクが完了していない場合、
Signal の更新が UI スレッドをブロックしてフリーズが発生する。

## 対策方針
切替操作を use_coroutine 経由で非同期に処理し、
UIスレッドを一切ブロックしないようにする。

## app.rs / options_bar.rs の変更
トグル切替ハンドラを以下のフローで実装する。
1. 既存の CancellationToken を cancel() する
2. tokio::time::sleep(Duration::from_millis(50)) で完了を待つ
3. results Signal をクリアする
4. SearchStatus::Idle にリセットする
5. SearchTrigger を新しいモードに更新する
上記すべてを use_coroutine 内で実行し、UIスレッドをブロックしない。

## search/engine.rs の変更
- CancellationToken のキャンセルを検知したら速やかにループを抜ける
- チャンネル送信は cancelled チェックを行ってから実行する

## 完了条件
- Enter → Live 切替時に UI がフリーズしないこと
- Live → Enter 切替時に UI がフリーズしないこと
- 検索中に切替しても安全に動作すること
```

### 完了条件

- Enter ↔ Live 切替時に UI がフリーズしないこと
- 検索中に切替しても安全に動作すること
- 切替後に新モードで正しく検索が動作すること

---

## フェーズ 10 — 長時間検索の待ち表示

**🎯 ゴール:** 検索開始から5秒超過でスピナーと待ちメッセージを表示する

### 追加・変更対象ファイル

- `src/state.rs` — `SearchStatus::Running` に `elapsed_ms` フィールドを追加
- `src/search/engine.rs` — 経過時間を定期送信する仕組みを追加
- `src/components/status_bar.rs` — スピナーと待ちメッセージの表示ロジック追加

### Claude Code プロンプト

```
フェーズ6の成果物に対して、長時間検索の待ち表示を実装してください。
既存のコードを上書き修正する形で対応してください。

## state.rs の変更
SearchStatus::Running に elapsed_ms を追加する（フェーズ8で追加済みの場合はスキップ）。

Running { scanned: usize, matched: usize, elapsed_ms: u64 }

## search/engine.rs の変更
検索タスク内で 100ms ごとに elapsed_ms を更新して Signal に送信する。
tokio::time::interval(Duration::from_millis(100)) を使い、
検索ループと select! で並走させる。

## components/status_bar.rs の変更
elapsed_ms の値に応じて表示を切り替える。

elapsed_ms < 5000:
  「検索中... X件マッチ / Yファイルスキャン済み」

elapsed_ms >= 5000:
  スピナーアニメーション（⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ を 100ms ごとに切替）
  + 「検索中です。しばらくお待ちください... (Xs)」
  ※ Xは経過秒数（整数）

## スピナーの実装
use_signal で frame_index: usize を持ち、
use_future で 100ms ごとに frame_index をインクリメントする。
SPINNER_FRAMES[frame_index % SPINNER_FRAMES.len()] を表示する。

## 完了条件
- 検索開始から5秒未満は通常の進捗表示であること
- 5秒以上でスピナーと待ちメッセージに切り替わること
- 経過秒数が更新され続けること
- 検索完了・中断時にスピナーが消えること
```

### 完了条件

- 5秒未満: 通常の進捗表示（件数・スキャン数）
- 5秒以上: スピナー + 待ちメッセージに切り替わること
- 検索完了・中断でスピナーが消えること

---

## フェーズ 11 — CSV/TSV エクスポート機能

**🎯 ゴール:** 検索結果を CSV または TSV ファイルとして保存できる

### 追加・変更対象ファイル

- `src/export.rs` — CSV/TSV 書き出しロジック（新規）
- `src/components/toolbar.rs` — エクスポートボタンを追加
- `Cargo.toml` — `csv = "1"` を追加

### Claude Code プロンプト

```
フェーズ6の成果物に対して、検索結果のCSV/TSVエクスポート機能を追加してください。
既存のコードを上書き修正する形で対応してください。

## Cargo.toml の変更
[dependencies] に以下を追加する。
csv = "1"

## export.rs（新規作成）
以下の関数を実装する。

pub enum ExportFormat { Csv, Tsv }

pub fn export_results(
    results: &[SearchResult],
    path: &Path,
    format: ExportFormat,
) -> Result<(), Box<dyn std::error::Error>>

- ExportFormat::Csv → カンマ区切り、拡張子 .csv
- ExportFormat::Tsv → タブ区切り、拡張子 .tsv
- 出力カラム: ファイルパス / 行番号 / マッチ内容
- ヘッダー行あり: "file_path","line_number","content"
- ファイルパスは p.display().to_string() で OS に合わせた形式で出力
- csv クレートの WriterBuilder を使い delimiter を切り替える

## components/toolbar.rs の変更
検索ボタンの隣に「エクスポート▼」ボタンを追加する。
- SearchStatus::Done のときのみ活性
- クリックでドロップダウンを表示し「CSV で保存」「TSV で保存」を選択
- 選択後に rfd::FileDialog::save_file() でパスを取得
- export_results() を呼び出して保存
- 保存成功 → ステータスバーに「エクスポート完了: <パス>」を一時表示（3秒）
- 保存失敗 → SearchStatus::Error にエラーメッセージをセット

## テスト（export.rs）
- CSV 出力の内容が正しいことを確認するユニットテスト
- TSV 出力の内容が正しいことを確認するユニットテスト

## 完了条件
- 「エクスポート▼」ボタンが Done 時のみ活性になること
- CSV / TSV それぞれで正しいファイルが保存されること
- ファイルにヘッダー行とデータ行が含まれること
- cargo test でエクスポートのユニットテストが通ること
```

### 完了条件

- 検索完了後に「エクスポート▼」ボタンが活性になること
- CSV / TSV 形式で正しくファイルが保存されること
- ヘッダー行 + データ行が含まれること
- `cargo test` でユニットテストが通ること

---

## フェーズ 12 — Enter 検索完了後の Live 切替フリーズ解消

**🎯 ゴール:** Enter 検索が `Done` 状態になった後に Live トグルを押してもフリーズしない

### フェーズ9との違い

| | フェーズ9 | フェーズ12 |
|--|---------|----------|
| 発生タイミング | 検索**中**に切替 | 検索**完了後**に切替 |
| 原因 | 実行中タスクが Signal をブロック | `Done` 後も残留する受信チャンネル・タスクハンドルのクリーンアップ漏れ |
| 対処 | `CancellationToken` でキャンセル | 完了時に確実にリソースを破棄する |

### 追加・変更対象ファイル

- `src/search/engine.rs` — 検索完了時のチャンネル・タスクハンドルの明示的クローズ
- `src/app.rs` — `SearchStatus::Done` 遷移時のリソース解放処理
- `src/components/options_bar.rs` — トグルハンドラで Done 状態を考慮したクリーンアップ追加

### Claude Code プロンプト

```
フェーズ6・9の成果物に対して、
「Enter検索が完了（Done）した後にLiveトグルをクリックするとフリーズする」
問題を修正してください。既存のコードを上書き修正する形で対応してください。

## 問題の再現手順
1. Enter モードで検索を実行し Done になるまで待つ
2. Live トグルをクリックする
3. → UI がフリーズする

## 原因の調査ポイント
以下を確認し、該当箇所を修正する。
- SearchStatus::Done 後も mpsc::Receiver がドロップされず残留していないか
- 完了したタスクの JoinHandle が保持されたままになっていないか
- トグル切替ハンドラが Done 状態を考慮せずに既存チャンネルを再利用していないか

## search/engine.rs の変更
- 検索完了時（SearchStatus::Done 送信後）に Sender を明示的にドロップする
- タスク終了後に JoinHandle を適切に破棄する

## app.rs の変更
- SearchStatus が Done に遷移したタイミングで以下を実施する
  1. Receiver をドロップしてチャンネルを完全に閉じる
  2. 保持中の JoinHandle があれば破棄する
  3. CancellationToken を新しいインスタンスに差し替える

## components/options_bar.rs の変更
トグル切替ハンドラを以下のフローに統一する（Done 状態も含めて対応）。
1. 現在の SearchStatus を確認する
2. Running の場合 → CancellationToken を cancel()
3. Done / Cancelled / Error の場合 → リソースが解放済みか確認し未解放なら解放
4. tokio::time::sleep(Duration::from_millis(50)) で非同期待機
5. results Signal をクリアする
6. SearchStatus::Idle にリセットする
7. SearchTrigger を新しいモードに更新する
上記すべてを use_coroutine 内で実行し、UI スレッドをブロックしない。

## 確認手順
以下のシナリオをすべて試してフリーズしないことを確認する。
- Enter 検索 → Done → Live トグル → Live 検索できること
- Enter 検索 → Done → Enter トグル → Enter 検索できること
- Live 検索中 → Enter トグル → Enter 検索できること（フェーズ9の回帰確認）
- 中断（Cancelled）→ Live トグル → Live 検索できること

## 完了条件
- Enter 検索完了後に Live トグルを押してもフリーズしないこと
- 切替後に Live 検索が正常に動作すること
- フェーズ9で対処済みの「検索中切替」も引き続き正常に動作すること
```

### 完了条件

- Enter 検索 `Done` 後に Live トグルを押してもフリーズしないこと
- 切替後に Live 検索が正常に動作すること
- `Running` / `Done` / `Cancelled` / `Error` 全状態からの切替が安全に動作すること
- フェーズ9の回帰テスト（検索中切替）が引き続き通ること

---

## フェーズ 13 — Enter/Live 切替時のリセット＆再検索による確実なフリーズ解消

**🎯 ゴール:** 切替時に結果をリセットして初期状態に戻し、新モードで検索を再開することでフリーズを根本解消する

### フェーズ12との違い

| | フェーズ12 | フェーズ13 |
|--|-----------|-----------|
| アプローチ | 残留リソースのクリーンアップ（技術的修正） | 切替時に状態を完全リセットして再検索（UX仕様変更） |
| 対象状態 | Done 後の切替を追加対処 | Running / Done / Cancelled / Error **全状態を統一処理** |
| 再検索 | しない（Idle に戻すだけ） | **クエリが入力済みなら即座に再検索を開始** |
| フリーズ原因 | リソース残留 | **状態に依らず常に新しいタスクとして起動するため発生しない** |

### 追加・変更対象ファイル

- `src/components/options_bar.rs` — トグルハンドラを「リセット＆再検索」フローに全面置換
- `src/app.rs` — `trigger_search()` を切替ハンドラから呼び出せるよう切り出し
- `src/search/engine.rs` — 変更なし（既存の CancellationToken の仕組みをそのまま利用）

### Claude Code プロンプト

```
フェーズ12の成果物に対して、Enter/Live 切替時のフリーズを
「リセット＆再検索」方針で確実に解消してください。
既存のコードを上書き修正する形で対応してください。

## 方針
切替時は以下を必ず順番に実行する。
どの SearchStatus 状態（Running / Done / Cancelled / Error / Idle）から
切替が発生しても同じフローを通るよう統一する。

切替時の統一フロー:
1. 既存の CancellationToken を cancel() する（状態に関わらず常に実行）
2. 既存の JoinHandle / Receiver を破棄して新しいインスタンスに差し替える
3. results Signal を空にクリアする
4. SearchStatus::Idle にリセットする
5. SearchTrigger を新しいモードに更新する
6. query が空でなければ即座に新モードで検索を再開する
   - OnEnter に切替 → 検索は開始しない（ユーザーが Enter を押すまで待機）
   - Realtime に切替 → debounce なしで即座に検索を開始する
上記すべてを use_coroutine 内で実行し、UI スレッドをブロックしない。

## app.rs の変更
検索開始ロジックを trigger_search() として関数に切り出す。
- 既存の CancellationToken を cancel() して新しいトークンを生成
- results をクリア、SearchStatus::Running にセット
- engine::search() をバックグラウンドで起動
- この関数を「検索ボタン押下」「Enter キー」「debounce 完了」「切替後の再検索」
  の4箇所から共通で呼び出せるようにする

## components/options_bar.rs の変更
トグル切替ハンドラを以下のように全面置換する。

let handle = use_coroutine(|_| async move {
    // 1. キャンセル
    cancel_token.cancel();
    // 2. リソース差し替え（新しい CancellationToken を生成して AppState に格納）
    // 3. results クリア
    // 4. SearchStatus::Idle
    // 5. SearchTrigger 更新
    // 6. Realtime に切替かつ query が空でなければ trigger_search() を呼ぶ
});

## 確認手順
以下のシナリオをすべて試してフリーズしないことを確認する。
- Enter 検索中 → Live トグル → Live 検索が再開されること
- Enter 検索完了（Done）→ Live トグル → Live 検索が再開されること
- Enter 検索中断（Cancelled）→ Live トグル → Live 検索が再開されること
- Live 検索中 → Enter トグル → Idle になり Enter 待機になること
- Live 検索完了（Done）→ Enter トグル → Idle になること
- query が空の状態で切替 → Idle になるだけで検索は開始しないこと

## 完了条件
- 全状態からの切替でフリーズが発生しないこと
- Realtime 切替後にクエリがあれば即座に検索が再開されること
- OnEnter 切替後は Idle になり Enter 待機になること
- trigger_search() が4箇所から共通利用されていること
```

### 完了条件

- Running / Done / Cancelled / Error / Idle 全状態からの切替でフリーズしないこと
- Realtime 切替後、クエリが入力済みなら即座に検索が再開されること
- OnEnter 切替後は Idle になり Enter 待機になること
- フェーズ8〜12の既存機能（中断・待ち表示・エクスポート）が引き続き正常に動作すること

---

## フェーズ 14 — Live切替後の中断ボタンフリーズ解消

**🎯 ゴール:** Live トグルに切り替えた後に中断ボタンを押してもフリーズしない

### 推定原因

フェーズ13で `trigger_search()` を共通化した際、Realtime 切替後の即時再検索で生成した**新しい `CancellationToken` が `AppState` に書き戻される前**に中断ボタンが押されると、ボタンが古い（すでに cancel 済みの）トークンに対して `cancel()` を呼んでしまい、実行中タスクを停止できないままチャンネルの送受信が噛み合わなくなってフリーズする。

### フェーズ8・13との違い

| | フェーズ8 | フェーズ13 | フェーズ14 |
|--|---------|-----------|-----------|
| 対象 | 中断機能の初期実装 | 切替時リセット＆再検索 | 切替後の中断が古いトークンを参照する問題 |
| 原因 | 中断機能未実装 | リソース差し替えフロー未整備 | トークン差し替えと AppState 反映のタイミングずれ |
| 対処 | CancellationToken を UI から操作 | 統一フローで差し替え | トークン生成→AppState 反映→検索起動の順序を保証 |

### 追加・変更対象ファイル

- `src/app.rs` — `trigger_search()` 内のトークン生成・反映・検索起動の順序を修正
- `src/components/toolbar.rs` — 中断ボタンが常に AppState の**最新**トークンを参照するよう修正
- `src/components/options_bar.rs` — 切替ハンドラ内の `trigger_search()` 呼び出しタイミングを修正

### Claude Code プロンプト

```
フェーズ13の成果物に対して、
「Live トグルに切り替えた後に中断ボタンを押すとフリーズする」問題を修正してください。
既存のコードを上書き修正する形で対応してください。

## 問題の再現手順
1. クエリを入力した状態で Live トグルに切り替える
2. Realtime 検索が自動開始される
3. 検索中に中断ボタンを押す
4. → UI がフリーズする

## 原因
trigger_search() 内で以下の順序になっている場合に発生する。

【NG な順序】
1. new_token = CancellationToken::new()  // 新トークン生成
2. engine::search(new_token.clone()) 起動 // 先に検索タスクを起動
3. app_state.cancel_token = new_token    // 後から AppState に反映 ← ここがずれる

中断ボタンは app_state.cancel_token.cancel() を呼ぶが、
ステップ2と3の間に押されると古いトークンまたは未反映のトークンを参照してしまう。

## app.rs の修正
trigger_search() 内のトークン差し替えと検索起動の順序を以下に統一する。

【正しい順序】
1. old_token.cancel()                      // 既存タスクをキャンセル
2. new_token = CancellationToken::new()    // 新トークン生成
3. app_state.cancel_token = new_token.clone() // 先に AppState に反映 ← ここを先に
4. app_state.status = SearchStatus::Running { ... }
5. engine::search(new_token) 起動          // 最後に検索タスクを起動

これにより、検索タスク起動前に AppState が新トークンを持つことが保証される。

## components/toolbar.rs の修正
中断ボタンの onclick ハンドラを以下のように修正する。
- cancel_token を変数にキャプチャするのではなく、
  クリック時に毎回 app_state.cancel_token を読み取って cancel() する
- これにより「クリック時点の最新トークン」を常に参照できる

【NG】
let token = app_state.cancel_token.clone(); // レンダリング時にキャプチャ
button { onclick: move |_| token.cancel() } // 古いトークンを参照する可能性

【OK】
button {
    onclick: move |_| {
        app_state.read().cancel_token.cancel() // クリック時に毎回読み取る
    }
}

## components/options_bar.rs の修正
切替ハンドラ内で trigger_search() を呼ぶ箇所は、
SearchTrigger の AppState 反映が完了した後に呼ぶことを明示的にコメントで示す。
処理順序は以下を厳守する。
1. cancel() → リソース破棄 → results クリア → Idle リセット → SearchTrigger 更新
2. （ここで AppState の更新が完了している）
3. Realtime かつ query が空でなければ trigger_search() を呼ぶ

## 確認手順
以下のシナリオをすべて試してフリーズしないことを確認する。
- Enter 検索中 → 中断ボタン → フリーズしないこと（フェーズ8の回帰確認）
- Live 検索中（最初から Live）→ 中断ボタン → フリーズしないこと
- Enter 検索完了 → Live トグル → 自動検索開始 → 中断ボタン → フリーズしないこと ← 主目的
- Enter 検索完了 → Live トグル → 中断 → 再度 Live 検索 → 中断 → 繰り返しても安定すること
- 中断後に Enter トグルに戻して Enter 検索 → 中断 → フリーズしないこと

## 完了条件
- Live 切替後の中断ボタンでフリーズしないこと
- 中断→再検索→中断を繰り返しても安定して動作すること
- フェーズ8（Enter 検索中の中断）が引き続き正常に動作すること
- フェーズ13（切替時リセット＆再検索）が引き続き正常に動作すること
```

### 完了条件

- Live 切替後の中断ボタンでフリーズしないこと
- 中断 → 再検索 → 中断の繰り返しが安定して動作すること
- `AppState` のトークン反映が検索タスク起動より必ず先に行われること
- フェーズ8・13の既存機能が引き続き正常に動作すること

---

## フェーズ 15 — フリーズ問題の包括的解消（SearchEngine による責務集約）

**🎯 ゴール:** トークン管理・状態遷移・検索起動の責務を `SearchEngine` に集約し、あらゆる操作順序でフリーズが発生しない構造に全面整理する

### これまでの対応履歴と根本問題

| フェーズ | 対処内容 | 残った問題 |
|---------|---------|----------|
| 9 | 検索中切替を `use_coroutine` で非同期化 | Done 後の切替は未対処 |
| 12 | Done 後のリソースクリーンアップ追加 | 状態ごとに分岐が増えて漏れが発生 |
| 13 | 統一フロー＋`trigger_search()` 切り出し | トークン反映とタスク起動のタイミングずれ |
| 14 | トークン順序修正＋クリック時読み取り | フェーズ13の統一フローを部分上書きして12の問題が再発 |

**根本問題:** フェーズごとの部分修正でトークン管理・状態遷移・検索起動の責務が `app.rs` / `toolbar.rs` / `options_bar.rs` / `engine.rs` に分散し、整合性が取れなくなっている。

### 設計方針

`SearchEngine` 構造体をフロントエンドとバックグラウンドの唯一の窓口として定義し、外部からは `start()` / `cancel()` / `switch_trigger()` の3つのメソッドだけを呼ぶ構造にする。内部のトークン管理・タスクハンドル・チャンネルはすべて `SearchEngine` が所有し、外部から直接触れない。

```
【変更前: 責務が分散】
app.rs         → trigger_search() でトークン生成・反映・タスク起動
toolbar.rs     → app_state からトークンを読んで cancel()
options_bar.rs → 切替ハンドラでトークン差し替え＋trigger_search() 呼び出し
engine.rs      → 検索ロジック

【変更後: SearchEngine に集約】
search_engine.rs → start() / cancel() / switch_trigger() のみ公開
app.rs           → SearchEngine を AppState に持たせて各コンポーネントに渡す
toolbar.rs       → engine.start() / engine.cancel() を呼ぶだけ
options_bar.rs   → engine.switch_trigger(new_mode) を呼ぶだけ
```

### 追加・変更対象ファイル

- `src/search/search_engine.rs` — 新規作成。責務を集約した構造体
- `src/state.rs` — `AppState` に `SearchEngine` を持たせる
- `src/app.rs` — `trigger_search()` を削除し `engine.start()` に一本化
- `src/components/toolbar.rs` — `engine.start()` / `engine.cancel()` を呼ぶだけに簡略化
- `src/components/options_bar.rs` — `engine.switch_trigger()` を呼ぶだけに簡略化

### Claude Code プロンプト

```
フェーズ9〜14の成果物に対して、フリーズ問題を包括的に解消してください。
部分修正ではなく、以下の設計に従って SearchEngine 構造体を新規作成し、
関連ファイルをすべて整合性が取れる状態に全面整理してください。

## 設計原則
- CancellationToken / JoinHandle / mpsc::Receiver の所有権は
  SearchEngine だけが持つ。外部から直接触れない。
- 外部に公開するインターフェースは start() / cancel() / switch_trigger() の3つのみ。
- SearchStatus の更新は SearchEngine 内部で完結させ、
  Signal 経由で UI に通知する。
- どのメソッドも非同期で完結し、呼び出し元（UI スレッド）を一切ブロックしない。

## src/search/search_engine.rs（新規作成）

pub struct SearchEngine {
    status: Signal<SearchStatus>,
    results: Signal<Vec<SearchResult>>,
    cancel_token: Arc<Mutex<CancellationToken>>,  // Mutex で保護して競合を防ぐ
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl SearchEngine {
    pub fn new(
        status: Signal<SearchStatus>,
        results: Signal<Vec<SearchResult>>,
    ) -> Self { ... }

    /// 検索を開始する。実行中の検索があれば先にキャンセルして置き換える。
    pub fn start(&self, query: String, path: PathBuf, options: SearchOptions) {
        // 1. 既存トークンを cancel()（Mutex ロックして安全に）
        // 2. 新しい CancellationToken を生成
        // 3. cancel_token を新トークンに差し替え（Mutex ロック解放前に完了）
        // 4. results を空にクリア
        // 5. status を Running にセット
        // 6. 新トークンを持つ検索タスクを tokio::spawn で起動
        // 7. task_handle を新しい JoinHandle に差し替え
        // ※ 3を必ず6より先に完了させることでタイミングずれを防ぐ
    }

    /// 実行中の検索を中断する。
    pub fn cancel(&self) {
        // 1. Mutex ロックして cancel_token.cancel() を呼ぶ
        // 2. status を Cancelled にセット
        // ※ task_handle は abort しない（CancellationToken で自然終了させる）
    }

    /// 検索トリガーモードを切り替える。
    /// Running 中でも Done 後でも安全に呼び出せる。
    pub fn switch_trigger(
        &self,
        new_trigger: SearchTrigger,
        trigger_signal: &mut Signal<SearchTrigger>,
        query: &str,
        path: &Path,
        options: &SearchOptions,
    ) {
        // 1. self.cancel() を呼んで現在のタスクを停止（状態に関わらず常に実行）
        // 2. results をクリア
        // 3. status を Idle にセット
        // 4. trigger_signal を新モードに更新
        // 5. new_trigger が Realtime かつ query が空でなければ self.start() を呼ぶ
        //    new_trigger が OnEnter なら Idle のまま待機
        // ※ 1〜5をすべて同期的に完了させてから戻る（async 不要）
        //   cancel() 内部は非ブロッキングなので UI スレッドをブロックしない
    }
}

## src/state.rs の変更
AppState に SearchEngine を追加する。

pub struct AppState {
    pub query: Signal<String>,
    pub search_path: Signal<PathBuf>,
    pub options: Signal<SearchOptions>,
    pub ui_settings: Signal<UiSettings>,
    pub results: Signal<Vec<SearchResult>>,
    pub status: Signal<SearchStatus>,
    pub engine: SearchEngine,  // ← 追加
}

## src/app.rs の変更
- trigger_search() を削除する
- AppState 初期化時に SearchEngine::new(status, results) を生成して持たせる
- 検索起動は engine.start() 呼び出しに一本化する

## src/components/toolbar.rs の変更
- 検索ボタン: app_state.engine.start(query, path, options) を呼ぶ
- 中断ボタン: app_state.engine.cancel() を呼ぶ
- トークンの直接参照をすべて削除する

## src/components/options_bar.rs の変更
- トグル切替ハンドラ: app_state.engine.switch_trigger(new_mode, ...) を呼ぶ
- use_coroutine / tokio::time::sleep による待機処理をすべて削除する
  （SearchEngine 内部が非ブロッキングで処理するため不要）

## 確認手順
以下のシナリオをすべて試してフリーズしないことを確認する。
- Enter 検索中 → 中断 → フリーズしないこと
- Enter 検索完了（Done）→ Live トグル → 自動検索開始 → フリーズしないこと  ← 主目的
- Enter 検索完了（Done）→ Live トグル → 中断 → フリーズしないこと
- Live 検索中 → Enter トグル → Idle になること
- Live 検索中 → 中断 → フリーズしないこと
- 中断 → Live トグル → 自動検索開始 → 中断 → 繰り返しても安定すること
- Enter 検索中 → Live トグル → 自動検索開始 → 中断 → フリーズしないこと
- query が空の状態で Live トグル → Idle のままで検索が起動しないこと

## 完了条件
- 全シナリオでフリーズが発生しないこと
- CancellationToken の直接参照が toolbar.rs / options_bar.rs から消えていること
- SearchEngine の外部インターフェースが start() / cancel() / switch_trigger() のみであること
- cargo check がエラーなく通ること
- フェーズ8〜14の既存機能（待ち表示・エクスポート等）が引き続き正常に動作すること
```

### 完了条件

- 全操作順序・全状態遷移でフリーズが発生しないこと
- `CancellationToken` の直接参照が UI コンポーネントから完全に消えていること
- `SearchEngine` の外部 API が `start()` / `cancel()` / `switch_trigger()` の3つのみであること
- `cargo check` がエラーなく通ること
- フェーズ8〜14の既存機能が引き続き正常に動作すること

---

## フェーズ 16 — Live検索機能の廃止

**🎯 ゴール:** Live（Realtime）検索機能を完全に削除し、Enter 検索のみのシンプルな構成に整理する

### 削除・変更対象ファイル

- `src/state.rs` — `SearchTrigger` enum / `UiSettings.search_trigger` / `UiSettings.debounce_ms` を削除
- `src/search/search_engine.rs` — `switch_trigger()` メソッド / debounce 関連処理を削除
- `src/components/options_bar.rs` — Enter/Live トグルボタンを削除
- `src/components/toolbar.rs` — Live モード時の SearchButton 非活性ロジックを削除
- `src/components/status_bar.rs` — Live モード関連の表示分岐を削除
- `src/app.rs` — `SearchTrigger` / debounce 関連の初期化・処理を削除

### Claude Code プロンプト

```
フェーズ15の成果物に対して、Live（Realtime）検索機能を完全に廃止してください。
削除後は Enter 検索のみが残るシンプルな構成に整理してください。

## 削除する機能
- SearchTrigger enum（OnEnter / Realtime）
- UiSettings.search_trigger フィールド
- UiSettings.debounce_ms フィールド
- SearchEngine.switch_trigger() メソッド
- debounce 処理（tokio::time::sleep による待機）
- options_bar.rs の Enter/Live トグルボタン
- Live モード時の SearchButton 非活性ロジック

## 残す機能（変更しない）
- Enter キー / 検索ボタンによる検索起動
- SearchEngine.start() / cancel()
- 中断ボタン（フェーズ8）
- 待ち表示・スピナー（フェーズ10）
- CSV/TSV エクスポート（フェーズ11）
- Tree/Flat 表示切り替え（フェーズ5）

## 各ファイルの修正内容

### src/state.rs
- SearchTrigger enum を削除する
- UiSettings から search_trigger / debounce_ms フィールドを削除する
- SearchTrigger を参照している箇所をすべて削除する

### src/search/search_engine.rs
- switch_trigger() メソッドを削除する
- debounce 関連のコード（tokio::time::sleep / interval 等）を削除する
- 外部 API は start() / cancel() の2つのみにする

### src/components/options_bar.rs
- Enter/Live トグルボタンを削除する
- SearchTrigger に関連する Signal / ハンドラをすべて削除する
- 残るのは以下の3つのチェックボックスのみとする
  - 正規表現
  - 大文字小文字区別
  - バイナリ含む

### src/components/toolbar.rs
- Live モード時に SearchButton を非活性にするロジックを削除する
- SearchButton は常に活性とする
- 中断ボタンの切り替えロジック（Running 時のみ表示）はそのまま残す

### src/app.rs
- SearchTrigger の初期化を削除する
- debounce 関連の処理を削除する

## 完了条件
- cargo check がエラーなく通ること
- SearchTrigger に関するコードがプロジェクト全体から消えていること
- Enter キー / 検索ボタンで検索が正常に動作すること
- 中断ボタン・待ち表示・エクスポートが引き続き正常に動作すること
- options_bar に Enter/Live トグルが表示されないこと
```

### 完了条件

- `SearchTrigger` に関するコードがプロジェクト全体から消えていること
- `cargo check` がエラーなく通ること
- Enter キー / 検索ボタンで検索が正常に動作すること
- フェーズ8・10・11の機能（中断・待ち表示・エクスポート）が引き続き正常に動作すること

---

## フェーズ 17 — 検索結果のクリア機能

**🎯 ゴール:** ボタン1つで検索結果・クエリ・ステータスをすべて初期状態に戻せる

### フェーズ8（中断）との違い

| | フェーズ8 中断 | フェーズ17 クリア |
|--|-------------|----------------|
| 対象 | 実行中タスクの停止 | 結果・クエリ・状態の全消去 |
| 実行タイミング | Running 中のみ | **どの状態でも実行可能** |
| 検索タスク | 停止する | 実行中なら停止してから消去 |
| クエリ入力欄 | そのまま | **空にクリア** |
| 検索パス | そのまま | そのまま（パスは維持） |
| 結果後の状態 | Cancelled | **Idle** |

### 追加・変更対象ファイル

- `src/search/search_engine.rs` — `clear()` メソッドを追加
- `src/components/toolbar.rs` — クリアボタンを追加
- `src/state.rs` — 変更なし

### Claude Code プロンプト

```
フェーズ16の成果物に対して、検索結果のクリア機能を追加実装してください。

## search/search_engine.rs の変更
clear() メソッドを追加する。

pub fn clear(&self, query: &mut Signal<String>) {
    // 1. 実行中タスクがあれば cancel() を呼ぶ
    // 2. results Signal を空にクリアする
    // 3. status Signal を SearchStatus::Idle にリセットする
    // 4. query Signal を空文字列にクリアする
    // ※ 検索パス（search_path）はクリアしない
}

## components/toolbar.rs の変更
検索ボタンの隣にクリアボタンを追加する。

- ボタンラベル: 「✕ クリア」
- 活性条件: query が空でない、または results が1件以上、または status が Idle 以外
  （すべて空・Idle の場合のみ非活性）
- onclick: engine.clear(&mut query) を呼ぶ
- 配置: 検索ボタン（または中断ボタン）の右隣

## 完了条件
- 「✕ クリア」ボタンが表示されること
- クリック後にクエリ入力欄が空になること
- クリック後に検索結果が消えること
- クリック後に StatusBar が「待機中」になること
- Running 中にクリックすると検索が停止されてからクリアされること
- 検索パスはクリアされないこと
- cargo check がエラーなく通ること
```

### 完了条件

- どの状態（Idle / Running / Done / Cancelled / Error）からでもクリアが動作すること
- クリア後にクエリ・結果・ステータスがすべて初期状態になること
- 検索パスはクリアされないこと
- `cargo check` がエラーなく通ること

---

## フェーズ 18 — 検索の中止機能

**🎯 ゴール:** 実行中の検索を停止し、結果・ステータスを完全リセットして Idle に戻す

### フェーズ8（中断）・フェーズ17（クリア）との違い

| | フェーズ8 中断 | フェーズ17 クリア | フェーズ18 中止 |
|--|-------------|----------------|--------------|
| 検索タスク停止 | ✅ する | 実行中なら停止 | ✅ する |
| 結果の消去 | ❌ 残る | ✅ 消える | ✅ 消える |
| クエリの消去 | ❌ 残る | ✅ 消える | ❌ **残る** |
| 最終ステータス | Cancelled | Idle | **Idle** |
| 実行可能タイミング | Running 中のみ | どの状態でも | **Running 中のみ** |
| ユースケース | 途中結果を確認したい | 最初からやり直したい | **同じクエリで再検索したい** |

つまり中止は「クエリは残したまま結果だけリセットして再検索できる状態に戻す」操作。

### 追加・変更対象ファイル

- `src/search/search_engine.rs` — `stop()` メソッドを追加
- `src/components/toolbar.rs` — Running 時のボタン表示を「中断」「中止」の2択に変更

### Claude Code プロンプト

```
フェーズ17の成果物に対して、検索の中止機能を追加実装してください。

## search/search_engine.rs の変更
stop() メソッドを追加する。

pub fn stop(&self) {
    // 1. cancel_token.cancel() を呼んで実行中タスクを停止する
    // 2. results Signal を空にクリアする
    // 3. status Signal を SearchStatus::Idle にリセットする
    // ※ query はクリアしない（クリアはフェーズ17の clear() の責務）
    // ※ 中断（cancel）との違いは results をクリアして Idle に戻す点
}

## SearchEngine の外部 API 整理（フェーズ16以降の全メソッド）
- start()  : 検索開始
- cancel() : 中断（結果を残して Cancelled）← フェーズ8、変更なし
- clear()  : クリア（クエリ・結果・状態を全消去して Idle）← フェーズ17、変更なし
- stop()   : 中止（クエリを残して結果・状態をリセットして Idle）← 今回追加

## components/toolbar.rs の変更
Running 時のボタン表示を以下の2つに変更する。

【Idle / Done / Cancelled / Error 時】
「🔍 検索」ボタン（活性） ｜ 「✕ クリア」ボタン

【Running 時】
「⏸ 中断」ボタン  ｜  「⏹ 中止」ボタン  ｜  「✕ クリア」ボタン（非活性）

- 「⏸ 中断」: engine.cancel() を呼ぶ（結果を残して Cancelled）
- 「⏹ 中止」: engine.stop() を呼ぶ（結果リセットして Idle、クエリ維持）
- 「✕ クリア」: Running 中は非活性（誤操作防止）

## status_bar.rs の変更
SearchStatus::Idle の表示を状態の来歴で出し分ける。
- 初期起動時の Idle       → 「待機中」
- stop() 後の Idle        → 「中止しました。検索ボタンで再検索できます。」
- clear() 後の Idle       → 「待機中」（clear は query もリセットするため同じ表示でよい）
上記の出し分けのために AppState に前回操作を示す
LastAction enum を追加してもよい。

pub enum LastAction { None, Cancelled, Stopped, Cleared }

## 確認手順
- 検索中に「⏸ 中断」→ 結果が残り「中断しました」が表示されること（フェーズ8の回帰）
- 検索中に「⏹ 中止」→ 結果が消えクエリは残り「中止しました...」が表示されること
- 中止後に「🔍 検索」→ 同じクエリで再検索が開始されること
- 検索中に「✕ クリア」→ ボタンが非活性で操作できないこと
- Done 後に「✕ クリア」→ クエリ・結果・状態がすべてリセットされること（フェーズ17の回帰）

## 完了条件
- Running 中に「⏸ 中断」「⏹ 中止」の2ボタンが表示されること
- 中止後にクエリが残り結果が消えて Idle になること
- 中止後に再検索できること
- cargo check がエラーなく通ること
```

### 完了条件

- Running 中に「⏸ 中断」「⏹ 中止」の2ボタンが表示されること
- 中止後にクエリが残り、結果が消えて Idle になること
- 中止後に同じクエリで再検索できること
- フェーズ8（中断）・フェーズ17（クリア）が引き続き正常に動作すること
- `cargo check` がエラーなく通ること

---

## フェーズ 19 — ステータスバーのリアルタイム性向上

**🎯 ゴール:** 検索中のスキャン件数・マッチ件数・経過時間が滑らかにリアルタイム更新される

### 現状の問題と原因

| 問題 | 原因 |
|------|------|
| 件数表示の更新が遅い・飛び飛び | `scanned` / `matched` の更新が結果チャンネルの受信タイミング依存 |
| 経過時間の表示がカクつく | スピナー更新（100ms）と件数更新が非同期で別タイミング |
| 大量ファイル時に長時間 0件表示 | バックグラウンド処理が重くてチャンネル送信まで時間がかかる |
| Cancelled / Done の表示が遅れる | 最後の結果受信後にステータス更新が走るため |

### 設計方針

ステータス更新専用の tick ループを `SearchEngine` 内に独立して持たせ、検索結果の受信とは分離する。件数はアトミックカウンタで管理し、tick ループが一定間隔で読み取って Signal を更新する。

```
【変更前】
検索タスク → チャンネル → 結果受信 → results 更新 → status 更新（まとめて）

【変更後】
検索タスク → Arc<AtomicUsize> カウンタをインクリメント（ブロックなし）
           → チャンネル → 結果受信 → results 更新のみ
tick ループ（50ms）→ カウンタを読んで status Signal を更新（独立）
```

### 追加・変更対象ファイル

- `src/search/search_engine.rs` — アトミックカウンタ導入・tick ループ追加
- `src/search/engine.rs` — 結果送信とカウンタ更新を分離
- `src/components/status_bar.rs` — スピナーと件数を同一 tick で更新

### Claude Code プロンプト

```
フェーズ18の成果物に対して、ステータスバーのリアルタイム性を向上させてください。

## 方針
- 件数カウントを Arc<AtomicUsize> で管理し、検索スレッドからロックなしで更新する
- 50ms ごとの tick ループがカウンタを読んで status Signal を更新する
- tick ループはスピナーのフレーム更新も兼ねる（フェーズ10の use_future を統合）
- 結果チャンネルは results Signal の更新のみに専念させる

## search/search_engine.rs の変更

SearchEngine に以下のフィールドを追加する。

scanned_count: Arc<AtomicUsize>,  // 走査済みファイル数
matched_count: Arc<AtomicUsize>,  // マッチ件数

start() の変更:
- scanned_count / matched_count を 0 にリセットしてから検索タスクを起動する
- 検索タスクには Arc クローンを渡す
- 50ms ごとの tick ループを tokio::spawn で別タスクとして起動する

tick ループの実装:
tokio::time::interval(Duration::from_millis(50)) で定期実行する。
CancellationToken がキャンセルされるか SearchStatus が Done / Cancelled / Idle に
なるまでループを継続する。
各 tick で以下を実行する。
  scanned = scanned_count.load(Ordering::Relaxed)
  matched = matched_count.load(Ordering::Relaxed)
  elapsed_ms = start_time.elapsed().as_millis() as u64
  status.set(SearchStatus::Running { scanned, matched, elapsed_ms })

cancel() / stop() / clear() の変更:
- それぞれの処理の最後に scanned_count / matched_count を 0 にリセットする

## search/engine.rs の変更

検索タスク内の各ファイル処理を以下のように変更する。
- ファイルを走査するたびに scanned_count.fetch_add(1, Ordering::Relaxed)
- マッチ行を発見するたびに matched_count.fetch_add(1, Ordering::Relaxed)
- Sender への送信は結果の Vec<SearchResult> のみとし、
  status の更新を行わない（tick ループに委譲）

## components/status_bar.rs の変更

フェーズ10で実装した use_future によるスピナー更新を削除する。
スピナーのフレーム管理を SearchEngine の tick ループに統合する。

SearchStatus::Running の表示:
- elapsed_ms < 5000:
  「検索中... {matched}件マッチ / {scanned}ファイルスキャン済み ({elapsed_ms}ms)」
- elapsed_ms >= 5000:
  「{spinner} 検索中です。しばらくお待ちください... ({elapsed_s}s)  
   {matched}件マッチ / {scanned}ファイルスキャン済み」

スピナーフレームの管理:
SearchEngine に spinner_frame: Arc<AtomicUsize> を追加し、
tick ループが 100ms ごと（50ms tick の2回に1回）にインクリメントする。
status_bar.rs は SearchStatus から spinner_frame を読んで表示する。
または SearchStatus::Running に spinner_frame: usize フィールドを追加する。

## 完了条件
- 件数表示が 50ms 以下の間隔で更新されること
- スピナーと件数が同じタイミングで更新されてカクつきがないこと
- 大量ファイルのディレクトリでも検索開始直後から件数が増加し始めること
- cancel() / stop() / clear() 後にカウンタが正しくリセットされること
- cargo check がエラーなく通ること
- フェーズ8・10・17・18の既存機能が引き続き正常に動作すること
```

### 完了条件

- スキャン件数・マッチ件数・経過時間が 50ms 以下の間隔で更新されること
- スピナーと件数が同一 tick で更新されてカクつきがないこと
- 大量ファイル時でも検索開始直後から件数が増加し始めること
- `cargo check` がエラーなく通ること
- フェーズ8・10・17・18の既存機能が引き続き正常に動作すること

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