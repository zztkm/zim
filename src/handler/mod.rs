pub mod command;
pub mod insert;
pub mod normal;
pub mod visual;
pub mod visual_line;

pub enum HandlerResult {
    Continue,
    Quit,
    StatusMessage(String),
    ClearStatus,
}
