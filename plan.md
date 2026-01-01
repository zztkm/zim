# Zim - Vim風テキストエディタ実装プラン

このプロジェクトは zztkm が実装を行い、Claude Code は Copilot の役割を持ちます。

## プロジェクト概要
Rust で vim 風のキーバインドを持つテキストエディタ「zim」を実装します。
C 言語の参考実装（kilolo.c）をベースに、vim 特有のモード管理を追加します。

## 要件
- **モード**: Normal, Insert, Visual, Command の4モード実装
- **基本機能のみ**: 検索やシンタックスハイライトは後回し
- **1ファイルのみ**: 同時に開けるのは1ファイル
- **段階的実装**: 最小限の機能で動くエディタを早く作る
- **UTF-8 対応**: 日本語などのマルチバイト文字を正しく扱う
- **対応 OS**: macOS / Linux のみ（改行コードは LF `\n` のみサポート）

## アーキテクチャ

### モジュール構成
```
src/
├── main.rs          # エントリーポイント、メインループ
├── editor.rs        # エディタ状態管理と中核ロジック
├── terminal.rs      # ターミナル制御（raw mode、画面描画）
├── buffer.rs        # テキストバッファ管理（行データ、編集操作）
├── mode.rs          # Vim モード管理（Normal/Insert/Visual/Command）
├── keymap.rs        # キーバインディングとコマンド処理
├── cursor.rs        # カーソル移動ロジック
├── screen.rs        # 画面レンダリング
└── file_io.rs       # ファイル読み込み・保存
```

### 依存クレート
```toml
[dependencies]
termion = "4"              # ターミナル制御（raw mode、キー入力、ANSI escape）
anyhow = "1.0"             # エラーハンドリング
thiserror = "2.0"          # カスタムエラー型定義
unicode-width = "0.1"      # 文字の表示幅計算（全角文字対応）
unicode-segmentation = "1" # グラフィームクラスタ単位の文字列操作
```

### 主要データ構造

**Mode 管理**
```rust
enum Mode { Normal, Insert, Visual, Command }

struct ModeManager {
    current: Mode,
    visual_start: Option<(usize, usize)>,
}
```

**Editor 状態**
```rust
struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    mode: ModeManager,
    terminal: Terminal,
    screen: Screen,
    filename: Option<PathBuf>,
    dirty: bool,
    status_message: String,
    command_buffer: String,
    yank_buffer: Vec<String>,
}
```

**Buffer & Row**
```rust
struct Buffer { rows: Vec<Row> }
struct Row { chars: String, render: String }
```

**Cursor**
```rust
struct Cursor {
    x: usize,      // 実際の文字位置（グラフィームクラスタ単位）
    y: usize,      // 行位置
    rx: usize,     // レンダリング位置（タブ・文字幅考慮）
    row_offset: usize,
    col_offset: usize,
}
```

**UTF-8 対応のポイント**:
- グラフィームクラスタ単位でカーソル移動（結合文字や絵文字を1文字として扱う）
- 文字の表示幅を考慮（全角文字は2カラム、半角は1カラム）
- バイト位置とグラフィーム位置を適切に変換

## 段階的実装計画

### フェーズ1: 基本ターミナル操作とNormalモード ✅
**目標**: 空のエディタが起動し、カーソル移動と終了ができる

**ステータス**: 完了

**実装内容**:
1. `terminal.rs`: raw mode 設定、画面クリア
2. `mode.rs`: Mode enum と ModeManager
3. `cursor.rs`: 基本的なカーソル移動（h/j/k/l）
4. `screen.rs`: 空画面に `~` を描画
5. `main.rs`: メインループ、`:q` で終了

**キーバインド**:
- `h/j/k/l`: カーソル移動
- `:q`: 終了

**Critical Files**:
- `src/terminal.rs`
- `src/mode.rs`
- `src/main.rs`

---

### フェーズ2: ファイル読み込みと表示 ✅
**目標**: ファイルを開いて閲覧できる

**ステータス**: 完了

**実装内容**:
1. `buffer.rs`: Row 構造、Buffer 構造、行データ管理
2. `file_io.rs`: ファイル読み込み（行ごとに Buffer に追加）
3. `screen.rs`: Buffer の内容を描画
4. `cursor.rs`: スクロール処理、行数に応じたカーソル制限
5. ステータスバー表示（ファイル名、行数）

