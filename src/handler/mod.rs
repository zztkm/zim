pub mod command;
pub mod insert;
pub mod normal;
pub mod visual;

pub enum HandlerResult {
    Continue,
    Quit,
    StatusMessage(String),
    ClearStatus,
}
