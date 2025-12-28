pub struct Row {
    chars: String,
    render: String,
}

impl Row {
    pub fn new(text: String) -> Self {
        let render = text.clone();
        Self {
            chars: text,
            render,
        }
    }

    pub fn chars(&self) -> &str {
        &self.chars
    }

    pub fn render(&self) -> &str {
        &self.render
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// 指定位置に文字を挿入
    pub fn insert_char(&mut self, at: usize, ch: char) {
        if at <= self.chars.len() {
            self.chars.insert(at, ch);
            // TODO: タブ展開は後で実装
            self.render = self.chars.clone();
        }
    }
    /// 指定位置の文字を削除
    pub fn delete_char(&mut self, at: usize) {
        if at < self.chars.len() {
            self.chars.remove(at);
            self.render = self.chars.clone();
        }
    }
    /// 指定位置から末尾までを分割して返す
    pub fn split_off(&mut self, at: usize) -> String {
        if at <= self.chars.len() {
            let tail = self.chars.split_off(at);
            self.render = self.chars.clone();
            tail
        } else {
            String::new()
        }
    }

    /// 文字列を末尾に追加
    pub fn append(&mut self, s: &str) {
        self.chars.push_str(s);
        self.render = self.chars.clone();
    }
}

pub struct Buffer {
    rows: Vec<Row>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn insert_row(&mut self, at: usize, text: String) {
        if at <= self.rows.len() {
            self.rows.insert(at, Row::new(text));
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// 指定行を削除
    pub fn delete_row(&mut self, at: usize) {
        if at < self.rows.len() {
            self.rows.remove(at);
        }
    }

    /// 指定行に文字を挿入
    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        // 行が存在しない場合は空行を追加
        if row >= self.rows.len() {
            self.insert_row(self.rows.len(), String::new());
        }

        if let Some(r) = self.rows.get_mut(row) {
            r.insert_char(col, ch);
        }
    }

    /// 指定行の文字を削除する
    pub fn delete_char(&mut self, row: usize, col: usize) {
        if let Some(r) = self.rows.get_mut(row) {
            r.delete_char(col);
        }
    }

    /// 改行を挿入（現在行を分割）
    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if row >= self.rows.len() {
            // 最後の行より後ろの場合は空行を追加
            self.insert_row(self.rows.len(), String::new());
        } else if let Some(current_row) = self.rows.get_mut(row) {
            // 現在行を分割
            let tail = current_row.split_off(col);
            // 次の行として挿入
            self.insert_row(row + 1, tail);
        }
    }

    /// 前の行と結合
    pub fn join_rows(&mut self, row: usize) {
        if row > 0 && row < self.rows.len() {
            let current_line = self.rows.remove(row);
            if let Some(prev_row) = self.rows.get_mut(row - 1) {
                prev_row.append(current_line.chars());
            }
        }
    }

    pub fn row_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }
}