**キーバインド**:
- `h/j/k/l`: スクロール対応のカーソル移動
- `0`: 行頭、`$`: 行末
- `gg`: ファイル先頭、`G`: ファイル末尾

**実装ノート**:
- `gg` は2キーストローク処理（`pending_key` で管理）
- スクロール処理は毎回自動的に呼ばれる（`cursor.scroll()`）
- ステータスバーで現在行とファイル名を表示

**Critical Files**:
- `src/buffer.rs`
- `src/file_io.rs`
- `src/screen.rs`
- `src/cursor.rs`
- `src/main.rs`

---

### フェーズ3: Insertモードと基本編集 ✅
**目標**: テキスト編集ができる

**ステータス**: 完了

**実装内容**:
1. `mode.rs`: Insert モードへの遷移処理
2. `buffer.rs`: 文字挿入、文字削除、改行挿入
3. `editor.rs`: dirty フラグ管理
4. Insert モード表示（ステータスバーに "-- INSERT --"）
5. `main.rs`: Insert モードでのキー入力処理

**キーバインド**:
- Normal モードで `i`: Insert モード開始（カーソル位置）
- Normal モードで `I`: 行頭から Insert
- Normal モードで `a`: カーソル後ろから Insert
- Normal モードで `A`: 行末から Insert
- Normal モードで `o`: 下に新しい行を挿入して Insert
- Normal モードで `O`: 上に新しい行を挿入して Insert
- Insert モードで `ESC`: Normal モードへ復帰
- Insert モードで文字入力: 文字挿入
- Insert モードで `Backspace`: 文字削除
- Insert モードで `Enter`: 改行

**実装ノート**:
- Insert モードでは行末+1の位置までカーソル移動可能（`a`, `A` コマンド、文字挿入後）
- 文字挿入後は `cursor.move_right(size.0, line.len() + 1)` で移動（Normal モードの制限を回避）
- 画面描画時に `termion::clear::UntilNewline` で各行末をクリア（古い文字が残らないように）
- コマンドライン描画時に `termion::clear::CurrentLine` でクリア（モード切替時の表示が残らないように）
- `editor.rs` で dirty フラグを管理（文字挿入・削除・改行・行結合時に true に設定）

**修正したバグ**:
1. 文字挿入時のカーソル位置ずれ（"hello" → "elloh" になる問題）
   - 原因: Insert モード後の `move_right` が Normal モードの制限（行末まで）で動作していた
   - 修正: Insert モードでは `line.len() + 1` を渡してカーソルが行末の次まで移動できるようにした
2. 画面に古い文字が残る問題
   - 原因: 各行を描画後に行末をクリアしていなかった
   - 修正: `screen.rs` で `termion::clear::UntilNewline` を追加
3. モード切替時に "-- INSERT --" が消えない問題
   - 原因: コマンドラインを描画前にクリアしていなかった
   - 修正: `screen.rs` の `draw_command_line` で `termion::clear::CurrentLine` を追加

**Critical Files**:
- `src/main.rs`
- `src/buffer.rs`
- `src/editor.rs`
- `src/screen.rs`

---

### フェーズ4: ファイル保存 ✅
**目標**: 編集内容を保存できる

**ステータス**: 完了

**実装内容**:
1. `file_io.rs`: ファイル保存処理
2. `main.rs`: Command モード処理（`:w`, `:wq`, `:q!`）
3. `editor.rs`: 保存メソッドと dirty フラグ管理
4. `screen.rs`: ステータスメッセージ表示機能
5. 未保存時の警告（`:q` で dirty の場合）

**キーバインド**:
- `:w`: 保存
- `:q`: 終了（未保存時は警告）
- `:wq`: 保存して終了
- `:q!`: 強制終了

**実装ノート**:
- ファイル保存時は改行コード LF (`\n`) のみ使用
- 最後の行には改行を付けない（vim の標準動作）
- 保存成功時に "written" メッセージを表示（行数とバイト数）
- 未保存で `:q` 実行時は "No write since last change" 警告
- ステータスメッセージはモード変更時に自動クリア
- `:w` 成功時に dirty フラグをクリア

