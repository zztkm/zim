pub struct Cursor {
    x: u16,
    y: u16,
    row_offset: u16,
    col_offset: u16,
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
        if self.y > 1 {
            self.y -= 1;
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

        // バッファの範囲内、かつ画面の範囲内のみで移動可能
        if current_file_row < last_row && self.y < max_rows {
            self.y += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    pub fn move_right(&mut self, max_cols: u16, line_len: usize) {
        // 空行の場合は移動しない
        if line_len == 0 {
            return;
        }

        // vim の Normal モードでは行の最後の文字まで移動可能
        let max_x = (line_len as u16).min(max_cols);

        if self.x < max_x {
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
    /// バッファ内の行インデックス雨を計算して返します。
    pub fn file_row(&self) -> usize {
        (self.row_offset + self.y - 1) as usize
    }
}
