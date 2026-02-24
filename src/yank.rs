use arboard::Clipboard;

#[derive(Debug)]
pub enum YankType {
    /// 行内にペースト
    InLine,
    /// 新しい行としてペースト
    NewLine,
}

pub struct YankManager {
    buffer: Vec<String>,
    yank_type: YankType,
    /// システムクリップボード連携
    clipboard: Option<Clipboard>,
}

impl YankManager {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            yank_type: YankType::InLine,
            clipboard: Clipboard::new().ok(),
        }
    }

    pub fn yank_inline(&mut self, text: String) {
        self.buffer = vec![text];
        self.yank_type = YankType::InLine;
    }

    pub fn yank_line(&mut self, text: String) {
        self.buffer = vec![text];
        self.yank_type = YankType::NewLine;
    }

    pub fn yank_lines(&mut self, lines: Vec<String>) {
        self.buffer = lines;
        self.yank_type = YankType::NewLine;
    }

    pub fn is_newline_yank(&self) -> bool {
        matches!(self.yank_type, YankType::NewLine)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn content(&self) -> &[String] {
        &self.buffer
    }

    pub fn sync_to_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard
            && !self.buffer.is_empty()
        {
            let text = self.buffer.join("\n");
            // set_text に失敗しても無視する
            let _ = clipboard.set_text(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yank_manager_new() {
        let ym = YankManager::new();
        assert!(ym.is_empty());
        assert!(!ym.is_newline_yank());
    }

    #[test]
    fn test_yank_manager_yank_inline() {
        let mut ym = YankManager::new();
        ym.yank_inline("hello".to_string());

        assert!(!ym.is_empty());
        assert!(!ym.is_newline_yank());
        assert_eq!(ym.content(), &["hello"]);
    }

    #[test]
    fn test_yank_manager_yank_line() {
        let mut ym = YankManager::new();
        ym.yank_line("line content".to_string());

        assert!(!ym.is_empty());
        assert!(ym.is_newline_yank());
        assert_eq!(ym.content(), &["line content"]);
    }

    #[test]
    fn test_yank_manager_type_change() {
        let mut ym = YankManager::new();

        // InLine → NewLine
        ym.yank_inline("char".to_string());
        assert!(!ym.is_newline_yank());

        ym.yank_line("line".to_string());
        assert!(ym.is_newline_yank());

        // NewLine → InLine
        ym.yank_inline("char2".to_string());
        assert!(!ym.is_newline_yank());
    }
}
