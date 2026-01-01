use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger {
    file: std::fs::File,
}

impl Logger {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file })
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let _ = writeln!(self.file, "[{}] {}", timestamp, message);
    }
}

// グローバルロガー用のスレッドローカル変数
use std::cell::RefCell;

thread_local! {
    static LOGGER: RefCell<Option<Logger>> = RefCell::new(None);
}

pub fn init(path: &str) -> std::io::Result<()> {
    // debug build でのみロガーを初期化
    #[cfg(debug_assertions)]
    {
        let logger = Logger::new(path)?;
        LOGGER.with(|l| {
            *l.borrow_mut() = Some(logger);
        });
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = path; // unused variable warning を回避
    }
    Ok(())
}

pub fn debug(message: &str) {
    // debug build でのみログを書き込む
    #[cfg(debug_assertions)]
    {
        LOGGER.with(|l| {
            if let Some(logger) = l.borrow_mut().as_mut() {
                logger.log(message);
            }
        });
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = message; // unused variable warning を回避
    }
}