**追加機能**:
- カーソルスタイルのモード別切り替え
  - Insert モード: 縦棒カーソル (`termion::cursor::SteadyBar`)
  - Normal/Command モード: ブロックカーソル (`termion::cursor::SteadyBlock`)

**Critical Files**:
- `src/file_io.rs`
- `src/editor.rs`
- `src/main.rs`
- `src/screen.rs`
- `src/terminal.rs`

---

### フェーズ4.5: ファイル切り替え (`:e` コマンド) ✅
**目標**: 別のファイルを開く、または現在のファイルを再読み込みできる

**ステータス**: 完了

**実装内容**:
1. `editor.rs`: `open_file` と `reload` メソッド
2. `main.rs`: `:e` コマンドの処理とコマンド引数パース
3. `cursor.rs`: `ensure_within_bounds` メソッド（カーソル範囲調整）
4. 未保存時の警告（`:e` で dirty の場合）
5. 強制実行（`:e!`）

**キーバインド**:
- `:e filename`: 指定したファイルを開く
- `:e`: 現在のファイルを再読み込み（変更を破棄）
- `:e!`: 強制的に再読み込み（未保存の変更を破棄）
- `:e! filename`: 強制的に指定ファイルを開く

**実装ノート**:
- コマンドバッファから引数を抽出（`split_whitespace()` で空白分割）
- `:e filename` 形式:
  - dirty なら警告を出して中止（`!` なしの場合）
  - ファイルが存在しなければエラー
  - 成功時はカーソルを先頭に移動（`Cursor::new()`）
- `:e` 形式（再読み込み）:
  - 現在のファイル名で再読み込み
  - ファイル名がない場合はエラー
  - **カーソル位置は維持**（UX 向上のため）
  - カーソルが範囲外になった場合は自動調整
- ファイル切り替え時の処理:
  - Buffer を新規作成
  - dirty フラグをクリア
  - filename を更新

**カーソル範囲調整機能**:
- `cursor.rs` に `ensure_within_bounds` メソッドを追加
- ファイル再読み込み後、カーソルがバッファ範囲外になった場合に自動調整
- 空バッファの場合は (1,1) に移動
- 行が範囲外の場合は最終行に移動
- x 座標を現在行の長さに合わせて調整

**設計判断**:
- **アプローチ2（メソッド化）を採用**: カーソル範囲チェックを `Cursor::ensure_within_bounds` として実装
  - 利点: カプセル化、再利用性、保守性の向上
  - フェーズ5以降（`dd` コマンドなど）でも活用可能

**エラーハンドリング**:
- ファイルが見つからない: `"Cannot open file: {error}"`
- ファイル名なしで `:e`: `"No file name"`
- 未保存で実行: `"No write since last change (add ! to override)"`

**Critical Files**:
- `src/editor.rs`（`open_file`, `reload` メソッド）
- `src/main.rs`（`:e` コマンド処理）
- `src/cursor.rs`（`ensure_within_bounds` メソッド）

---

### フェーズ5: 追加のNormalモードコマンド ✅
**目標**: vim らしい編集操作ができる

**ステータス**: 完了

**実装内容**:
1. `editor.rs`: ヤンクバッファ管理と編集メソッド
2. `buffer.rs`: 行削除、行ヤンク処理、文字削除の改善
3. `main.rs`: 2キーストローク処理（`dd`, `yy`）とキーバインド
4. `pending_key` パターンで2キーコマンドを実装

**キーバインド**:
- `x`: カーソル位置の文字削除（削除した文字をヤンク）
- `dd`: 行削除（ヤンクバッファに保存）
- `yy`: 行ヤンク（コピー）
- `p`: ヤンクバッファの内容をカーソルの下の行にペースト
- `P`: ヤンクバッファの内容をカーソルの上の行にペースト

**実装ノート**:

**ヤンクバッファ**:
- `editor.rs` に `yank_buffer: Vec<String>` を追加
- `Vec<String>` で実装し、将来的な複数行ヤンク（`3dd` など）に対応可能な設計
- `x`, `dd` での削除内容も自動的にヤンクバッファに保存

**編集メソッド（editor.rs）**:
- `delete_char_at_cursor(row, col)`: 文字削除、ヤンクバッファに保存
- `delete_line(row)`: 行削除、ヤンクバッファに保存
- `yank_line(row)`: 行をヤンクバッファにコピー
- `paste_below(row)`: カーソルの下にペースト
- `paste_above(row)`: カーソルの上にペースト

