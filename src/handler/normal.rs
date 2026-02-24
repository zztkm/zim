use termion::event::Key;

use crate::cursor::Cursor;
use crate::editor::{Editor, PasteDirection, PasteResult};
use crate::mode::ModeManager;

use super::HandlerResult;

pub fn handle(
    key: Key,
    editor: &mut Editor,
    cursor: &mut Cursor,
    mode_manager: &mut ModeManager,
    pending_key: &mut Option<char>,
    terminal_size: (u16, u16),
    editor_rows: u16,
) -> HandlerResult {
    let mut next_pending_key: Option<char> = None;

    match key {
        Key::Char(':') => {
            mode_manager.enter_command();
        }
        Key::Char('i') => {
            mode_manager.enter_insert();
        }
        Key::Char('I') => {
            // 行頭から Insert mode
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('a') => {
            // カーソルの後ろから Insert mode
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                // Insert mode では行末+1まで移動可能
                cursor.move_right(terminal_size.0, line.char_count() + 1);
            }
            mode_manager.enter_insert();
        }
        Key::Char('A') => {
            // 行末から Insert mode
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                // Insert mode では行末の1つ後ろに配置
                let line_len = line.char_count() as u16;
                if line_len == 0 {
                    cursor.move_to_line_start();
                } else {
                    cursor.move_to_line_end(line_len + 1);
                }
            }
            mode_manager.enter_insert();
        }
        Key::Char('o') => {
            // 下に新しい行を追加して Insert mode
            let row = cursor.file_row();
            editor.buffer_mut().insert_row(row + 1, String::new());
            cursor.move_down(editor_rows, editor.buffer().len());
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('O') => {
            // 上に新しい行を追加して Insert mode
            let row = cursor.file_row();
            editor.buffer_mut().insert_row(row, String::new());
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('x') => {
            let pos = cursor.position();
            if editor.delete_char_at_cursor(pos) {
                // 削除成功後、行末を超えないように調整
                let line_len = editor.current_line_len(pos.row);
                if line_len > 0 && cursor.x() > line_len as u16 {
                    cursor.move_left();
                }
            }
            return HandlerResult::ClearStatus;
        }
        Key::Char('d') => {
            // dd コマンド実行時
            if *pending_key == Some('d') {
                let row = cursor.file_row();
                if editor.delete_line(row) {
                    // 削除成功後、カーソル位置調整
                    let (buffer_len, line_len) = editor.buffer_info(cursor.file_row());
                    cursor.ensure_within_bounds(buffer_len, line_len, editor_rows);
                }
            } else {
                next_pending_key = Some('d');
            }
            *pending_key = next_pending_key;
            return HandlerResult::ClearStatus;
        }
        Key::Char('y') => {
            // yy
            if *pending_key == Some('y') {
                let row = cursor.file_row();
                editor.yank_line(row);
            } else {
                next_pending_key = Some('y');
            }
            *pending_key = next_pending_key;
            return HandlerResult::ClearStatus;
        }
        Key::Char('p') => {
            let pos = cursor.position();

            match editor.paste(pos, PasteDirection::Below) {
                PasteResult::InLine => {
                    let line_len = editor.current_line_len(pos.row);
                    cursor.move_right(terminal_size.0, line_len);
                }
                PasteResult::Below => {
                    cursor.move_down(editor_rows, editor.buffer().len());
                }
                _ => {}
            }
            *pending_key = next_pending_key;
            return HandlerResult::ClearStatus;
        }
        Key::Char('P') => {
            let pos = cursor.position();

            // Above の場合は特にカーソル移動する必要がない
            if let PasteResult::InLine = editor.paste(pos, PasteDirection::Above) {
                let line_len = editor.current_line_len(pos.row);
                cursor.move_right(terminal_size.0, line_len);
            }
            *pending_key = next_pending_key;
            return HandlerResult::ClearStatus;
        }
        // Visual mode 系
        Key::Char('v') => {
            mode_manager.enter_visual(cursor.position());
        }
        Key::Char('V') => {
            mode_manager.enter_visual_line(cursor.position());
        }
        // 移動系
        Key::Char('h') => cursor.move_left(),
        Key::Char('j') => {
            cursor.move_down(editor_rows, editor.buffer().len());
            // 移動後の行に合わせて x 座標を調整する
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                cursor.adjust_cursor_x(line.char_count());
            }
        }
        Key::Char('k') => {
            cursor.move_up();
            // 移動後の行に合わせて x 座標を調整する
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                cursor.adjust_cursor_x(line.char_count());
            }
        }
        Key::Char('l') => {
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                cursor.move_right(terminal_size.0, line.char_count());
            }
        }
        Key::Char('0') => cursor.move_to_line_start(),
        Key::Char('$') => {
            // 現在の行の長さを取得して行末に移動
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                cursor.move_to_line_end(line.char_count() as u16);
            }
        }
        Key::Char('g') => {
            if *pending_key == Some('g') {
                // gg: ファイル先頭に移動する
                cursor.move_to_top();
                // 移動後の行に合わせて x 座標を調整する
                let row = cursor.file_row();
                if let Some(line) = editor.buffer().row(row) {
                    cursor.adjust_cursor_x(line.char_count());
                }
            } else {
                next_pending_key = Some('g');
            }
        }
        Key::Char('G') => {
            cursor.move_to_bottom(editor.buffer().len(), editor_rows);
            // 移動後の行に合わせて x 座標を調整する
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                cursor.adjust_cursor_x(line.char_count());
            }
        }
        _ => {}
    }

    *pending_key = next_pending_key;
    HandlerResult::Continue
}
