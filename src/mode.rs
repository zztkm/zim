use crate::cursor::Position;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Command,
    Insert,
    Visual,
}

pub struct ModeManager {
    current: Mode,
    visual_start: Option<Position>,
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeManager {
    pub fn new() -> Self {
        Self {
            current: Mode::Normal,
            visual_start: None,
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

    pub fn enter_insert(&mut self) {
        self.current = Mode::Insert;
    }

    pub fn enter_visual(&mut self, pos: Position) {
        self.visual_start = Some(pos);
        self.current = Mode::Visual;
    }

    pub fn is_normal(&self) -> bool {
        self.current == Mode::Normal
    }

    pub fn is_command(&self) -> bool {
        self.current == Mode::Command
    }

    pub fn is_insert(&self) -> bool {
        self.current == Mode::Insert
    }

    pub fn is_visual(&self) -> bool {
        self.current() == Mode::Visual
    }

    pub fn clear_visual(&mut self) {
        self.visual_start = None;
    }

    pub fn visual_start(&self) -> Option<Position> {
        self.visual_start
    }
}

