use std::io;

use crate::{buffer::Buffer, file_io::FileIO};

pub struct Editor {
    buffer: Buffer,
    filename: Option<String>,
    /// 未保存の変更があるか
    dirty: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            filename: None,
            dirty: false,
        }
    }

    pub fn from_buffer(buffer: Buffer, filename: Option<String>) -> Self {
        Self {
            buffer,
            filename,
            dirty: false,
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
}
