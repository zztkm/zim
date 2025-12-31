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

    /// 指定位置に文字を挿入
    pub fn insert_str(&mut self, at: usize, s: &str) {
        if at <= self.chars.len() {
            self.chars.insert_str(at, s);
            self.render = self.chars.clone();
        }
    }

    /// 指定位置の文字を削除し、削除した文字を返す
    pub fn delete_char(&mut self, at: usize) -> Option<char> {
        if at < self.chars.len() {
            let ch = self.chars.remove(at);
            self.render = self.chars.clone();
            Some(ch)
        } else {
            None
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
    pub fn delete_char(&mut self, row: usize, col: usize) -> Option<char> {
        if let Some(r) = self.rows.get_mut(row) {
            r.delete_char(col)
        } else {
            None
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

    /// 指定行を削除して、その行の内容を返す
    pub fn delete_row_with_content(&mut self, at: usize) -> Option<String> {
        if at < self.rows.iter().len() {
            let row = self.rows.remove(at);
            Some(row.chars().to_string())
        } else {
            None
        }
    }

    /// 指定行の内容を取得
    pub fn get_row_content(&self, at: usize) -> Option<String> {
        self.rows.get(at).map(|r| r.chars().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Row のテスト
    #[test]
    fn test_row_new() {
        let row = Row::new("hello".to_string());
        assert_eq!(row.chars(), "hello");
        assert_eq!(row.len(), 5);
    }

    #[test]
    fn test_row_insert_char() {
        let mut row = Row::new("helo".to_string());
        row.insert_char(2, 'l');
        assert_eq!(row.chars(), "hello");
    }

    #[test]
    fn test_row_insert_str() {
        let mut row = Row::new("heo".to_string());
        row.insert_str(2, "ll");
        assert_eq!(row.chars(), "hello");
    }

    #[test]
    fn test_row_delete_char() {
        let mut row = Row::new("hello".to_string());
        let ch = row.delete_char(4);
        assert_eq!(ch, Some('o'));
        assert_eq!(row.chars(), "hell");
    }

    #[test]
    fn test_row_delete_char_out_of_bounds() {
        let mut row = Row::new("hi".to_string());
        let ch = row.delete_char(5);
        assert_eq!(ch, None);
        assert_eq!(row.chars(), "hi");
    }

    #[test]
    fn test_row_split_off() {
        let mut row = Row::new("hello".to_string());
        let tail = row.split_off(2);
        assert_eq!(row.chars(), "he");
        assert_eq!(tail, "llo");
    }

    #[test]
    fn test_row_append() {
        let mut row = Row::new("hello".to_string());
        row.append(" world");
        assert_eq!(row.chars(), "hello world");
    }

    // Buffer のテスト
    #[test]
    fn test_buffer_new() {
        let buffer = Buffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_insert_row() {
        let mut buffer = Buffer::new();
        buffer.insert_row(0, "first".to_string());
        buffer.insert_row(1, "second".to_string());
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.row(0).unwrap().chars(), "first");
        assert_eq!(buffer.row(1).unwrap().chars(), "second");
    }

    #[test]
    fn test_buffer_delete_row_with_content() {
        let mut buffer = Buffer::new();
        buffer.insert_row(0, "line1".to_string());
        buffer.insert_row(1, "line2".to_string());

        let content = buffer.delete_row_with_content(0);
        assert_eq!(content, Some("line1".to_string()));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "line2");
    }

    #[test]
    fn test_buffer_insert_char() {
        let mut buffer = Buffer::new();
        buffer.insert_char(0, 0, 'a');
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "a");

        buffer.insert_char(0, 1, 'b');
        assert_eq!(buffer.row(0).unwrap().chars(), "ab");
    }

    #[test]
    fn test_buffer_insert_newline() {
        let mut buffer = Buffer::new();
        buffer.insert_row(0, "hello".to_string());
        buffer.insert_newline(0, 2);

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.row(0).unwrap().chars(), "he");
        assert_eq!(buffer.row(1).unwrap().chars(), "llo");
    }

    #[test]
    fn test_buffer_join_rows() {
        let mut buffer = Buffer::new();
        buffer.insert_row(0, "hello".to_string());
        buffer.insert_row(1, " world".to_string());

        buffer.join_rows(1);

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "hello world");
    }
}
