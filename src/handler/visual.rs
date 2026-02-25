use termion::event::Key;

use crate::cursor::Cursor;
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
            mode_manager.clear_visual();
            return HandlerResult::ClearStatus;
        }
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
        Key::Char('y') => {
            // ヤンク
            if let Some(start) = mode_manager.visual_start() {
                let end = cursor.position();
                editor.yank_range(start, end);
                mode_manager.enter_normal();
                mode_manager.clear_visual();
                return HandlerResult::StatusMessage("Yanked selection".to_string());
            }
        }
        Key::Char('d') => {
            // 削除してヤンク
            if let Some(start) = mode_manager.visual_start() {
                editor.history.commit(editor.snapshot(cursor));
                let end = cursor.position();
                if editor.delete_range(start, end) {
                    // 削除後、カーソルを範囲の開始位置に移動
                    let (norm_start, _) = Editor::normalize_range(start, end);

                    // カーソルを norm_start の位置に合わせる
                    // row の移動
                    let current_row = cursor.file_row();
                    if norm_start.row < current_row {
                        for _ in 0..(current_row - norm_start.row) {
                            cursor.move_up();
                        }
                    } else if norm_start.row > current_row {
                        for _ in 0..(norm_start.row - current_row) {
                            cursor.move_down(editor_rows, editor.buffer().len());
                        }
                    }

                    // col の移動
                    cursor.move_to_line_start();
                    let line_len = editor.current_line_len(norm_start.row);
                    for _ in 0..norm_start.col.min(line_len) {
                        cursor.move_right(terminal_size.0, line_len);
                    }

                    cursor.scroll(editor_rows, editor.buffer().len());
                }
                mode_manager.enter_normal();
                mode_manager.clear_visual();
                return HandlerResult::StatusMessage("Deleted selection".to_string());
            }
        }
        _ => {}
    }
    HandlerResult::Continue
}
