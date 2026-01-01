use std::io::{self};

use termion::{event::Key, input::TermRead};
use zim::{
    cursor::{Cursor, Position},
    editor::{Editor, PasteDirection, PasteResult},
    file_io::FileIO,
    logger,
    mode::ModeManager,
    screen::Screen,
    terminal::Terminal,
};

fn main() -> io::Result<()> {
    // ロガー初期化 (debug build のみ)
    let _ = logger::init("/tmp/zim_debug.log");

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
        mode_manager.visual_start(),
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
                    let pos = cursor.position();
                    if editor.delete_char_at_cursor(pos) {
                        // 削除成功後、行末を超えないように調整
                        let line_len = editor.current_line_len(pos.row);
                        if line_len > 0 && cursor.x() > line_len as u16 {
                            cursor.move_left();
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
                            let (buffer_len, line_len) = editor.buffer_info(cursor.file_row());
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
                    let pos = cursor.position();

                    match editor.paste(pos, PasteDirection::Below) {
                        PasteResult::InLine => {
                            let line_len = editor.current_line_len(pos.row);
                            cursor.move_right(size.0, line_len);
                        }
                        PasteResult::Below => {
                            cursor.move_down(editor_rows, editor.buffer().len());
                        }
                        _ => {}
                    }
                    status_message.clear();
                }
                Key::Char('P') => {
                    let pos = cursor.position();

                    // Above の場合は特にカーソル移動する必要がない
                    if let PasteResult::InLine = editor.paste(pos, PasteDirection::Above) {
                        let line_len = editor.current_line_len(pos.row);
                        cursor.move_right(size.0, line_len);
                    }
                    status_message.clear();
                }
                // Visual mode 系
                Key::Char('v') => {
                    mode_manager.enter_visual(cursor.position());
                }
                // 移動系
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
                    let cmd = parts.first().copied().unwrap_or("");

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
                                            let (buffer_len, line_len) =
                                                editor.buffer_info(cursor.file_row());
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
                            editor.buffer().row(prev_row).map(|r| r.len()).unwrap_or(0);
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
                        size.0,
                        editor.buffer().row(pos.row).map(|r| r.len()).unwrap_or(0) + 1,
                    );
                }
                _ => {}
            }
        } else if mode_manager.is_visual() {
            match key? {
                Key::Esc => {
                    mode_manager.enter_normal();
                    mode_manager.clear_visual();
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
                Key::Char('y') => {
                    // ヤンク
                    if let Some(start) = mode_manager.visual_start() {
                        let end = cursor.position();
                        editor.yank_range(start, end);
                        mode_manager.enter_normal();
                        mode_manager.clear_visual();
                        status_message = "Yanked selection".to_string();
                    }
                }
                Key::Char('d') => {
                    // 削除してヤンク
                    if let Some(start) = mode_manager.visual_start() {
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
                                cursor.move_right(size.0, line_len);
                            }

                            cursor.scroll(editor_rows, editor.buffer().len());
                        }
                        mode_manager.enter_normal();
                        mode_manager.clear_visual();
                        status_message = "Deleted selection".to_string();
                    }
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
            mode_manager.visual_start(),
        )?;
    }

    Ok(())
}
