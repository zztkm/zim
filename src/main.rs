use std::io::{self};

use termion::{event::Key, input::TermRead};
use zim::{
    cursor::Cursor, editor::Editor, file_io::FileIO, mode::ModeManager, screen::Screen,
    terminal::Terminal,
};

fn main() -> io::Result<()> {
    // ターミナル初期化
    let mut terminal = Terminal::new()?;
    terminal.clear_screen()?;

    // コマンドライン引数からファイル名を取得する
    let args: Vec<String> = std::env::args().collect();
    let mut editor = if args.iter().len() > 1 {
        let path = &args[1];
        match FileIO::open(path) {
            Ok(buf) => Editor::from_buffer(buf, Some(path.clone())),
            Err(e) => {
                eprintln!("Error opening file: {}", e);
                return Err(e);
            }
        }
    } else {
        Editor::new()
    };

    // 状態初期化
    let mut cursor = Cursor::new();
    let mut mode_manager = ModeManager::new();
    let mut command_buffer = String::new();
    let mut pending_key: Option<char> = None;
    let prev_mode = mode_manager.current();
    let mut status_message = String::new();

    // 初期描画
    Screen::refresh(
        terminal.stdout(),
        &cursor,
        mode_manager.current(),
        &command_buffer,
        editor.buffer(),
        editor.filename(),
        &status_message,
    )?;

    // main loop
    let stdin = io::stdin();
    let size = terminal.size();
    let editor_rows = Screen::editor_rows(size.1);

    for key in stdin.keys() {
        let mut next_pending_key: Option<char> = None;

        if mode_manager.is_normal() {
            match key? {
                Key::Char(':') => {
                    mode_manager.enter_command();
                    command_buffer.clear();
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
                        cursor.move_right(size.0, line.len() + 1);
                    }
                    mode_manager.enter_insert();
                }
                Key::Char('A') => {
                    // 行末から Insert mode
                    let row = cursor.file_row();
                    if let Some(line) = editor.buffer().row(row) {
                        // Insert mode では行末の1つ後ろに配置
                        let line_len = line.len() as u16;
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
                    let row = cursor.file_row();
                    let col = (cursor.x() - 1) as usize;
                    if editor.delete_char_at_cursor(row, col) {
                        // 削除成功後、行末を超えないように調整
                        if let Some(line) = editor.buffer().row(row) {
                            if line.len() > 0 && cursor.x() > line.len() as u16 {
                                cursor.move_left();
                            }
                        }
                    }
                    status_message.clear();
                }
                Key::Char('d') => {
                    // dd コマンド実行時
                    if pending_key == Some('d') {
                        let row = cursor.file_row();
                        if editor.delete_line(row) {
                            // 削除成功後、カーソル位置調整
                            let buffer_len = editor.buffer().len();
                            let line_len = if buffer_len > 0 {
                                editor
                                    .buffer()
                                    .row(cursor.file_row().min(buffer_len - 1))
                                    .map(|r| r.len())
                                    .unwrap_or(0)
                            } else {
                                0
                            };
                            cursor.ensure_within_bounds(buffer_len, line_len, editor_rows);
                        }
                    } else {
                        next_pending_key = Some('d');
                    }
                    status_message.clear();
                }
                Key::Char('y') => {
                    // yy
                    if pending_key == Some('y') {
                        let row = cursor.file_row();
                        editor.yank_line(row);
                    } else {
                        next_pending_key = Some('y');
                    }
                    status_message.clear();
                }
                Key::Char('p') => {
                    let row = cursor.file_row();
                    if editor.paste_below(row) {
                        cursor.move_down(editor_rows, editor.buffer().len());
                    }
                    status_message.clear();
                }
                Key::Char('P') => {
                    let row = cursor.file_row();
                    editor.paste_above(row);
                    status_message.clear();
                }
                Key::Char('h') => cursor.move_left(),
                Key::Char('j') => {
                    cursor.move_down(editor_rows, editor.buffer().len());
                    // 移動後の行に合わせて x 座標を調整する
                    let row = cursor.file_row();
                    if let Some(line) = editor.buffer().row(row) {
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                Key::Char('k') => {
                    cursor.move_up();
                    // 移動後の行に合わせて x 座標を調整する
                    let row = cursor.file_row();
                    if let Some(line) = editor.buffer().row(row) {
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                Key::Char('l') => {
                    let row = cursor.file_row();
                    if let Some(line) = editor.buffer().row(row) {
                        cursor.move_right(size.0, line.len());
                    }
                }
                Key::Char('0') => cursor.move_to_line_start(),
                Key::Char('$') => {
                    // 現在の行の長さを取得して行末に移動
                    let row = cursor.file_row();
                    if let Some(line) = editor.buffer().row(row) {
                        cursor.move_to_line_end(line.len() as u16);
                    }
                }
                Key::Char('g') => {
                    if pending_key == Some('g') {
                        // gg: ファイル先頭に移動する
                        cursor.move_to_top();
                        // 移動後の行に合わせて x 座標を調整する
                        let row = cursor.file_row();
                        if let Some(line) = editor.buffer().row(row) {
                            cursor.adjust_cursor_x(line.len());
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
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                _ => {}
            }
        } else if mode_manager.is_command() {
            match key? {
                Key::Char('\n') => {
                    let parts: Vec<&str> = command_buffer.split_whitespace().collect();
                    let cmd = parts.get(0).copied().unwrap_or("");

                    // コマンド実行
                    match cmd {
                        "q" => {
                            // 未保存の変更がある場合は警告
                            if editor.is_dirty() {
                                status_message =
                                    "No write since last change (add ! to override)".to_string();
                                mode_manager.enter_normal();
                                command_buffer.clear();
                            } else {
                                break;
                            }
                        }
                        "q!" => {
                            break;
                        }
                        "w" => {
                            match editor.save() {
                                Ok(_) => {
                                    let bytes = editor
                                        .buffer()
                                        .rows()
                                        .iter()
                                        .map(|r| r.chars().len())
                                        .sum::<usize>();
                                    status_message = format!(
                                        "\"{}\" {}L {}B written",
                                        editor.filename().unwrap_or("[No Name]"),
                                        editor.buffer().len(),
                                        bytes
                                    )
                                }
                                Err(e) => {
                                    status_message = format!("Error: {}", e);
                                }
                            }
                            mode_manager.enter_normal();
                            command_buffer.clear();
                        }
                        "wq" => match editor.save() {
                            Ok(_) => break,
                            Err(e) => {
                                status_message = format!("Error: {}", e);
                                mode_manager.enter_normal();
                                command_buffer.clear();
                            }
                        },
                        "e" | "e!" => {
                            let force = cmd == "e!";

                            if let Some(filename) = parts.get(1) {
                                if !force && editor.is_dirty() {
                                    status_message =
                                        "No write since last change (add ! to override)"
                                            .to_string();
                                } else {
                                    match editor.open_file(filename.to_string()) {
                                        Ok(_) => {
                                            status_message = format!("\"{}\" loaded", filename);
                                            cursor = Cursor::new();
                                        }
                                        Err(e) => {
                                            status_message = format!("Cannot open file: {}", e)
                                        }
                                    }
                                }
                            } else {
                                // ファイル名なしのパターン
                                if !force && editor.is_dirty() {
                                    status_message =
                                        "No write since last change (add ! to override)"
                                            .to_string();
                                } else {
                                    match editor.reload() {
                                        Ok(_) => {
                                            // このときはカーソル位置をリセットしない(いきなり位置が変わるとびっくりするため
                                            status_message = format!(
                                                "\"{}\" reloaded",
                                                editor.filename().unwrap_or("[No Name]")
                                            );

                                            // カーソル位置調整
                                            // (更新前のカーソル位置よりファイルが短くなった場合などに必要
                                            let buffer_len = editor.buffer().len();
                                            let line_len = if buffer_len > 0 {
                                                editor
                                                    .buffer()
                                                    .row(cursor.file_row().min(buffer_len - 1))
                                                    .map(|r| r.len())
                                                    .unwrap_or(0)
                                            } else {
                                                0
                                            };
                                            cursor.ensure_within_bounds(
                                                buffer_len,
                                                line_len,
                                                editor_rows,
                                            );
                                        }
                                        Err(e) => status_message = format!("Error: {}", e),
                                    }
                                }
                            }
                            mode_manager.enter_normal();
                            command_buffer.clear();
                        }
                        "" => {
                            // 無視
                            mode_manager.enter_normal();
                            command_buffer.clear();
                        }
                        _ => {
                            status_message = format!("Not an editor command: {}", command_buffer);
                            mode_manager.enter_normal();
                            command_buffer.clear();
                        }
                    }
                }
                Key::Esc => {
                    // コマンドモードをキャンセル
                    mode_manager.enter_normal();
                    command_buffer.clear();
                }
                Key::Char(c) => command_buffer.push(c),
                Key::Backspace => {
                    command_buffer.pop();
                }
                _ => {}
            }
        } else if mode_manager.is_insert() {
            match key? {
                Key::Esc => {
                    mode_manager.enter_normal();
                    cursor.move_left();
                }
                Key::Char('\n') => {
                    // 改行
                    let row = cursor.file_row();
                    let col = (cursor.x() - 1) as usize;
                    editor.insert_newline(row, col);
                    cursor.move_down(editor_rows, editor.buffer().len());
                    cursor.move_to_line_start();
                    // TODO:
                    // 設定に応じて、改行したときに前の行とインデントを合わせることができるようにする
                }
                Key::Backspace => {
                    // 削除
                    let row = cursor.file_row();
                    let col = (cursor.x() - 1) as usize;

                    if col > 0 {
                        // 文字を削除
                        editor.delete_char(row, col - 1);
                        cursor.move_left();
                    } else if row > 0 {
                        // 行頭で Backspace + 前の行と結合
                        let prev_row = row - 1;
                        let prev_line_len =
                            editor.buffer().row(prev_row).map(|r| r.len()).unwrap_or(0);
                        editor.join_rows(row);
                        cursor.move_up();
                        cursor.move_to_line_end((prev_line_len as u16) + 1);
                    }
                }
                Key::Char(ch) => {
                    // 文字挿入
                    let row = cursor.file_row();
                    let col = (cursor.x() - 1) as usize;
                    editor.insert_char(row, col, ch);
                    // Insert モードでは行末の次の位置まで移動可能
                    cursor.move_right(
                        size.0,
                        editor.buffer().row(row).map(|r| r.len()).unwrap_or(0) + 1,
                    );
                }
                _ => {}
            }
        }

        // pending_key を更新する
        pending_key = next_pending_key;

        if mode_manager.current() != prev_mode {
            status_message.clear();
        }

        cursor.scroll(editor_rows, editor.buffer().len());

        // キー入力後に再描画
        Screen::refresh(
            terminal.stdout(),
            &cursor,
            mode_manager.current(),
            &command_buffer,
            editor.buffer(),
            editor.filename(),
            &status_message,
        )?;
    }

    Ok(())
}
