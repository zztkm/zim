use std::io::{self, Stdout, Write};

use termion::raw::{IntoRawMode, RawTerminal};

pub struct Terminal {
    stdout: RawTerminal<Stdout>,
    size: (u16, u16),
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let raw_size = termion::terminal_size()?;
        // PTY サイズが (0, 0) の場合（テスト環境等）はデフォルトサイズを使用
        let size = if raw_size.0 == 0 || raw_size.1 == 0 {
            (80, 24)
        } else {
            raw_size
        };
        Ok(Self { stdout, size })
    }

    pub fn stdout(&mut self) -> &mut RawTerminal<Stdout> {
        &mut self.stdout
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn rows(&self) -> u16 {
        self.size.1
    }

    pub fn cols(&self) -> u16 {
        self.size.0
    }

    pub fn clear_screen(&mut self) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;
        self.stdout.flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // カーソルスタイルをリセット
        let _ = write!(self.stdout, "{}", termion::cursor::SteadyBlock);
        // 終了時の画面クリア
        let _ = self.clear_screen();
    }
}
