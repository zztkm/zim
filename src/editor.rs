use crate::{buffer::Buffer, file_io::FileIO};
use std::io;

pub struct Editor {
    buffer: Buffer,
    filename: Option<String>,
    /// 未保存の変更があるか
    dirty: bool,
    /// ヤンクバッファ (コピー or 削除した行を保存)
    yank_buffer: Vec<String>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            filename: None,
            dirty: false,
            yank_buffer: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: Buffer, filename: Option<String>) -> Self {
        Self {
            buffer,
            filename,
            dirty: false,
            yank_buffer: Vec::new(),
        }
    }

    pub fn open_file(&mut self, filename: String) -> io::Result<()> {
        let buffer = FileIO::open(&filename)?;
        // Editor のプロパティを更新する
        self.buffer = buffer;
        self.filename = Some(filename);
        self.dirty = false;
        Ok(())
    }

    pub fn reload(&mut self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let buffer = FileIO::open(&filename)?;
            // Editor のプロパティを更新する
            self.buffer = buffer;
            self.dirty = false;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file name"))
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// 文字を挿入
    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        self.buffer.insert_char(row, col, ch);
        self.dirty = true;
    }

    /// 文字を削除
    pub fn delete_char(&mut self, row: usize, col: usize) {
        self.buffer.delete_char(row, col);
        self.dirty = true;
    }

    /// 改行を挿入
    pub fn insert_newline(&mut self, row: usize, col: usize) {
        self.buffer.insert_newline(row, col);
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
            FileIO::save(filename, &self.buffer)?;
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
    pub fn delete_char_at_cursor(&mut self, row: usize, col: usize) -> bool {
        if let Some(line) = self.buffer.row(row) {
            if col < line.len() {
                // 削除文字列を取得できた場合は yank_buffer に入れる
                if let Some(ch) = self.buffer.delete_char(row, col) {
                    self.yank_buffer = vec![ch.to_string()];
                }
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// 指定行を削除してヤンクバッファに保存 (dd 用
    pub fn delete_line(&mut self, row: usize) -> bool {
        if let Some(content) = self.buffer.delete_row_with_content(row) {
            self.yank_buffer = vec![content];
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// ヤンクバッファにコピーする (yy 用
    pub fn yank_line(&mut self, row: usize) -> bool {
        if let Some(content) = self.buffer.get_row_content(row) {
            self.yank_buffer = vec![content];
            true
        } else {
            false
        }
    }

    /// ヤンクバッファの内容を指定行の下にペースト (p)
    pub fn paste_below(&mut self, row: usize) -> bool {
        if self.yank_buffer.is_empty() {
            return false;
        }

        for (i, line) in self.yank_buffer.iter().enumerate() {
            self.buffer.insert_row(row + i + 1, line.clone());
        }
        self.dirty = true;
        true
    }

    /// ヤンクバッファの内容を指定行の上にペースト (P)
    pub fn paste_above(&mut self, row: usize) -> bool {
        if self.yank_buffer.is_empty() {
            return false;
        }

        for (i, line) in self.yank_buffer.iter().enumerate() {
            self.buffer.insert_row(row + i, line.clone());
        }
        self.dirty = true;
        true
    }
}
