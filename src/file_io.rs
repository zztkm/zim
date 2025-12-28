use std::{
    fs::{File, read},
    io::{self, BufRead, BufReader},
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
}
