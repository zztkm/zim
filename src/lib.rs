pub mod buffer;
pub mod cursor;
pub mod file_io;
pub mod mode;
pub mod screen;
pub mod terminal;

// 画面レイアウト定数
pub const STATUS_BAR_HEIGHT: u16 = 1;
pub const COMMAND_LINE_HEIGHT: u16 = 1;
pub const UI_HEIGHT: u16 = STATUS_BAR_HEIGHT + COMMAND_LINE_HEIGHT;
