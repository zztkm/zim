use termion::event::Key;

use crate::cursor::{Cursor, Position};
use crate::editor::Editor;
use crate::mode::ModeManager;

use super::HandlerResult;

pub fn handle(
    key: Key,
    editor: &mut Editor,
    cursor: &mut Cursor,
    mode_manager: &mut ModeManager,
    terminal_size: (u16, u16),
    editor_rows: u16,
) -> HandlerResult {
    match key {
        Key::Esc => {
            mode_manager.enter_normal();
            cursor.move_left();
        }
        Key::Char('\n') => {
            // 改行
            let pos = cursor.position();
            editor.insert_newline(pos);
            cursor.move_down(editor_rows, editor.buffer().len());
            cursor.move_to_line_start();
            // TODO:
            // 設定に応じて、改行したときに前の行とインデントを合わせることができるようにする
        }
        Key::Backspace => {
            // 削除
            let pos = cursor.position();

            if pos.col > 0 {
                // 文字を削除
                editor.delete_char(Position::new(pos.row, pos.col - 1));
                cursor.move_left();
            } else if pos.row > 0 {
                // 行頭で Backspace + 前の行と結合
                let prev_row = pos.row - 1;
                let prev_line_len =
                    editor.buffer().row(prev_row).map(|r| r.char_count()).unwrap_or(0);
                editor.join_rows(pos.row);
                cursor.move_up();
                cursor.move_to_line_end((prev_line_len as u16) + 1);
            }
        }
        Key::Char(ch) => {
            // 文字挿入
            let pos = cursor.position();
            editor.insert_char(pos, ch);
            // Insert モードでは行末の次の位置まで移動可能
            cursor.move_right(
                terminal_size.0,
                editor.buffer().row(pos.row).map(|r| r.char_count()).unwrap_or(0) + 1,
            );
        }
        _ => {}
    }
    HandlerResult::Continue
}
