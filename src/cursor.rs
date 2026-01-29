use crate::buffer::Row;

/// ファイル内の位置を表す構造体 (0-indexed)
///
/// バッファ操作は常に 0-indexed で行われるため、
/// Cursor の 1-indexed (x, y) とは区別して使用します。
///
/// # Fields
/// - `row`: 行番号 (0-indexed)
/// - `col`: 列番号 (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    /// 新しい Position を作成
    ///
    /// # Arguments
    /// - `row`: 行番号 (0-indexed)
    /// - `col`: 列番号 (0-indexed)
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

pub struct Cursor {
    x: u16,
    y: u16,
    row_offset: u16,
    col_offset: u16,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 1,
            y: 1,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn row_offset(&self) -> u16 {
        self.row_offset
    }
    pub fn col_offset(&self) -> u16 {
        self.col_offset
    }

    pub fn move_up(&mut self) {
        // ファイル内の現在行
        let current_file_row = self.row_offset + self.y - 1;

        // ファイルの先頭より上には移動できない
        if current_file_row > 0 {
            if self.y > 1 {
                // 画面内では y を減らす
                self.y -= 1;
            } else {
                // 画面上端に達している場合は、スクロールする
                self.row_offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self, max_rows: u16, buffer_len: usize) {
        // バッファが空なので移動しない
        if buffer_len == 0 {
            return;
        }

        // ファイル内の現在行
        let current_file_row = self.row_offset + self.y - 1;

        // バッファの最後の行のインデックス
        let last_row = (buffer_len as u16).saturating_sub(1);

        // バッファの範囲内で移動可能
        if current_file_row < last_row {
            if self.y < max_rows {
                // 画面内では y を増やす
                self.y += 1;
            } else {
                // 画面下端に到達している場合は、スクロールする
                self.row_offset += 1;
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    pub fn move_right(&mut self, line_len: usize) {
        // 空行の場合は移動しない
        if line_len == 0 {
            return;
        }

        // vim の Normal モードでは行の最後の文字まで移動可能
        if self.x < line_len as u16 {
            self.x += 1;
        }
    }
    pub fn move_to_line_start(&mut self) {
        self.x = 1;
    }

    pub fn move_to_line_end(&mut self, line_len: u16) {
        if line_len == 0 {
            self.x = 1;
        } else {
            self.x = line_len;
        }
    }

    /// ファイル先頭に移動
    pub fn move_to_top(&mut self) {
        self.y = 1;
        self.row_offset = 0;
    }

    /// ファイル末尾に移動
    pub fn move_to_bottom(&mut self, buffer_len: usize, editor_rows: u16) {
        if buffer_len == 0 {
            self.y = 1;
            self.row_offset = 0;
            return;
        }

        let last_line = buffer_len.saturating_sub(1) as u16;

        // 画面に収まる場合
        if last_line < editor_rows {
            self.y = last_line + 1;
            self.row_offset = 0;
        } else {
            // スクロールが必要な場合
            self.row_offset = last_line.saturating_sub(editor_rows - 1);
            self.y = last_line - self.row_offset + 1;
        }
    }

    pub fn adjust_cursor_x(&mut self, line_len: usize) {
        if line_len == 0 {
            self.x = 1;
        } else {
            let max_x = line_len as u16;
            if self.x > max_x {
                self.x = max_x
            }
        }
    }

    /// カーソル位置をバッファの範囲内に調整する
    ///
    /// ファイル再読み込みや行数削除後など、カーソルが範囲外になる操作が
    /// 行われたあとに呼び出す。
    ///
    /// # Arguments
    ///
    /// - `buffer_len`: バッファの行数
    /// - `line_len`: 現在行の長さ(文字数)
    /// - `editor_rows`: エディタ領域の行数(画面の行数 - UI 要素)
    pub fn ensure_within_bounds(&mut self, buffer_len: usize, line_len: usize, editor_rows: u16) {
        if buffer_len == 0 {
            self.y = 1;
            self.x = 1;
            self.row_offset = 0;
            return;
        }

        let current_file_row = self.file_row();

        // 行が範囲外の場合は、最終行に移動
        if current_file_row >= buffer_len {
            self.move_to_bottom(buffer_len, editor_rows);
        }

        self.adjust_cursor_x(line_len);
    }

    /// スクロール処理
    /// editor_rows: エディタ領域の行数(ステータスバーなどを除く)
    pub fn scroll(&mut self, editor_rows: u16, buffer_len: usize) {
        // バッファが空の場合はスクロールしない
        if buffer_len == 0 {
            self.y = 1;
            self.row_offset = 0;
            return;
        }

        let file_row = self.row_offset + self.y - 1;
        let last_row = buffer_len.saturating_sub(1) as u16;

        // カーソルがバッファの範囲を超えていたら修正
        if file_row > last_row {
            if last_row < editor_rows {
                self.y = last_row + 1;
                self.row_offset = 0;
            } else {
                self.row_offset = last_row.saturating_sub(editor_rows - 1);
                self.y = last_row - self.row_offset + 1;
            }
            return;
        }

        // 画面上端より上にカーソルがある場合
        if file_row < self.row_offset {
            self.row_offset = file_row;
        }

        // 画面下端より下にカーソルがある場合
        if file_row >= self.row_offset + editor_rows {
            self.row_offset = file_row.saturating_sub(editor_rows - 1);
        }

        // カーソルの y 座標を画面内の位置に調整
        self.y = file_row - self.row_offset + 1;
    }

    /// ファイル内の実際の行番号を取得する (0-indexed)
    ///
    /// カーソルの画面上の位置 y とスクロールオフセット row_offset から
    /// バッファ内の行インデックスを計算して返します。
    pub fn file_row(&self) -> usize {
        (self.row_offset + self.y - 1) as usize
    }

    /// カーソルの列位置を 0-indexed で取得
    ///
    /// Cursor の x は 1-indexed（画面座標）ですが、
    /// バッファ操作では 0-indexed が必要なため、変換して返します。
    ///
    /// # Returns
    /// 0-indexed の列番号
    pub fn col_index(&self) -> usize {
        (self.x - 1) as usize
    }

    /// 画面描画用の x 座標を計算（表示幅ベース、1-indexed）
    ///
    /// グラフィームインデックスベースの `x` を、
    /// 実際の画面表示位置（表示幅考慮）に変換します。
    ///
    /// # Arguments
    /// - `row`: 現在行の Row
    ///
    /// # Returns
    /// 画面描画用の x 座標（1-indexed）
    pub fn render_x(&self, row: &Row) -> u16 {
        let grapheme_idx = self.col_index();
        let render_pos = row.render_x(grapheme_idx);
        (render_pos as u16) + 1
    }

    /// カーソルの現在位置を Position として取得
    ///
    /// バッファ操作用の 0-indexed の Position を返します。
    ///
    /// # Returns
    /// 現在のカーソル位置を表す Position (row, col ともに 0-indexed)
    pub fn position(&self) -> Position {
        Position::new(self.file_row(), self.col_index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.x(), 1);
        assert_eq!(cursor.y(), 1);
        assert_eq!(cursor.row_offset(), 0);
        assert_eq!(cursor.file_row(), 0);
    }

    #[test]
    fn test_cursor_move_basic() {
        let mut cursor = Cursor::new();

        cursor.move_right(10);
        assert_eq!(cursor.x(), 2);

        cursor.move_left();
        assert_eq!(cursor.x(), 1);

        cursor.move_down(24, 10);
        assert_eq!(cursor.y(), 2);

        cursor.move_up();
        assert_eq!(cursor.y(), 1);
    }

    #[test]
    fn test_cursor_move_boundaries() {
        let mut cursor = Cursor::new();

        // 左端でさらに左に移動しても x=1 のまま
        cursor.move_left();
        assert_eq!(cursor.x(), 1);

        // 上端でさらに上に移動しても y=1 のまま
        cursor.move_up();
        assert_eq!(cursor.y(), 1);
    }

    #[test]
    fn test_cursor_move_right_limit() {
        let mut cursor = Cursor::new();
        let line_len = 5; // "hello" の長さ

        // Normal モードでは行末まで移動可能
        for _ in 0..10 {
            cursor.move_right(line_len);
        }
        assert_eq!(cursor.x(), 5); // 最後の文字まで
    }

    #[test]
    fn test_cursor_move_down_limit() {
        let mut cursor = Cursor::new();
        let buffer_len = 3;
        let editor_rows = 24;

        // バッファの最後の行まで移動
        for _ in 0..10 {
            cursor.move_down(editor_rows, buffer_len);
        }
        assert_eq!(cursor.y(), 3); // buffer_len まで
    }

    #[test]
    fn test_cursor_move_to_line_start() {
        let mut cursor = Cursor::new();
        cursor.move_right(10);
        cursor.move_right(10);
        assert_eq!(cursor.x(), 3);

        cursor.move_to_line_start();
        assert_eq!(cursor.x(), 1);
    }

    #[test]
    fn test_cursor_move_to_line_end() {
        let mut cursor = Cursor::new();
        cursor.move_to_line_end(5);
        assert_eq!(cursor.x(), 5);

        cursor.move_to_line_end(0); // 空行
        assert_eq!(cursor.x(), 1);
    }

    #[test]
    fn test_cursor_move_to_top() {
        let mut cursor = Cursor::new();
        cursor.move_down(24, 10);
        cursor.move_down(24, 10);
        cursor.move_right(10);

        cursor.move_to_top();
        assert_eq!(cursor.y(), 1);
        assert_eq!(cursor.row_offset(), 0);
    }

    #[test]
    fn test_cursor_move_to_bottom() {
        let mut cursor = Cursor::new();
        let buffer_len = 5;
        let editor_rows = 24;

        cursor.move_to_bottom(buffer_len, editor_rows);

        // buffer_len=5 なので最後の行は index 4 → y=5
        assert_eq!(cursor.y(), 5);
        assert_eq!(cursor.row_offset(), 0);
    }

    #[test]
    fn test_cursor_move_to_bottom_large_file() {
        let mut cursor = Cursor::new();
        let buffer_len = 100;
        let editor_rows = 24;

        cursor.move_to_bottom(buffer_len, editor_rows);

        // スクロールが必要
        let last_line = 99u16; // 0-indexed で最後は 99
        assert_eq!(
            cursor.row_offset(),
            last_line.saturating_sub(editor_rows - 1)
        );
        assert_eq!(cursor.y(), last_line - cursor.row_offset() + 1);
    }

    #[test]
    fn test_cursor_adjust_cursor_x() {
        let mut cursor = Cursor::new();
        cursor.move_right(10);
        cursor.move_right(10);
        cursor.move_right(10);
        assert_eq!(cursor.x(), 4);

        // 短い行に移動した場合
        cursor.adjust_cursor_x(2);
        assert_eq!(cursor.x(), 2);

        // 空行に移動した場合
        cursor.adjust_cursor_x(0);
        assert_eq!(cursor.x(), 1);
    }

    #[test]
    fn test_cursor_ensure_within_bounds_empty_buffer() {
        let mut cursor = Cursor::new();
        cursor.move_down(24, 10);
        cursor.move_right(10);

        cursor.ensure_within_bounds(0, 0, 24);

        assert_eq!(cursor.x(), 1);
        assert_eq!(cursor.y(), 1);
        assert_eq!(cursor.row_offset(), 0);
    }

    #[test]
    fn test_cursor_ensure_within_bounds_row_out_of_range() {
        let mut cursor = Cursor::new();
        // カーソルを 10 行目に移動
        for _ in 0..9 {
            cursor.move_down(24, 100);
        }
        assert_eq!(cursor.file_row(), 9);

        // バッファが 5 行しかない場合
        cursor.ensure_within_bounds(5, 10, 24);

        // 最終行（index 4 → y=5）に移動
        assert_eq!(cursor.file_row(), 4);
    }

    #[test]
    fn test_cursor_ensure_within_bounds_col_adjustment() {
        let mut cursor = Cursor::new();
        cursor.move_right(20);
        cursor.move_right(20);
        cursor.move_right(20);
        assert_eq!(cursor.x(), 4);

        // 現在行が 2 文字しかない場合
        cursor.ensure_within_bounds(10, 2, 24);

        assert_eq!(cursor.x(), 2);
    }

    #[test]
    fn test_cursor_file_row() {
        let mut cursor = Cursor::new();
        assert_eq!(cursor.file_row(), 0);

        cursor.move_down(24, 10);
        assert_eq!(cursor.file_row(), 1);

        cursor.move_down(24, 10);
        assert_eq!(cursor.file_row(), 2);
    }

    #[test]
    fn test_cursor_scroll() {
        let mut cursor = Cursor::new();
        let buffer_len = 100;
        let editor_rows = 24;

        // 画面下端を超えて移動（実際のメインループでは scroll が毎回呼ばれる）
        for _ in 0..30 {
            cursor.move_down(editor_rows, buffer_len);
            cursor.scroll(editor_rows, buffer_len);
        }

        // スクロールが発生しているはず
        assert!(cursor.row_offset() > 0);
        assert_eq!(cursor.file_row(), 30);
    }

    #[test]
    fn test_cursor_scroll_at_bottom() {
        let mut cursor = Cursor::new();
        let buffer_len = 10;
        let editor_rows = 24;

        // 小さいファイルではスクロールは発生しない
        for _ in 0..20 {
            cursor.move_down(editor_rows, buffer_len);
            cursor.scroll(editor_rows, buffer_len);
        }

        assert_eq!(cursor.row_offset(), 0);
        assert_eq!(cursor.file_row(), 9); // 最終行（0-indexed）
    }

    #[test]
    fn test_cursor_scroll_up() {
        let mut cursor = Cursor::new();
        let buffer_len = 100;
        let editor_rows = 24;

        // ファイルの 30行目に移動（スクロールが発生する位置）
        for _ in 0..30 {
            cursor.move_down(editor_rows, buffer_len);
            cursor.scroll(editor_rows, buffer_len);
        }

        assert_eq!(cursor.file_row(), 30);
        let initial_offset = cursor.row_offset();
        assert!(initial_offset > 0); // スクロールしている
        assert_eq!(initial_offset, 7); // row_offset = 30 - 23 = 7
        assert_eq!(cursor.y(), 24); // 画面下端

        // 上に 25回移動（画面上端を超えて移動）
        // 最初の 23回で y=1 に到達、残り 2回で row_offset が減少
        for _ in 0..25 {
            cursor.move_up();
            cursor.scroll(editor_rows, buffer_len);
        }

        // 上方向のスクロールが発生しているはず
        assert_eq!(cursor.file_row(), 5); // 30 - 25 = 5
        assert_eq!(cursor.y(), 1); // 画面上端
        assert!(cursor.row_offset() < initial_offset); // 7 → 5
        assert_eq!(cursor.row_offset(), 5);
    }

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.row, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(1, 2);
        let pos2 = Position::new(1, 2);
        let pos3 = Position::new(2, 1);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_cursor_col_index() {
        let mut cursor = Cursor::new();
        // 初期状態: x=1 → col_index=0
        assert_eq!(cursor.col_index(), 0);

        // 右に1回移動: x=2 → col_index=1
        cursor.move_right(10);
        assert_eq!(cursor.col_index(), 1);

        // さらに右に移動: x=3 → col_index=2
        cursor.move_right(10);
        assert_eq!(cursor.col_index(), 2);
    }

    #[test]
    fn test_cursor_position() {
        let mut cursor = Cursor::new();
        // 初期状態: row=0, col=0
        let pos = cursor.position();
        assert_eq!(pos, Position::new(0, 0));

        // 右に移動してから確認
        cursor.move_right(10);
        let pos = cursor.position();
        assert_eq!(pos, Position::new(0, 1));

        // 下に移動してから確認
        cursor.move_down(24, 10);
        let pos = cursor.position();
        assert_eq!(pos, Position::new(1, 1));
    }
}
