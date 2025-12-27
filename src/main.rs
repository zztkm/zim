use std::io::{self, Write, stdout};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

enum Mode {
    Normal,
    Command,
}

struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    fn new() -> Self {
        Self { x: 1, y: 1 }
    }

    fn move_up(&mut self) {
        if self.y > 1 {
            self.y -= 1;
        }
    }

    fn move_down(&mut self, max_rows: u16) {
        if self.y < max_rows {
            self.y += 1;
        }
    }
    fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    fn move_right(&mut self, max_cols: u16) {
        if self.x < max_cols {
            self.x += 1;
        }
    }
}

fn draw_rows(stdout: &mut impl Write, rows: u16) -> io::Result<()> {
    // 最後の行はステータス / コマンドライン用
    for i in 0..rows - 1 {
        write!(stdout, "~")?;
        // 最後の行の手前の行以外は改行する
        if i < rows - 2 {
            write!(stdout, "\r\n")?;
        }
    }
    Ok(())
}

fn refresh_screen(
    stdout: &mut impl Write,
    cursor: &Cursor,
    mode: &Mode,
    command_buffer: &str,
) -> io::Result<()> {
    // カーソルを隠す
    write!(stdout, "{}", termion::cursor::Hide)?;
    // カーソルを左上に移動
    write!(stdout, "{}", termion::cursor::Goto(1, 1))?;

    // 画面描画
    let size = termion::terminal_size()?;
    draw_rows(stdout, size.1)?;

    // コマンドライン / ステータスライン (最下行)
    write!(stdout, "\r\n")?;
    match mode {
        Mode::Command => {
            write!(stdout, ":{}", command_buffer)?;
        }
        Mode::Normal => {
            write!(stdout, " ")?;
        }
    }

    // カーソル位置に移動
    match mode {
        Mode::Command => {
            // コマンドモード時はコマンドライン上にカーソル
            write!(
                stdout,
                ":{}",
                termion::cursor::Goto((command_buffer.len() + 2) as u16, size.1)
            )?;
        }
        Mode::Normal => {
            // ノーマルモード時はエディタ上にカーソル
            write!(stdout, "{}", termion::cursor::Goto(cursor.x, cursor.y))?;
        }
    }

    write!(stdout, "{}", termion::cursor::Show)?;
    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // raw mode 切り替え
    let mut stdout = stdout().into_raw_mode()?;
    let mut cursor = Cursor::new();
    let mut mode = Mode::Normal;
    let mut command_buffer = String::new();

    // 画面クリア
    write!(stdout, "{}", termion::clear::All)?;
    // 初期画面
    refresh_screen(&mut stdout, &cursor, &mode, &command_buffer)?;

    let stdin = io::stdin();
    let size = termion::terminal_size()?;

    // キー入力ループ
    for key in stdin.keys() {
        match mode {
            Mode::Normal => {
                match key? {
                    Key::Char(':') => {
                        mode = Mode::Command;
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
            }
            Mode::Command => {
                match key? {
                    Key::Char('\n') => {
                        // コマンド実行
                        if command_buffer == "q" {
                            break;
                        }
                        // その他のコマンドは今は無視
                        mode = Mode::Normal;
                        command_buffer.clear();
                    }
                    Key::Esc => {
                        // コマンドモードをキャンセル
                        mode = Mode::Normal;
                        command_buffer.clear();
                    }
                    Key::Char(c) => command_buffer.push(c),
                    Key::Backspace => {
                        command_buffer.pop();
                    }
                    _ => {}
                }
            }
        }

        // キー入力後に再描画
        refresh_screen(&mut stdout, &cursor, &mode, &command_buffer)?;
    }

    // 終了時に画面をクリアする
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )?;
    stdout.flush()?;

    Ok(())
}
