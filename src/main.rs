use std::{
    fs::write,
    io::{self, Write, stdout},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

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
    for i in 0..rows {
        write!(stdout, "~")?;

        // 最後の行以外は改行する
        if i < rows - 1 {
            write!(stdout, "\r\n")?;
        }
    }
    Ok(())
}

fn refresh_screen(stdout: &mut impl Write, cursor: &Cursor) -> io::Result<()> {
    // カーソルを隠す
    write!(stdout, "{}", termion::cursor::Hide)?;
    // カーソルを左上に移動
    write!(stdout, "{}", termion::cursor::Goto(1, 1))?;

    // 画面描画
    let size = termion::terminal_size()?;
    draw_rows(stdout, size.1)?;

    // カーソル移動
    write!(stdout, "{}", termion::cursor::Goto(cursor.x, cursor.y))?;
    // カーソル表示
    write!(stdout, "{}", termion::cursor::Show)?;

    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // raw mode 切り替え
    let mut stdout = stdout().into_raw_mode()?;
    let mut cursor = Cursor::new();

    // 画面クリア
    write!(stdout, "{}", termion::clear::All)?;
    // 初期画面
    refresh_screen(&mut stdout, &cursor)?;

    let stdin = io::stdin();
    let size = termion::terminal_size()?;

    // キー入力ループ
    for key in stdin.keys() {
        match key? {
            // Ctrl + q で終了
            Key::Ctrl('q') => break,
            // vim キーバインド
            Key::Char('h') => cursor.move_left(),
            Key::Char('j') => cursor.move_down(size.1),
            Key::Char('k') => cursor.move_up(),
            Key::Char('l') => cursor.move_right(size.0),
            _ => {}
        }

        // キー入力後に再描画
        refresh_screen(&mut stdout, &cursor)?;
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
