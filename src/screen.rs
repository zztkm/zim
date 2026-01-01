use std::io::{self, Write};
use termion;

use crate::UI_HEIGHT;
use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::mode::Mode;

pub struct Screen;

impl Screen {
    pub fn editor_rows(rows: u16) -> u16 {
        rows.saturating_sub(UI_HEIGHT)
    }

    pub fn draw_rows(
        stdout: &mut impl Write,
        rows: u16,
        buffer: &Buffer,
        row_offset: u16,
    ) -> io::Result<()> {
        let editor_rows = Self::editor_rows(rows);

        for i in 0..editor_rows {
            let file_row = (row_offset + i) as usize;

            if file_row < buffer.len() {
                // バッファ内容を表示
                if let Some(row) = buffer.row(file_row) {
                    let text = row.render();
                    // 画面に収まるように切り詰める（文字単位で安全に処理）
                    let chars: Vec<char> = text.chars().collect();
                    let display_text: String = if chars.len() > 80 {
                        chars.iter().take(80).collect()
                    } else {
                        text.to_string()
                    };
                    write!(stdout, "{}", display_text)?;
                }
                // 行末までクリア
                write!(stdout, "{}", termion::clear::UntilNewline)?;
            } else {
                // ファイルの終端を超えたら ~ を表示
                write!(stdout, "~")?;
                write!(stdout, "{}", termion::clear::UntilNewline)?;
            }

            if i < editor_rows - 1 {
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
        let current_line = if buffer_len > 0 {
            cursor_file_row + 1
        } else {
            0
        };
        let pos = format!(" {}/{} ", current_line, buffer_len);
        let padding = 80usize
            .saturating_sub(status.len())
            .saturating_sub(pos.len());
        write!(stdout, "{}{}", " ".repeat(padding), pos)?;

        write!(stdout, "{}", termion::style::Reset)?;
        Ok(())
    }

    pub fn draw_command_line(
        stdout: &mut impl Write,
        mode: Mode,
        command_buffer: &str,
        status_message: &str,
    ) -> io::Result<()> {
        write!(stdout, "\r\n")?;
        // 行をクリアしてから描画
        write!(stdout, "{}", termion::clear::CurrentLine)?;
        match mode {
            Mode::Command => {
                // コマンドバッファをそのまま表示（: は含まれていない前提）
                write!(stdout, ":{}", command_buffer)?;
            }
            Mode::Normal => {
                write!(stdout, "{}", status_message)?;
            }
            Mode::Insert => {
                write!(stdout, "-- INSERT --")?;
            }
            Mode::Visual => {
                write!(stdout, "-- Visual --")?;
            }
        }
        Ok(())
    }

    pub fn refresh(
        stdout: &mut impl Write,
        cursor: &Cursor,
        mode: Mode,
        command_buffer: &str,
        buffer: &Buffer,
        filename: Option<&str>,
        status_message: &str,
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
        Self::draw_command_line(stdout, mode, command_buffer, status_message)?;

        // カーソル位置に移動
        match mode {
            Mode::Command => {
                // コマンドモード時はコマンドライン上にカーソル
                write!(
                    stdout,
                    "{}",
                    termion::cursor::Goto((command_buffer.len() as u16) + 2, size.1)
                )?;
            }
            Mode::Normal | Mode::Insert | Mode::Visual => {
                // ノーマルモード時はエディタ上にカーソル
                write!(stdout, "{}", termion::cursor::Goto(cursor.x(), cursor.y()))?;
            }
        }

        // カーソルスタイルを設定
        match mode {
            Mode::Insert => {
                // Insert モードでは縦棒カーソル
                write!(stdout, "{}", termion::cursor::SteadyBar)?;
            }
            Mode::Normal | Mode::Command | Mode::Visual => {
                // Normal/Command モードではブロックカーソル
                write!(stdout, "{}", termion::cursor::SteadyBlock)?;
            }
        }

        // カーソル表示
        write!(stdout, "{}", termion::cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }
}
