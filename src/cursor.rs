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

    pub fn move_down(&mut self, max_rows: u16) {
        if self.y < max_rows {
            self.y += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    pub fn move_right(&mut self, max_cols: u16) {
        if self.x < max_cols {
            self.x += 1;
        }
    }
    pub fn move_to_line_start(&mut self) {
        self.x = 1;
    }

    pub fn move_to_line_end(&mut self, line_len: u16) {
        self.x = line_len + 1;
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
            self.y = last_line - self.row_offset + 1
        }
    }

    /// スクロール処理
    /// editor_rows: エディタ領域の行数(ステータスバーなどを除く)
    pub fn scroll(&mut self, editor_rows: u16, buffer_len: usize) {
        let file_row = self.row_offset + self.y - 1;

        // 画面上端より上にカーソルがある場合
        if file_row < self.row_offset {
            self.row_offset = file_row;
        }

        // 画面下端より下にカーソルがある場合
        if file_row >= self.row_offset + editor_rows {
            self.row_offset = file_row.saturating_sub(editor_rows - 1);
        }

        // カーソルが画面内に収まるように調整
        if (self.row_offset as usize) < buffer_len {
            self.y = file_row - self.row_offset + 1;
        }
    }

    pub fn file_row(&self) -> usize {
        (self.row_offset + self.y - 1) as usize
    }
}
