use std::io::{self, Write};
use termion;

use crate::cursor::Cursor;
use crate::mode::Mode;

pub struct Screen;

impl Screen {
    pub fn draw_rows(stdout: &mut impl Write, rows: u16) -> io::Result<()> {
        // 最後の行はステータス / コマンドライン用
        for i in 0..rows - 1 {
            write!(stdout, "~")?;
            // 最後の行の手前の行以外は改行する
            if i < rows - 2 {
                write!(stdout, "\r\n")?;
            }
        }
        Ok(())
    }

    pub fn refresh(
        stdout: &mut impl Write,
        cursor: &Cursor,
        mode: &Mode,
        command_buffer: &str,
    ) -> io::Result<()> {
        // カーソルを隠す
        write!(stdout, "{}", termion::cursor::Hide)?;
        // カーソルを左上に移動
        write!(stdout, "{}", termion::cursor::Goto(1, 1))?;

        // 画面描画
        let size = termion::terminal_size()?;
        Self::draw_rows(stdout, size.1)?;

        // コマンドライン / ステータスライン (最下行)
        write!(stdout, "\r\n")?;
        match mode {
            Mode::Command => {
                write!(stdout, ":{}", command_buffer)?;
            }
            Mode::Normal => {
                write!(stdout, " ")?;
            }
        }

        // カーソル位置に移動
        match mode {
            Mode::Command => {
                // コマンドモード時はコマンドライン上にカーソル
                write!(
                    stdout,
                    ":{}",
                    termion::cursor::Goto((command_buffer.len() + 2) as u16, size.1)
                )?;
            }
            Mode::Normal => {
                // ノーマルモード時はエディタ上にカーソル
                write!(stdout, "{}", termion::cursor::Goto(cursor.x(), cursor.y()))?;
            }
        }

        // カーソル表示
        write!(stdout, "{}", termion::cursor::Show)?;

        stdout.flush()?;
        Ok(())
    }
}
