pub struct Row {
    chars: String,
    render: String,
}

impl Row {
    pub fn new(text: String) -> Self {
        let render = text.clone();
        Self {
            chars: text,
            render,
        }
    }

    pub fn chars(&self) -> &str {
        &self.chars
    }

    pub fn render(&self) -> &str {
        &self.render
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

pub struct Buffer {
    rows: Vec<Row>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn insert_row(&mut self, at: usize, text: String) {
        if at <= self.rows.len() {
            self.rows.insert(at, Row::new(text));
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
}
