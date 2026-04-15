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
        Key::Char('u') => {
            let current = editor.snapshot(cursor);
            if let Some(prev) = editor.history.undo(current) {
                editor.restore_snapshot(prev, cursor);
                let (buf_len, line_len) = editor.buffer_info(cursor.file_row());
                cursor.ensure_within_bounds(buf_len, line_len, editor_rows);
                return HandlerResult::StatusMessage("1 change; before #1".to_string());
            }
            return HandlerResult::StatusMessage("Already at oldest change".to_string());
        }
        Key::Ctrl('r') => {
            let current = editor.snapshot(cursor);
            if let Some(next) = editor.history.redo(current) {
                editor.restore_snapshot(next, cursor);
                let (buf_len, line_len) = editor.buffer_info(cursor.file_row());
                cursor.ensure_within_bounds(buf_len, line_len, editor_rows);
                return HandlerResult::StatusMessage("1 change".to_string());
            }
            return HandlerResult::StatusMessage("Already at newest change".to_string());
        }
        Key::Char('i') => {
            editor.history.commit(editor.snapshot(cursor));
            mode_manager.enter_insert();
        }
        Key::Char('I') => {
            // 行頭から Insert mode
            editor.history.commit(editor.snapshot(cursor));
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('a') => {
            // カーソルの後ろから Insert mode
            editor.history.commit(editor.snapshot(cursor));
            let row = cursor.file_row();
            if let Some(line) = editor.buffer().row(row) {
                // Insert mode では行末+1まで移動可能
                cursor.move_right(terminal_size.0, line.char_count() + 1);
            }
            mode_manager.enter_insert();
        }
        Key::Char('A') => {
            // 行末から Insert mode
            editor.history.commit(editor.snapshot(cursor));
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
            // スナップショットはバッファ変更前に取得する
            editor.history.commit(editor.snapshot(cursor));
            let row = cursor.file_row();
            editor.buffer_mut().insert_row(row + 1, String::new());
            cursor.move_down(editor_rows, editor.buffer().len());
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('O') => {
            // 上に新しい行を追加して Insert mode
            // スナップショットはバッファ変更前に取得する
            editor.history.commit(editor.snapshot(cursor));
            let row = cursor.file_row();
            editor.buffer_mut().insert_row(row, String::new());
            cursor.move_to_line_start();
            mode_manager.enter_insert();
        }
        Key::Char('x') => {
            editor.history.commit(editor.snapshot(cursor));
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
                editor.history.commit(editor.snapshot(cursor));
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
            editor.history.commit(editor.snapshot(cursor));
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
            editor.history.commit(editor.snapshot(cursor));
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

#[cfg(test)]
mod tests {
    use super::handle;
    use termion::event::Key;
    use crate::buffer::Buffer;
    use crate::cursor::Cursor;
    use crate::editor::Editor;
    use crate::handler::HandlerResult;
    use crate::mode::ModeManager;

    fn make_editor_with_lines(lines: &[&str]) -> Editor {
        let mut buffer = Buffer::new();
        for (i, line) in lines.iter().enumerate() {
            buffer.insert_row(i, line.to_string());
        }
        Editor::from_buffer(buffer, None)
    }

    fn send_key(
        key: Key,
        editor: &mut Editor,
        cursor: &mut Cursor,
        mode_manager: &mut ModeManager,
        pending_key: &mut Option<char>,
    ) -> HandlerResult {
        let terminal_size = (80u16, 24u16);
        let editor_rows = 22u16; // 24 - UI_HEIGHT(2)
        handle(key, editor, cursor, mode_manager, pending_key, terminal_size, editor_rows)
    }

    #[test]
    fn test_dd_deletes_correct_line() {
        let mut editor = make_editor_with_lines(&["aaa", "bbb", "ccc", "ddd", "eee"]);
        let mut cursor = Cursor::new();
        let mut mode_manager = ModeManager::new();
        let mut pending_key: Option<char> = None;

        // j を 2 回押して "ccc" (row index 2) に移動
        send_key(Key::Char('j'), &mut editor, &mut cursor, &mut mode_manager, &mut pending_key);
        send_key(Key::Char('j'), &mut editor, &mut cursor, &mut mode_manager, &mut pending_key);

        assert_eq!(cursor.file_row(), 2, "cursor should be on row index 2 (ccc)");

        // dd: d を 2 回押す
        send_key(Key::Char('d'), &mut editor, &mut cursor, &mut mode_manager, &mut pending_key);
        assert_eq!(pending_key, Some('d'), "after first d, pending_key should be Some('d')");
        send_key(Key::Char('d'), &mut editor, &mut cursor, &mut mode_manager, &mut pending_key);

        // "ccc" が削除されて 4 行になっているはず
        assert_eq!(editor.buffer().len(), 4, "buffer should have 4 lines after dd");
        assert_eq!(editor.buffer().row(0).map(|r| r.chars()), Some("aaa"));
        assert_eq!(editor.buffer().row(1).map(|r| r.chars()), Some("bbb"));
        assert_eq!(editor.buffer().row(2).map(|r| r.chars()), Some("ddd"), "ccc should be deleted");
        assert_eq!(editor.buffer().row(3).map(|r| r.chars()), Some("eee"));
    }
}
