use std::io::{self, Write};
use termion;

use crate::UI_HEIGHT;
use crate::buffer::Buffer;
use crate::cursor::{Cursor, Position};
use crate::mode::Mode;

pub struct Screen;

impl Screen {
    pub fn editor_rows(rows: u16) -> u16 {
        rows.saturating_sub(UI_HEIGHT)
    }

    pub fn draw_rows(
        stdout: &mut impl Write,
        rows: u16,
        cols: u16,
        buffer: &Buffer,
        row_offset: u16,
        selection: Option<(Position, Position)>,
        line_selection: bool,
    ) -> io::Result<()> {
        let editor_rows = Self::editor_rows(rows);

        for i in 0..editor_rows {
            let file_row = (row_offset + i) as usize;

            if file_row < buffer.len() {
                // バッファ内容を表示
                if let Some(row) = buffer.row(file_row) {
                    let text = row.render();
                    let chars: Vec<char> = text.chars().collect();

                    // 選択範囲のハイライト処理
                    if let Some((start, end)) = selection {
                        // 範囲を正規化
                        let (norm_start, norm_end) = if start <= end {
                            (start, end)
                        } else {
                            (end, start)
                        };

                        // この行が選択範囲内かチェック
                        if file_row >= norm_start.row && file_row <= norm_end.row {
                            if line_selection {
                                // 行全体をハイライト
                                write!(stdout, "{}", termion::style::Invert)?;
                                if chars.is_empty() {
                                    write!(stdout, " ")?;
                                } else {
                                    let display: String = if chars.len() > cols as usize {
                                        chars.iter().take(cols as usize).collect()
                                    } else {
                                        text.to_string()
                                    };
                                    write!(stdout, "{}", display)?;
                                }
                                write!(stdout, "{}", termion::style::Reset)?;
                            } else {
                                // 行内の選択範囲を計算
                                let start_col = if file_row == norm_start.row {
                                    norm_start.col
                                } else {
                                    0
                                };

                                let end_col = if file_row == norm_end.row {
                                    norm_end.col.min(chars.len().saturating_sub(1))
                                } else {
                                    chars.len().saturating_sub(1)
                                };

                                // ハイライト表示（before + selected + after が cols を超えないよう管理）
                                let mut remaining = cols as usize;

                                // 選択前
                                let before_len = start_col.min(remaining);
                                let before: String = chars.iter().take(before_len).collect();
                                write!(stdout, "{}", before)?;
                                remaining = remaining.saturating_sub(before_len);

                                // 選択部分（反転）
                                let selected_len =
                                    (end_col.saturating_sub(start_col) + 1).min(remaining);
                                write!(stdout, "{}", termion::style::Invert)?;
                                let selected: String = chars
                                    .iter()
                                    .skip(start_col)
                                    .take(selected_len)
                                    .collect();
                                write!(stdout, "{}", selected)?;
                                write!(stdout, "{}", termion::style::Reset)?;
                                remaining = remaining.saturating_sub(selected_len);

                                // 選択後
                                let after: String =
                                    chars.iter().skip(end_col + 1).take(remaining).collect();
                                write!(stdout, "{}", after)?;
                            }
                        } else {
                            // 選択範囲外の通常表示
                            let display_text: String = if chars.len() > cols as usize {
                                chars.iter().take(cols as usize).collect()
                            } else {
                                text.to_string()
                            };
                            write!(stdout, "{}", display_text)?;
                        }
                    } else {
                        // 選択なしの通常表示
                        let display_text: String = if chars.len() > cols as usize {
                            chars.iter().take(cols as usize).collect()
                        } else {
                            text.to_string()
                        };
                        write!(stdout, "{}", display_text)?;
                    }
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
        cols: u16,
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
        let padding = (cols as usize)
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
            Mode::VisualLine => {
                write!(stdout, "-- VISUAL LINE --")?;
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
        visual_start: Option<Position>,
    ) -> io::Result<()> {
        // カーソルを隠す
        write!(stdout, "{}", termion::cursor::Hide)?;
        // カーソルを左上に移動
        write!(stdout, "{}", termion::cursor::Goto(1, 1))?;

        let size = termion::terminal_size()?;

        // Visual / VisualLine モードの場合は選択範囲を計算
        let (selection, line_selection) = match mode {
            Mode::Visual => (visual_start.map(|start| (start, cursor.position())), false),
            Mode::VisualLine => (visual_start.map(|start| (start, cursor.position())), true),
            _ => (None, false),
        };

        // 行を描画
        Self::draw_rows(
            stdout,
            size.1,
            size.0,
            buffer,
            cursor.row_offset(),
            selection,
            line_selection,
        )?;

        // ステータスバー描画
        Self::draw_status_bar(stdout, filename, buffer.len(), cursor.file_row(), size.0)?;

        // コマンドライン / ステータスライン (最下行)
        Self::draw_command_line(stdout, mode, command_buffer, status_message)?;

        // カーソル位置に移動
        let current_line = buffer
            .row(cursor.file_row())
            .map(|r| r.chars())
            .unwrap_or("");
        match mode {
            Mode::Command => {
                // コマンドモード時はコマンドライン上にカーソル
                write!(
                    stdout,
                    "{}",
                    termion::cursor::Goto((command_buffer.len() as u16) + 2, size.1)
                )?;
            }
            Mode::Normal | Mode::Insert | Mode::Visual | Mode::VisualLine => {
                // 全角文字を考慮した端末カラム位置を使用
                write!(
                    stdout,
                    "{}",
                    termion::cursor::Goto(cursor.screen_col(current_line), cursor.y())
                )?;
            }
        }

        // カーソルスタイルを設定
        match mode {
            Mode::Insert => {
                // Insert モードでは縦棒カーソル
                write!(stdout, "{}", termion::cursor::SteadyBar)?;
            }
            Mode::Normal | Mode::Command | Mode::Visual | Mode::VisualLine => {
                // Normal/Command/Visual モードではブロックカーソル
                write!(stdout, "{}", termion::cursor::SteadyBlock)?;
            }
        }

        // カーソル表示
        write!(stdout, "{}", termion::cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }
}
