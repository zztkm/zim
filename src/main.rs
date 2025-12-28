use std::{
    env::args,
    io::{self, stdout},
};

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

    // 初期描画
    Screen::refresh(
        terminal.stdout(),
        &cursor,
        &mode_manager.current(),
        &command_buffer,
        &buffer,
        filename.as_deref(),
    )?;

    // main loop
    let stdin = io::stdin();
    let size = terminal.size();

    for key in stdin.keys() {
        if mode_manager.is_normal() {
            match key? {
                Key::Char(':') => {
                    mode_manager.enter_command();
                    command_buffer.clear();
                }
                // vim キーバインド
                Key::Char('h') => cursor.move_left(),
                // ステータスライン分 - 1
                Key::Char('j') => cursor.move_down(size.1 - 1),
                Key::Char('k') => cursor.move_up(),
                Key::Char('l') => cursor.move_right(size.0),
                Key::Char('0') => cursor.move_to_line_start(),
                Key::Char('$') => {
                    // 現在の行の長さを取得して行末に移動
                    let row = cursor.file_row();
                    if let Some(line) = buffer.row(row) {
                        cursor.move_to_line_end(line.len() as u16);
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

        // キー入力後に再描画
        Screen::refresh(
            terminal.stdout(),
            &cursor,
            &mode_manager.current(),
            &command_buffer,
            &buffer,
            filename.as_deref(),
        )?;
    }

    Ok(())
}
