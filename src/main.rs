use std::io::{self, stdout};

use termion::{event::Key, input::TermRead};
use zim::{cursor::Cursor, mode::ModeManager, screen::Screen, terminal::Terminal};

fn main() -> io::Result<()> {
    // ターミナル初期化
    let mut terminal = Terminal::new()?;
    terminal.clear_screen()?;

    // 初期化
    let mut cursor = Cursor::new();
    let mut mode_manager = ModeManager::new();
    let mut command_buffer = String::new();

    // 初期描画
    Screen::refresh(
        terminal.stdout(),
        &cursor,
        &mode_manager.current(),
        &command_buffer,
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
        )?;
    }

    Ok(())
}