**Buffer の改善（buffer.rs）**:
- `Row::delete_char`: 返り値を `Option<char>` に変更（削除した文字を返す）
- `delete_row_with_content(at)`: 行を削除して内容を返す（`dd` 用）
- `get_row_content(at)`: 行の内容を取得（`yy` 用）

**2キーストローク処理**:
- `pending_key` パターンを活用
- `dd`: 1回目の `d` で `next_pending_key = Some('d')`、2回目で実行
- `yy`: 同様のパターン
- `gg` との共存も問題なし

**カーソル調整**:
- `x` コマンド後: 行末を超えないように調整
- `dd` コマンド後: **Phase 4.5 の `ensure_within_bounds` を再利用**
  - 空バッファになった場合も適切に処理
  - メソッドの再利用により保守性向上
- `p` コマンド後: ペーストした行にカーソルを移動

**設計の利点**:
- Phase 4.5 の `ensure_within_bounds` メソッドを活用し、コードの重複を回避
- ヤンクバッファを `Vec<String>` で実装し、拡張性を確保
- 各操作が `bool` を返すことで、成功/失敗を判断可能

**Critical Files**:
- `src/editor.rs`（ヤンクバッファ、編集メソッド）
- `src/buffer.rs`（行操作メソッド、文字削除の改善）
- `src/main.rs`（キーバインド、2キーストローク処理）

---

### フェーズ5.5: ペースト機能の改善とYankManager ✅
**目標**: ペースト動作の改善と状態管理の強化

**ステータス**: 完了

**実装内容**:
1. 文字単位と行単位のペーストを区別
2. YankManager による状態管理の導入
3. `p` / `P` コマンドの動作改善
4. システムクリップボード統合（Phase 5 で既に実装済み）

**Part 1: YankManager の導入**

**データ構造**:
```rust
enum YankType {
    InLine,   // 行内にペースト
    NewLine,  // 新しい行としてペースト
}

struct YankManager {
    buffer: Vec<String>,
    yank_type: YankType,
}
```

**設計の利点**:
- タイプと内容が必ず同期（構造的にバグを防止）
- `yank_inline()`, `yank_line()` メソッドで type を確実に設定
- カプセル化により将来の拡張が容易（Visual mode など）

**Part 2: ペースト動作の改善**

**PasteDirection と PasteResult**:
```rust
pub enum PasteDirection {
    Below,  // p コマンド
    Above,  // P コマンド
}

pub enum PasteResult {
    Empty,   // ヤンクバッファが空
    InLine,  // カーソルのある行に挿入
    Above,   // 上の行に挿入
    Below,   // 下の行に挿入
}
```

**ペースト動作**:
- **行単位ヤンク** (`dd`, `yy`):
  - `p`: カーソルの下の行にペースト
  - `P`: カーソルの上の行にペースト
- **文字単位ヤンク** (`x`):
  - `p`: カーソルの後ろにペースト
  - `P`: カーソルの前にペースト

**実装の詳細**:
```rust
// editor.rs の paste() メソッド
pub fn paste(&mut self, row: usize, col: usize, direction: PasteDirection) -> PasteResult {
    if self.yank_manager.is_newline_yank() {
        // 行単位: 新しい行として挿入
        match direction {
            Below => insert_row(row + 1, ...),
            Above => insert_row(row, ...),
        }
    } else {
        // 文字単位: 行内に挿入
        let col = match direction {
            Below => col + 1,  // カーソルの後ろ
            Above => col,       // カーソルの前
        };
        insert_str(col, ...);
    }
}
```

**Part 3: クリップボード統合**

**依存クレート**:
```toml
[dependencies]
arboard = "3.4"  # クロスプラットフォームクリップボード
```

**実装**:
- Phase 5 で既に実装済み
- `sync_to_clipboard()` でヤンク時にクリップボードにコピー
- オプション機能として動作（失敗してもエディタは継続）

**Part 4: リファクタリング（Phase 5 で一部実装済み）**

**実装済み**:
- `editor.rs` に `buffer_info(row)` メソッドを追加
  - バッファの長さと指定行の長さを取得
  - カーソル位置調整時に頻繁に使用
- `editor.rs` に `current_line_len(row)` メソッドを追加
  - 現在行の長さを取得

