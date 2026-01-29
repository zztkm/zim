use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use crate::buffer::Buffer;

pub struct FileIO;

impl FileIO {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<(Buffer, bool)> {
        let content = std::fs::read_to_string(path)?;
        let ends_with_newline = content.ends_with('\n');

        let mut buffer = Buffer::new();

        if !content.is_empty() {
            for (index, line) in content.lines().enumerate() {
                buffer.insert_row(index, line.to_string());
            }
        }

        Ok((buffer, ends_with_newline))
    }

    pub fn save<P: AsRef<Path>>(path: P, buffer: &Buffer, ends_with_newline: bool) -> io::Result<()> {
        let mut file = File::create(path)?;

        if buffer.is_empty() {
            // 空バッファの場合
            if ends_with_newline {
                writeln!(file)?;
            }
            file.flush()?;
            return Ok(());
        }

        for (i, row) in buffer.rows().iter().enumerate() {
            if i < buffer.len() - 1 {
                // 最後の行以外は常に改行を追加
                writeln!(file, "{}", row.chars())?;
            } else {
                // 最後の行
                write!(file, "{}", row.chars())?;
                if ends_with_newline {
                    writeln!(file)?;
                }
            }
        }

        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_with_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line1").unwrap();
        writeln!(file, "line2").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.row(0).unwrap().chars(), "line1");
        assert_eq!(buffer.row(1).unwrap().chars(), "line2");
        assert!(ends_with_newline);
    }

    #[test]
    fn test_file_without_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "line1\nline2").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.row(0).unwrap().chars(), "line1");
        assert_eq!(buffer.row(1).unwrap().chars(), "line2");
        assert!(!ends_with_newline);
    }

    #[test]
    fn test_round_trip_with_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "test").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();
        FileIO::save(file.path(), &buffer, ends_with_newline).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert_eq!(content, "test\n");
    }

    #[test]
    fn test_round_trip_without_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "test").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();
        FileIO::save(file.path(), &buffer, ends_with_newline).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert_eq!(content, "test");
    }

    #[test]
    fn test_empty_file_with_trailing_newline() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        writeln!(std::fs::File::create(&path).unwrap()).unwrap();

        let (buffer, ends_with_newline) = FileIO::open(&path).unwrap();

        // A file with just a newline is one empty line
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "");
        assert!(ends_with_newline);
    }

    #[test]
    fn test_empty_file_without_trailing_newline() {
        let file = NamedTempFile::new().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 0);
        assert!(!ends_with_newline);
    }

    #[test]
    fn test_single_line_with_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "single line").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "single line");
        assert!(ends_with_newline);
    }

    #[test]
    fn test_single_line_without_trailing_newline() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "single line").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.row(0).unwrap().chars(), "single line");
        assert!(!ends_with_newline);
    }

    #[test]
    fn test_multiple_lines_with_empty_last_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line1").unwrap();
        writeln!(file, "line2").unwrap();
        writeln!(file, "").unwrap();
        file.flush().unwrap();

        let (buffer, ends_with_newline) = FileIO::open(file.path()).unwrap();

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.row(0).unwrap().chars(), "line1");
        assert_eq!(buffer.row(1).unwrap().chars(), "line2");
        assert_eq!(buffer.row(2).unwrap().chars(), "");
        assert!(ends_with_newline);
    }
}
