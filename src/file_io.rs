use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use crate::buffer::Buffer;

pub struct FileIO;

impl FileIO {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Buffer> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut buffer = Buffer::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            buffer.insert_row(index, line);
        }

        Ok(buffer)
    }

    pub fn save<P: AsRef<Path>>(path: P, buffer: &Buffer) -> io::Result<()> {
        // 既存ファイルがある場合は上書きする
        let mut file = File::create(path)?;

        for (i, row) in buffer.rows().iter().enumerate() {
            // 最後の行以外は改行を追加
            if i < buffer.len() - 1 {
                writeln!(file, "{}", row.chars())?;
            } else {
                write!(file, "{}", row.chars())?;
            }
        }

        file.flush()?;
        Ok(())
    }
}