**将来のリファクタリング候補**:
- main.rs のキーハンドラの重複コード削減
- カーソル位置情報取得のヘルパー関数化
- ステータスメッセージ管理の改善

**設計の成果**:
- YankManager による状態管理で、タイプミスや設定忘れを防止
- InLine / NewLine の明確な区別により、ペースト動作が直感的に
- 変数の shadowing を活用した簡潔なコード（`col` の調整）
- Phase 4.5 の `ensure_within_bounds` を再利用

**Critical Files**:
- `src/editor.rs`（YankManager, paste メソッド、便利メソッド）
- `src/buffer.rs`（insert_str メソッド）
- `src/main.rs`（p/P コマンドの実装）

---

### フェーズ6: Visualモード（基本 - character-wise）
**目標**: 文字単位でテキスト選択、ヤンク、削除ができる

**実装内容**:
1. `mode.rs`: Visual モード管理、選択範囲の記録
2. `screen.rs`: 選択範囲のハイライト表示
3. `main.rs`: Visual モードでのキー処理
4. `editor.rs`: 範囲削除、範囲ヤンク

**キーバインド**:
- Normal モードで `v`: Visual モード開始（character-wise）
- Visual モードで `h/j/k/l`: 選択範囲拡大
- Visual モードで `y`: 選択範囲をヤンク、Normal モードへ
- Visual モードで `d`: 選択範囲を削除、Normal モードへ
- Visual モードで `ESC`: Normal モードへ（選択解除）

**実装の詳細**:
- **文字単位選択**: 開始位置から現在のカーソル位置まで文字単位で選択
- **行をまたぐ選択**: 複数行にまたがる選択も可能
- **ハイライト表示**: 選択範囲を反転表示（`termion::style::Invert`）
- **座標の正規化**: 開始位置より前にカーソル移動した場合も正しく処理

**注意点**:
- このフェーズでは `v` (character-wise) のみ実装
- `V` (line-wise) と `Ctrl+v` (block-wise) は Phase 6.5 で実装

**Critical Files**:
- `src/mode.rs`
- `src/screen.rs`
- `src/editor.rs`
- `src/main.rs`

---

### フェーズ6.5: Visual Line / Visual Block モード
**目標**: 行単位選択と矩形選択ができる

**実装内容**:
1. `mode.rs`: Visual Line と Visual Block モードを追加
2. `screen.rs`: 各モードに応じたハイライト表示
3. `main.rs`: `V` と `Ctrl+v` のキーバインド
4. `editor.rs`: 行単位・ブロック単位の範囲操作

**キーバインド**:

**Visual Line モード (`V`)**:
- Normal モードで `V`: Visual Line モード開始
- カーソル移動で行単位で選択範囲が広がる
- `y`: 選択した行をヤンク
- `d`: 選択した行を削除
- `ESC`: Normal モードへ

**Visual Block モード (`Ctrl+v`)**:
- Normal モードで `Ctrl+v`: Visual Block モード開始
- カーソル移動で矩形領域が選択される
- `y`: 矩形領域をヤンク
- `d`: 矩形領域を削除
- `I`: 矩形領域の各行の先頭に挿入（高度な機能、オプション）
- `ESC`: Normal モードへ

**実装の詳細**:

**Visual Line モード**:
- 選択範囲は常に行全体
- 開始行から現在行まで、全ての行が選択される
- ヤンク・削除は行単位で実行

**Visual Block モード**:
- 矩形領域を選択（開始位置と現在位置で矩形を定義）
- 各行の同じ列範囲を選択
- ヤンク時は各行の選択部分を保存
- ペースト時は各行に貼り付け
- 表示幅を考慮した列計算が必要（全角文字対応）

**Mode enum の拡張**:
```rust
enum Mode {
    Normal,
    Insert,
    Command,
    Visual,          // character-wise (Phase 6)
    VisualLine,      // line-wise (Phase 6.5)
    VisualBlock,     // block-wise (Phase 6.5)
}
```

**注意点**:
- Visual Block モードは全角文字の扱いが複雑
- Phase 7 (UTF-8 完全対応) 後に実装するとより安全
- または ASCII 範囲での動作に限定して実装し、Phase 7 で改善

**Critical Files**:
- `src/mode.rs`
- `src/screen.rs`
- `src/editor.rs`
- `src/main.rs`

