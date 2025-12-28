use std::fmt::format;
use std::io::{self, Write, stdout};
use termion;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::mode::Mode;

pub struct Screen;

impl Screen {
    pub fn draw_rows(
        stdout: &mut impl Write,
        rows: u16,
        buffer: &Buffer,
        row_offset: u16,
    ) -> io::Result<()> {
        // -2 はステータスバー / コマンドライン用
        for i in 0..rows - 2 {
            let file_row = (row_offset + i) as usize;

            if file_row < buffer.len() {
                // バッファ内容を表示
                if let Some(row) = buffer.row(file_row) {
                    let text = row.render();
                    // 画面に収まるように切り詰める（簡易的な処理）
                    let display_text = if text.len() > 80 { &text[..80] } else { text };
                    write!(stdout, "{}", display_text)?;
                }
            } else {
                // ファイルの終端を超えたら ~ を表示
                write!(stdout, "~")?;
            }

            if i < rows - 3 {
                write!(stdout, "\r\n")?;
            }
        }
        Ok(())
    }

    pub fn draw_status_bar(
        stdout: &mut impl Write,
        filename: Option<&str>,
        buffer_len: usize,
        cursor_file_row: usize,
    ) -> io::Result<()> {
        // ステータスバー（反転表示）
        write!(stdout, "\r\n{}", termion::style::Invert)?;

        let name = filename.unwrap_or("[No Name]");
        let status = format!("{} - {} lines", name, buffer_len);
        write!(stdout, "{}", status)?;

        // 現在の行番号の右端に表示
        let pos = format!(" {}/{} ", cursor_file_row + 1, buffer_len);
        let padding = 80usize.saturating_sub(status.len().saturating_sub(pos.len()));
        write!(stdout, "{}{}", " ".repeat(padding), pos)?;

        write!(stdout, "{}", termion::style::Reset)?;
        Ok(())
    }

    pub fn refresh(
        stdout: &mut impl Write,
        cursor: &Cursor,
        mode: &Mode,
        command_buffer: &str,
        buffer: &Buffer,
        filename: Option<&str>,
    ) -> io::Result<()> {
        // カーソルを隠す
        write!(stdout, "{}", termion::cursor::Hide)?;
        // カーソルを左上に移動
        write!(stdout, "{}", termion::cursor::Goto(1, 1))?;

        let size = termion::terminal_size()?;

        // 行を描画
        Self::draw_rows(stdout, size.1, buffer, cursor.row_offset())?;

        // ステータスバー描画
        Self::draw_status_bar(stdout, filename, buffer.len(), cursor.file_row())?;

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
