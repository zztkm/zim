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
    command_buffer: &mut String,
    editor_rows: u16,
) -> HandlerResult {
    match key {
        Key::Char('\n') => {
            let parts: Vec<&str> = command_buffer.split_whitespace().collect();
            let cmd = parts.first().copied().unwrap_or("");

            // コマンド実行
            let result = match cmd {
                "q" => {
                    // 未保存の変更がある場合は警告
                    if editor.is_dirty() {
                        mode_manager.enter_normal();
                        command_buffer.clear();
                        return HandlerResult::StatusMessage(
                            "No write since last change (add ! to override)".to_string(),
                        );
                    } else {
                        return HandlerResult::Quit;
                    }
                }
                "q!" => {
                    return HandlerResult::Quit;
                }
                "w" => {
                    let msg = match editor.save() {
                        Ok(_) => {
                            let bytes = editor
                                .buffer()
                                .rows()
                                .iter()
                                .map(|r| r.chars().len())
                                .sum::<usize>();
                            format!(
                                "\"{}\" {}L {}B written",
                                editor.filename().unwrap_or("[No Name]"),
                                editor.buffer().len(),
                                bytes
                            )
                        }
                        Err(e) => {
                            format!("Error: {}", e)
                        }
                    };
                    mode_manager.enter_normal();
                    command_buffer.clear();
                    HandlerResult::StatusMessage(msg)
                }
                "wq" => match editor.save() {
                    Ok(_) => return HandlerResult::Quit,
                    Err(e) => {
                        mode_manager.enter_normal();
                        command_buffer.clear();
                        HandlerResult::StatusMessage(format!("Error: {}", e))
                    }
                },
                "e" | "e!" => {
                    let force = cmd == "e!";
                    let msg = if let Some(filename) = parts.get(1) {
                        if !force && editor.is_dirty() {
                            "No write since last change (add ! to override)".to_string()
                        } else {
                            match editor.open_file(filename.to_string()) {
                                Ok(_) => {
                                    *cursor = Cursor::new();
                                    format!("\"{}\" loaded", filename)
                                }
                                Err(e) => format!("Cannot open file: {}", e),
                            }
                        }
                    } else {
                        // ファイル名なしのパターン
                        if !force && editor.is_dirty() {
                            "No write since last change (add ! to override)".to_string()
                        } else {
                            match editor.reload() {
                                Ok(_) => {
                                    // このときはカーソル位置をリセットしない(いきなり位置が変わるとびっくりするため
                                    let msg = format!(
                                        "\"{}\" reloaded",
                                        editor.filename().unwrap_or("[No Name]")
                                    );

                                    // カーソル位置調整
                                    // (更新前のカーソル位置よりファイルが短くなった場合などに必要
                                    let (buffer_len, line_len) =
                                        editor.buffer_info(cursor.file_row());
                                    cursor.ensure_within_bounds(buffer_len, line_len, editor_rows);

                                    msg
                                }
                                Err(e) => format!("Error: {}", e),
                            }
                        }
                    };
                    mode_manager.enter_normal();
                    command_buffer.clear();
                    HandlerResult::StatusMessage(msg)
                }
                "" => {
                    // 無視
                    mode_manager.enter_normal();
                    command_buffer.clear();
                    HandlerResult::Continue
                }
                _ => {
                    let msg = format!("Not an editor command: {}", command_buffer);
                    mode_manager.enter_normal();
                    command_buffer.clear();
                    HandlerResult::StatusMessage(msg)
                }
            };
            result
        }
        Key::Esc => {
            // コマンドモードをキャンセル
            mode_manager.enter_normal();
            command_buffer.clear();
            HandlerResult::Continue
        }
        Key::Char(c) => {
            command_buffer.push(c);
            HandlerResult::Continue
        }
        Key::Backspace => {
            command_buffer.pop();
            HandlerResult::Continue
        }
        _ => HandlerResult::Continue,
    }
}
