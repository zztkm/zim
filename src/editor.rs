use arboard::Clipboard;

use crate::{buffer::Buffer, cursor::Position, file_io::FileIO};
use std::io;

pub enum PasteDirection {
    // `p`
    Below,
    // `P`
    Above,
}

pub enum PasteResult {
    Empty,
    // カーソルのある行に挿入
    InLine,
    // 上の行に挿入
    Above,
    // 下の行に挿入
    Below,
}

enum YankType {
    /// 行内にペースト
    InLine,
    /// 新しい行としてペースト
    NewLine,
}

struct YankManager {
    buffer: Vec<String>,
    yank_type: YankType,
}

impl YankManager {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            yank_type: YankType::InLine,
        }
    }

    pub fn yank_inline(&mut self, text: String) {
        self.buffer = vec![text];
        self.yank_type = YankType::InLine;
    }

    pub fn yank_line(&mut self, text: String) {
        self.buffer = vec![text];
        self.yank_type = YankType::NewLine;
    }

    pub fn is_newline_yank(&self) -> bool {
        matches!(self.yank_type, YankType::NewLine)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn content(&self) -> &[String] {
        &self.buffer
    }
}

pub struct Editor {
    buffer: Buffer,
    filename: Option<String>,
    /// 未保存の変更があるか
    dirty: bool,
    /// ファイルが末尾改行で終わるか
    ends_with_newline: bool,
    yank_manager: YankManager,
    /// システムクリップボード連携
    clipboard: Option<Clipboard>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            filename: None,
            dirty: false,
            ends_with_newline: true,
            yank_manager: YankManager::new(),
            clipboard: Clipboard::new().ok(),
        }
    }

    pub fn from_buffer(buffer: Buffer, filename: Option<String>) -> Self {
        Self {
            buffer,
            filename,
            dirty: false,
            ends_with_newline: true,
            yank_manager: YankManager::new(),
            clipboard: Clipboard::new().ok(),
        }
    }

    pub fn open_file(&mut self, filename: String) -> io::Result<()> {
        let (buffer, ends_with_newline) = FileIO::open(&filename)?;
        // Editor のプロパティを更新する
        self.buffer = buffer;
        self.filename = Some(filename);
        self.dirty = false;
        self.ends_with_newline = ends_with_newline;
        // yank の状態は継続して良いため、YankManager は意図的に更新していない
        Ok(())
    }

    pub fn reload(&mut self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let (buffer, ends_with_newline) = FileIO::open(filename)?;
            // Editor のプロパティを更新する
            self.buffer = buffer;
            self.dirty = false;
            self.ends_with_newline = ends_with_newline;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file name"))
        }
    }

    pub fn sync_to_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard
            && !self.yank_manager.is_empty()
        {
            let text = self.yank_manager.content().join("\n");
            // set_text に失敗しても無視する
            // TODO: ステータスメッセージに連携するかはあとで検討
            let _ = clipboard.set_text(text);
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// バッファの長さと指定行の長さを取得
    ///
    /// カーソル位置調整時に頻繁に使用される
    ///
    /// # Returns
    ///
    /// (バッファの長さ, 指定行の長さ)
    pub fn buffer_info(&self, row: usize) -> (usize, usize) {
        let buffer_len = self.buffer.len();
        let line_len = if buffer_len > 0 {
            self.buffer
                .row(row.min(buffer_len - 1))
                .map(|r| r.len())
                .unwrap_or(0)
        } else {
            0
        };
        (buffer_len, line_len)
    }

    /// 現在のカーソル位置の行の長さを取得
    pub fn current_line_len(&self, row: usize) -> usize {
        self.buffer.row(row).map(|r| r.len()).unwrap_or(0)
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_ends_with_newline(&mut self, ends_with_newline: bool) {
        self.ends_with_newline = ends_with_newline;
    }

    /// 文字を挿入
    pub fn insert_char(&mut self, pos: Position, ch: char) {
        self.buffer.insert_char(pos, ch);
        self.dirty = true;
    }

    /// 文字を削除
    pub fn delete_char(&mut self, pos: Position) {
        self.buffer.delete_char(pos);
        self.dirty = true;
    }

    /// 改行を挿入
    pub fn insert_newline(&mut self, pos: Position) {
        self.buffer.insert_newline(pos);
        self.dirty = true;
    }

    /// 前の行と結合
    pub fn join_rows(&mut self, row: usize) {
        self.buffer.join_rows(row);
        self.dirty = true;
    }

    /// ファイルに保存
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            FileIO::save(filename, &self.buffer, self.ends_with_newline)?;
            self.dirty = false;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No filename specified",
            ))
        }
    }

    /// カーソル位置の文字を削除する
    pub fn delete_char_at_cursor(&mut self, pos: Position) -> bool {
        if let Some(line) = self.buffer.row(pos.row)
            && pos.col < line.len()
        {
            // 削除文字列を取得できた場合は yank_buffer に入れる
            if let Some(ch) = self.buffer.delete_char(pos) {
                self.yank_manager.yank_inline(ch.to_string());
                self.sync_to_clipboard();
            }
            self.dirty = true;
            return true;
        }
        false
    }

    /// 指定行を削除してヤンクバッファに保存 (dd 用
    pub fn delete_line(&mut self, row: usize) -> bool {
        if let Some(content) = self.buffer.delete_row_with_content(row) {
            self.yank_manager.yank_line(content);
            self.sync_to_clipboard();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// ヤンクバッファにコピーする (yy 用
    pub fn yank_line(&mut self, row: usize) -> bool {
        if let Some(content) = self.buffer.get_row_content(row) {
            self.yank_manager.yank_line(content);
            self.sync_to_clipboard();
            true
        } else {
            false
        }
    }

    /// 範囲ヤンク(Visual mode 用)
    pub fn yank_range(&mut self, start: Position, end: Position) -> bool {
        let yank_lines = self.extract_range_text(start, end);

        if yank_lines.is_empty() {
            return false;
        }

        if yank_lines.len() == 1 {
            // 単一行の場合は inline
            self.yank_manager.yank_inline(yank_lines[0].clone());
        } else {
            for line in yank_lines {
                self.yank_manager.yank_line(line);
            }
        }

        self.sync_to_clipboard();
        true
    }

    /// 範囲削除(Visual mode 用)
    pub fn delete_range(&mut self, start: Position, end: Position) -> bool {
        if !self.yank_range(start, end) {
            return false;
        }

        let (norm_start, norm_end) = Self::normalize_range(start, end);

        if norm_start.row == norm_end.row {
            if let Some(row) = self.buffer.row_mut(norm_start.row) {
                for _ in norm_start.col..=norm_end.col {
                    // 削除されると次の削除対象文字がその index になるため
                    // norm_start.col 固定で良い
                    row.delete_char(norm_start.col);
                }
                self.dirty = true;
                return true;
            }
        } else {
            if let Some(first_row) = self.buffer.row_mut(norm_start.row) {
                let chars: Vec<char> = first_row.chars().chars().collect();
                let remaining: String = chars.iter().take(norm_start.col).collect();
                // 入れ替え
                *first_row = crate::buffer::Row::new(remaining);
            }

            let tail = if let Some(last_row) = self.buffer.row(norm_end.row) {
                let chars: Vec<char> = last_row.chars().chars().collect();
                chars.iter().skip(norm_end.col + 1).collect()
            } else {
                String::new()
            };

            // 中間行と最後の行を削除
            for _ in norm_start.row + 1..=norm_end.row {
                self.buffer.delete_row(norm_start.row + 1);
            }

            // tail を最初の行に結合して文字詰め
            if let Some(first_row) = self.buffer.row_mut(norm_start.row) {
                first_row.append(&tail);
            }

            self.dirty = true;
            return true;
        }
        false
    }

    pub fn normalize_range(start: Position, end: Position) -> (Position, Position) {
        if start <= end {
            (start, end)
        } else {
            (end, start)
        }
    }

    fn is_in_selection(pos: Position, start: Position, end: Position) -> bool {
        let (norm_start, norm_end) = Self::normalize_range(start, end);
        pos >= norm_start && pos <= norm_end
    }

    fn extract_range_text(&self, start: Position, end: Position) -> Vec<String> {
        let (norm_start, norm_end) = Self::normalize_range(start, end);
        let mut result = Vec::new();

        if norm_start.row == norm_end.row {
            // 同じ行内
            if let Some(row) = self.buffer().row(norm_start.row) {
                let chars: Vec<char> = row.chars().chars().collect();
                let text: String = chars
                    .iter()
                    .skip(norm_start.col)
                    .take(norm_end.col - norm_start.col + 1)
                    .collect();
                result.push(text);
            }
        } else {
            // 複数行にまたがる選択
            for row_idx in norm_start.row..=norm_end.row {
                if let Some(row) = self.buffer().row(norm_start.row) {
                    let chars: Vec<char> = row.chars().chars().collect();
                    let text: String = if row_idx == norm_start.row {
                        // 最初の行: start.col から行末まで
                        chars.iter().skip(norm_start.col).collect()
                    } else if row_idx == norm_end.row {
                        // 最終行: 行頭から end.col まで
                        chars.iter().take(norm_end.col + 1).collect()
                    } else {
                        // 中間行
                        row.chars().to_string()
                    };
                    result.push(text);
                }
            }
        }
        result
    }

    pub fn paste(&mut self, pos: Position, direction: PasteDirection) -> PasteResult {
        if self.yank_manager.is_empty() {
            return PasteResult::Empty;
        }

        if self.yank_manager.is_newline_yank() {
            match direction {
                PasteDirection::Below => {
                    for (i, line) in self.yank_manager.content().iter().enumerate() {
                        self.buffer.insert_row(pos.row + i + 1, line.clone());
                    }
                    self.dirty = true;
                    PasteResult::Below
                }
                PasteDirection::Above => {
                    for (i, line) in self.yank_manager.content().iter().enumerate() {
                        self.buffer.insert_row(pos.row + i, line.clone());
                    }
                    self.dirty = true;
                    PasteResult::Above
                }
            }
        } else {
            let col = match direction {
                PasteDirection::Below => pos.col + 1,
                PasteDirection::Above => pos.col,
            };
            if let Some(r) = self.buffer.row_mut(pos.row) {
                // 挿入位置が行末を超えないようにクランプ
                let safe_col = col.min(r.len());
                r.insert_str(safe_col, &self.yank_manager.content()[0]);
                self.dirty = true;
                PasteResult::InLine
            } else {
                PasteResult::Empty
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // YankManager のテスト
    #[test]
    fn test_yank_manager_new() {
        let ym = YankManager::new();
        assert!(ym.is_empty());
        assert!(!ym.is_newline_yank());
    }

    #[test]
    fn test_yank_manager_yank_inline() {
        let mut ym = YankManager::new();
        ym.yank_inline("hello".to_string());

        assert!(!ym.is_empty());
        assert!(!ym.is_newline_yank());
        assert_eq!(ym.content(), &["hello"]);
    }

    #[test]
    fn test_yank_manager_yank_line() {
        let mut ym = YankManager::new();
        ym.yank_line("line content".to_string());

        assert!(!ym.is_empty());
        assert!(ym.is_newline_yank());
        assert_eq!(ym.content(), &["line content"]);
    }

    #[test]
    fn test_yank_manager_type_change() {
        let mut ym = YankManager::new();

        // InLine → NewLine
        ym.yank_inline("char".to_string());
        assert!(!ym.is_newline_yank());

        ym.yank_line("line".to_string());
        assert!(ym.is_newline_yank());

        // NewLine → InLine
        ym.yank_inline("char2".to_string());
        assert!(!ym.is_newline_yank());
    }

    // Editor のテスト
    #[test]
    fn test_editor_new() {
        let editor = Editor::new();
        assert!(editor.buffer().is_empty());
        assert!(!editor.is_dirty());
        assert_eq!(editor.filename(), None);
    }

    #[test]
    fn test_editor_insert_char() {
        let mut editor = Editor::new();
        editor.insert_char(Position::new(0, 0), 'a');

        assert!(editor.is_dirty());
        assert_eq!(editor.buffer().len(), 1);
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "a");
    }

    #[test]
    fn test_editor_delete_line() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "line1".to_string());
        editor.buffer_mut().insert_row(1, "line2".to_string());

        let success = editor.delete_line(0);

        assert!(success);
        assert!(editor.is_dirty());
        assert_eq!(editor.buffer().len(), 1);
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "line2");
        assert!(editor.yank_manager.is_newline_yank());
        assert_eq!(editor.yank_manager.content(), &["line1"]);
    }

    #[test]
    fn test_editor_yank_line() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "content".to_string());

        let success = editor.yank_line(0);

        assert!(success);
        assert!(!editor.is_dirty()); // yank は dirty にしない
        assert_eq!(editor.buffer().len(), 1); // バッファは変更なし
        assert!(editor.yank_manager.is_newline_yank());
        assert_eq!(editor.yank_manager.content(), &["content"]);
    }

    #[test]
    fn test_editor_delete_char_at_cursor() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "hello".to_string());

        let success = editor.delete_char_at_cursor(Position::new(0, 0));

        assert!(success);
        assert!(editor.is_dirty());
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "ello");
        assert!(!editor.yank_manager.is_newline_yank()); // 文字削除は InLine
        assert_eq!(editor.yank_manager.content(), &["h"]);
    }

    #[test]
    fn test_editor_paste_newline_below() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "line1".to_string());
        editor.yank_manager.yank_line("yanked".to_string());

        let result = editor.paste(Position::new(0, 0), PasteDirection::Below);

        assert!(matches!(result, PasteResult::Below));
        assert_eq!(editor.buffer().len(), 2);
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "line1");
        assert_eq!(editor.buffer().row(1).unwrap().chars(), "yanked");
    }

    #[test]
    fn test_editor_paste_newline_above() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "line1".to_string());
        editor.yank_manager.yank_line("yanked".to_string());

        let result = editor.paste(Position::new(0, 0), PasteDirection::Above);

        assert!(matches!(result, PasteResult::Above));
        assert_eq!(editor.buffer().len(), 2);
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "yanked");
        assert_eq!(editor.buffer().row(1).unwrap().chars(), "line1");
    }

    #[test]
    fn test_editor_paste_inline_below() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "helo".to_string());
        editor.yank_manager.yank_inline("l".to_string());

        // col=2 (e の後ろ) で Below なので col+1=3 に挿入
        let result = editor.paste(Position::new(0, 2), PasteDirection::Below);

        assert!(matches!(result, PasteResult::InLine));
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "hello");
    }

    #[test]
    fn test_editor_paste_inline_above() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "helo".to_string());
        editor.yank_manager.yank_inline("l".to_string());

        // col=3 (o の位置) で Above なので col=3 に挿入
        let result = editor.paste(Position::new(0, 3), PasteDirection::Above);

        assert!(matches!(result, PasteResult::InLine));
        assert_eq!(editor.buffer().row(0).unwrap().chars(), "hello");
    }

    #[test]
    fn test_editor_paste_empty() {
        let mut editor = Editor::new();
        editor.buffer_mut().insert_row(0, "line".to_string());

        let result = editor.paste(Position::new(0, 0), PasteDirection::Below);

        assert!(matches!(result, PasteResult::Empty));
        assert_eq!(editor.buffer().len(), 1); // 変更なし
    }

    #[test]
    fn test_normalize_range() {
        let start = Position::new(1, 5);
        let end = Position::new(3, 2);

        let (norm_start, norm_end) = Editor::normalize_range(start, end);
        assert_eq!(norm_start, start);
        assert_eq!(norm_end, end);

        // start, end を反転させていた場合
        let (norm_start, norm_end) = Editor::normalize_range(end, start);
        assert_eq!(norm_start, start);
        assert_eq!(norm_end, end);
    }

    #[test]
    fn test_is_in_selection() {
        let start = Position::new(1, 5);
        let end = Position::new(3, 2);

        // 範囲内チェックテスト
        assert!(Editor::is_in_selection(Position::new(2, 0), start, end));
        assert!(Editor::is_in_selection(Position::new(1, 5), start, end));
        assert!(Editor::is_in_selection(Position::new(3, 2), start, end));

        // 範囲外
        assert!(!Editor::is_in_selection(Position::new(0, 0), start, end));
        assert!(!Editor::is_in_selection(Position::new(4, 0), start, end));
    }
}