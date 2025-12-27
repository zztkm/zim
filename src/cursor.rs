pub struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Self { x: 1, y: 1 }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn move_up(&mut self) {
        if self.y > 1 {
            self.y -= 1;
        }
    }

    pub fn move_down(&mut self, max_rows: u16) {
        if self.y < max_rows {
            self.y += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    pub fn move_right(&mut self, max_cols: u16) {
        if self.x < max_cols {
            self.x += 1;
        }
    }
}
