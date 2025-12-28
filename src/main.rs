use std::io::{self};

use termion::{event::Key, input::TermRead};
use zim::{
    buffer::Buffer, cursor::Cursor, file_io::FileIO, mode::ModeManager, screen::Screen,
    terminal::Terminal,
};

fn main() -> io::Result<()> {
    // ターミナル初期化
    let mut terminal = Terminal::new()?;
    terminal.clear_screen()?;

    // コマンドライン引数からファイル名を取得する
    let args: Vec<String> = std::env::args().collect();
    let (buffer, filename) = if args.len() > 1 {
        let path = &args[1];
        match FileIO::open(path) {
            Ok(buf) => (buf, Some(path.clone())),
            Err(e) => {
                eprintln!("Error opening file: {}", e);
                return Err(e);
            }
        }
    } else {
        (Buffer::new(), None)
    };

    // 状態初期化
    let mut cursor = Cursor::new();
    let mut mode_manager = ModeManager::new();
    let mut command_buffer = String::new();
    let mut pending_key: Option<char> = None;

    // 初期描画
    Screen::refresh(
        terminal.stdout(),
        &cursor,
        mode_manager.current(),
        &command_buffer,
        &buffer,
        filename.as_deref(),
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
                // vim キーバインド
                Key::Char('h') => cursor.move_left(),
                Key::Char('j') => {
                    cursor.move_down(editor_rows, buffer.len());
                    // 移動後の行に合わせて x 座標を調整する
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                Key::Char('k') => {
                    cursor.move_up();
                    // 移動後の行に合わせて x 座標を調整する
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                Key::Char('l') => {
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.move_right(size.0, line.len());
                    }
                }
                Key::Char('0') => cursor.move_to_line_start(),
                Key::Char('$') => {
                    // 現在の行の長さを取得して行末に移動
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.move_to_line_end(line.len() as u16);
                    }
                }
                Key::Char('g') => {
                    if pending_key == Some('g') {
                        // gg: ファイル先頭に移動する
                        cursor.move_to_top();
                        // 移動後の行に合わせて x 座標を調整する
                        let row = cursor.file_row();
                        if let Some(line) = buffer.row(row) {
                            cursor.adjust_cursor_x(line.len());
                        }
                    } else {
                        next_pending_key = Some('g');
                    }
                }
                Key::Char('G') => {
                    cursor.move_to_bottom(buffer.len(), editor_rows);
                    // 移動後の行に合わせて x 座標を調整する
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.adjust_cursor_x(line.len());
                    }
                }
                _ => {}
            }
        } else if mode_manager.is_command() {
            match key? {
                Key::Char('\n') => {
                    // コマンド実行
                    if command_buffer == "q" {
                        break;
                    }
                    // その他のコマンドは今は無視
                    mode_manager.enter_normal();
                    command_buffer.clear();
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
        }

        // pending_key を更新する
        pending_key = next_pending_key;

        cursor.scroll(editor_rows, buffer.len());

        // キー入力後に再描画
        Screen::refresh(
            terminal.stdout(),
            &cursor,
            mode_manager.current(),
            &command_buffer,
            &buffer,
            filename.as_deref(),
        )?;
    }

    Ok(())
}
