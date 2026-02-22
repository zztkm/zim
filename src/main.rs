use std::io;

use termion::input::TermRead;
use zim::{
    app::App, buffer::Buffer, editor::Editor, file_io::FileIO, handler::HandlerResult, logger,
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
    let editor = if args.len() > 1 {
        let path = &args[1];
        match FileIO::open(path) {
            Ok(buf) => Editor::from_buffer(buf, Some(path.clone())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // ファイルが存在しない場合は空バッファで開く
                // こうすることで保存時にファイルが作成される
                Editor::from_buffer(Buffer::new(), Some(path.clone()))
            }
            Err(e) => {
                eprintln!("Error opening file: {}", e);
                return Err(e);
            }
        }
    } else {
        Editor::new()
    };

    let mut app = App::new(editor, terminal.size());

    // 初期描画
    app.refresh(terminal.stdout())?;

    // main loop
    let stdin = io::stdin();
    for key in stdin.keys() {
        match app.handle_key(key?) {
            HandlerResult::Quit => break,
            _ => {}
        }
        app.refresh(terminal.stdout())?;
    }

    Ok(())
}