---

### フェーズ7: UTF-8 完全対応
**目標**: 日本語などのマルチバイト文字を正しく表示・編集できる

**実装内容**:
1. `buffer.rs`: グラフィームクラスタ単位での文字列操作
2. `cursor.rs`: グラフィーム単位でのカーソル移動
3. `screen.rs`: 文字幅を考慮したレンダリング（全角文字は2カラム幅）
4. `file_io.rs`: UTF-8 ファイルの読み書き（既に Rust の String で対応済みだが確認）

**実装の詳細**:

**グラフィームクラスタ対応**:
```rust
use unicode_segmentation::UnicodeSegmentation;

// 文字列をグラフィームクラスタに分割
let graphemes: Vec<&str> = text.graphemes(true).collect();

// カーソル移動は graphemes のインデックスで行う
```

**文字幅計算**:
```rust
use unicode_width::UnicodeWidthStr;

// 文字列の表示幅を取得
let width = text.width();

// 1文字ごとの幅
for grapheme in text.graphemes(true) {
    let width = grapheme.width(); // 全角=2, 半角=1
}
```

**考慮すべき点**:
- 結合文字（é = e + ´）を1文字として扱う
- 絵文字（👨‍👩‍👧‍👦 など）を1文字として扱う
- 全角文字のカーソル位置計算（2カラム幅）
- 行の途中で折り返す際の文字幅考慮

**Critical Files**:
- `src/buffer.rs`
- `src/cursor.rs`
- `src/screen.rs`

---

## kilolo.c との対応関係

| kilolo.c | zim (Rust) | 役割 |
|----------|------------|------|
| `enableRawMode()` | `terminal.rs`: `Terminal::enable_raw_mode()` | raw mode 設定 |
| `struct editorConfig` | `editor.rs`: `Editor` | エディタ状態管理 |
| `erow` | `buffer.rs`: `Row` | 1行のテキスト |
| `editorRefreshScreen()` | `screen.rs`: `Screen::refresh()` | 画面再描画 |
| `editorProcessKeypress()` | `keymap.rs` + `Editor::execute_command()` | キー入力処理 |
| `editorMoveCursor()` | `cursor.rs`: `Cursor::move_*()` | カーソル移動 |
| `editorOpen()` | `file_io.rs`: `FileIO::open()` | ファイル読み込み |
| `editorSave()` | `file_io.rs`: `FileIO::save()` | ファイル保存 |

**追加要素（Vim 特有）**:
- `mode.rs`: モード管理（Normal/Insert/Visual/Command）
- 2キーストローク処理（dd, yy など）
- ヤンクバッファとペースト機能

## 実装戦略

1. **各フェーズで動作するエディタを作る**: フェーズ1終了時点で「起動して終了できる」、フェーズ2で「ファイルが見られる」など、常に動作する状態を保つ

2. **最小限から始める**: 各フェーズで必要最小限の機能のみ実装。機能追加は後のフェーズで

3. **kilolo.c を参考にする**: 実装に困ったら kilolo.c の該当部分を確認

4. **テストしながら進める**: 各機能実装後、実際にエディタを起動して動作確認

5. **UTF-8 は段階的に対応**: フェーズ1-6では ASCII ベースで実装し、フェーズ7で UTF-8 完全対応に移行。Rust の String は既に UTF-8 なので基本的な読み書きは自然に対応する

6. **改行コードは LF のみ**: macOS/Linux の改行コード `\n` のみサポート。ターミナル出力（raw mode）では `\r\n` を使用するが、ファイル保存時は `\n` のみ

## 最初のステップ（フェーズ1詳細）

### 1. Cargo.toml に依存を追加
```toml
termion = "4"
anyhow = "1.0"
thiserror = "2.0"
```

### 2. terminal.rs 実装
- `Terminal` 構造体
- raw mode 設定
- 画面クリア
- 画面サイズ取得

### 3. mode.rs 実装
- `Mode` enum
- `ModeManager` 構造体
- モード遷移メソッド

### 4. main.rs 実装
- 基本的なメインループ
- キー入力読み取り
- `:q` で終了

### 5. 動作確認
- `cargo run` でエディタが起動
- 画面に `~` が表示される
- `:q` で終了できる

これが動作したら、フェーズ2へ進みます。

