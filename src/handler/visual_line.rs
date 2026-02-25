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
    editor_rows: u16,
) -> HandlerResult {
    match key {
        Key::Esc => {
            mode_manager.enter_normal();
            mode_manager.clear_visual();
            return HandlerResult::ClearStatus;
        }
        Key::Char('j') => {
            cursor.move_down(editor_rows, editor.buffer().len());
        }
        Key::Char('k') => {
            cursor.move_up();
        }
        Key::Char('y') => {
            if let Some(start) = mode_manager.visual_start() {
                let end = cursor.position();
                editor.yank_lines_range(start.row, end.row);
                mode_manager.enter_normal();
                mode_manager.clear_visual();
                return HandlerResult::StatusMessage("Yanked lines".to_string());
            }
        }
        Key::Char('d') => {
            if let Some(start) = mode_manager.visual_start() {
                editor.history.commit(editor.snapshot(cursor));
                let end = cursor.position();
                let min_row = start.row.min(end.row);
                if editor.delete_lines_range(start.row, end.row) {
                    // カーソルを min_row か、バッファ末尾のいずれか小さい方へ
                    let buffer_len = editor.buffer().len();
                    let target_row = min_row.min(buffer_len.saturating_sub(1));
                    let current_row = cursor.file_row();
                    if target_row < current_row {
                        for _ in 0..(current_row - target_row) {
                            cursor.move_up();
                        }
                    } else if target_row > current_row {
                        for _ in 0..(target_row - current_row) {
                            cursor.move_down(editor_rows, editor.buffer().len());
                        }
                    }
                    cursor.move_to_line_start();
                }
                mode_manager.enter_normal();
                mode_manager.clear_visual();
                return HandlerResult::StatusMessage("Deleted lines".to_string());
            }
        }
        _ => {}
    }
    HandlerResult::Continue
}
