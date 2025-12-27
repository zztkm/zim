#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Command,
}

pub struct ModeManager {
    current: Mode,
}

impl ModeManager {
    pub fn new() -> Self {
        Self {
            current: Mode::Normal,
        }
    }

    pub fn current(&self) -> Mode {
        self.current
    }

    pub fn enter_command(&mut self) {
        self.current = Mode::Command;
    }

    pub fn enter_normal(&mut self) {
        self.current = Mode::Normal;
    }

    pub fn is_normal(&self) -> bool {
        self.current == Mode::Normal
    }

    pub fn is_command(&self) -> bool {
        self.current == Mode::Command
    }
}
