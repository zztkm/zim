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
termion = "4"      # ターミナル制御（raw mode、キー入力、ANSI escape）
anyhow = "1.0"     # エラーハンドリング
thiserror = "2.0"  # カスタムエラー型定義
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
    x: usize,      // 実際の文字位置
    y: usize,      // 行位置
    rx: usize,     // レンダリング位置（タブ考慮）
    row_offset: usize,
    col_offset: usize,
}
```

## 段階的実装計画

### フェーズ1: 基本ターミナル操作とNormalモード
**目標**: 空のエディタが起動し、カーソル移動と終了ができる

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

### フェーズ2: ファイル読み込みと表示
**目標**: ファイルを開いて閲覧できる

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

**Critical Files**:
- `src/buffer.rs`
- `src/file_io.rs`
- `src/screen.rs`

---

### フェーズ3: Insertモードと基本編集
**目標**: テキスト編集ができる

**実装内容**:
1. `mode.rs`: Insert モードへの遷移処理
2. `keymap.rs`: Insert モードでのキー入力処理
3. `buffer.rs`: 文字挿入、文字削除、改行挿入
4. `editor.rs`: dirty フラグ管理
5. Insert モード表示（ステータスバーに "-- INSERT --"）

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

**Critical Files**:
- `src/keymap.rs`
- `src/editor.rs`

---

### フェーズ4: ファイル保存
**目標**: 編集内容を保存できる

**実装内容**:
1. `file_io.rs`: ファイル保存処理
2. `keymap.rs`: Command モード処理（`:w`, `:wq`, `:q!`）
3. Command バッファ管理（`:` 入力時の処理）
4. 未保存時の警告（`:q` で dirty の場合）

**キーバインド**:
- `:w`: 保存
- `:q`: 終了（未保存時は警告）
- `:wq`: 保存して終了
- `:q!`: 強制終了

**Critical Files**:
- `src/file_io.rs`
- `src/keymap.rs`

---

### フェーズ5: 追加のNormalモードコマンド
**目標**: vim らしい編集操作ができる

**実装内容**:
1. `keymap.rs`: 2キーストローク処理（dd, yy など）
2. `buffer.rs`: 行削除、行ヤンク処理
3. `editor.rs`: ヤンクバッファ管理、ペースト処理
4. PendingCommand 構造体（dd, yy の実装）

**キーバインド**:
- `x`: カーソル位置の文字削除
- `dd`: 行削除
- `yy`: 行ヤンク
- `p`: カーソル後ろにペースト
- `P`: カーソル前にペースト
- `u`: Undo（オプション）

**Critical Files**:
- `src/keymap.rs`
- `src/buffer.rs`

---

### フェーズ6: Visualモード（基本）
**目標**: テキスト選択、ヤンク、削除ができる

**実装内容**:
1. `mode.rs`: Visual モード管理、選択範囲の記録
2. `screen.rs`: 選択範囲のハイライト表示
3. `keymap.rs`: Visual モードでのキー処理
4. `buffer.rs`: 範囲削除、範囲ヤンク

**キーバインド**:
- Normal モードで `v`: Visual モード開始
- Visual モードで `h/j/k/l`: 選択範囲拡大
- Visual モードで `y`: 選択範囲をヤンク、Normal モードへ
- Visual モードで `d`: 選択範囲を削除、Normal モードへ
- Visual モードで `ESC`: Normal モードへ（選択解除）

**Critical Files**:
- `src/mode.rs`
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

