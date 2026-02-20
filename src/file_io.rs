use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use crate::buffer::Buffer;

pub struct FileIO;

impl FileIO {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Buffer> {
        let content = std::fs::read_to_string(path)?;
        let trailing_newline = content.ends_with('\n');

        let mut buffer = Buffer::new();
        for (index, line) in content.lines().enumerate() {
            buffer.insert_row(index, line.to_string());
        }
        buffer.set_trailing_newline(trailing_newline);

        Ok(buffer)
    }

    pub fn save<P: AsRef<Path>>(path: P, buffer: &Buffer) -> io::Result<()> {
        // 既存ファイルがある場合は上書きする
        let mut file = File::create(path)?;

        for (i, row) in buffer.rows().iter().enumerate() {
            if i < buffer.len() - 1 || buffer.trailing_newline() {
                writeln!(file, "{}", row.chars())?;
            } else {
                write!(file, "{}", row.chars())?;
            }
        }

        file.flush()?;
        Ok(())
    }
}
