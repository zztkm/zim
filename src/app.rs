use std::io::{self, Write};

use termion::event::Key;

use crate::cursor::Cursor;
use crate::editor::Editor;
use crate::handler::{self, HandlerResult};
use crate::mode::{Mode, ModeManager};
use crate::screen::Screen;

pub struct App {
    pub editor: Editor,
    pub cursor: Cursor,
    pub mode_manager: ModeManager,
    pub command_buffer: String,
    pub pending_key: Option<char>,
    pub status_message: String,
    pub terminal_size: (u16, u16),
    pub editor_rows: u16,
    prev_mode: Mode,
}

impl App {
    pub fn new(editor: Editor, terminal_size: (u16, u16)) -> Self {
        let editor_rows = Screen::editor_rows(terminal_size.1);
        Self {
            editor,
            cursor: Cursor::new(),
            mode_manager: ModeManager::new(),
            command_buffer: String::new(),
            pending_key: None,
            status_message: String::new(),
            terminal_size,
            editor_rows,
            prev_mode: Mode::Normal,
        }
    }

    pub fn handle_key(&mut self, key: Key) -> HandlerResult {
        let prev_mode = self.mode_manager.current();

        let result = if self.mode_manager.is_normal() {
            let r = handler::normal::handle(
                key,
                &mut self.editor,
                &mut self.cursor,
                &mut self.mode_manager,
                &mut self.pending_key,
                self.terminal_size,
                self.editor_rows,
            );
            // ':' でコマンドモードに入った場合、command_buffer をクリアする
            if self.mode_manager.is_command() {
                self.command_buffer.clear();
            }
            r
        } else if self.mode_manager.is_command() {
            handler::command::handle(
                key,
                &mut self.editor,
                &mut self.cursor,
                &mut self.mode_manager,
                &mut self.command_buffer,
                self.editor_rows,
            )
        } else if self.mode_manager.is_insert() {
            let r = handler::insert::handle(
                key,
                &mut self.editor,
                &mut self.cursor,
                &mut self.mode_manager,
                self.terminal_size,
                self.editor_rows,
            );
            // Insert モードからコマンドモードに入った場合、command_buffer をクリアする
            if self.mode_manager.is_command() {
                self.command_buffer.clear();
            }
            r
        } else if self.mode_manager.is_visual() {
            handler::visual::handle(
                key,
                &mut self.editor,
                &mut self.cursor,
                &mut self.mode_manager,
                self.terminal_size,
                self.editor_rows,
            )
        } else {
            HandlerResult::Continue
        };

        // HandlerResult に基づいてステータスメッセージを更新
        match &result {
            HandlerResult::StatusMessage(msg) => self.status_message = msg.clone(),
            HandlerResult::ClearStatus => self.status_message.clear(),
            _ => {}
        }

        // モードが変わった場合はステータスメッセージをクリア
        if self.mode_manager.current() != prev_mode {
            // ただし、ハンドラが明示的にメッセージを設定した場合は維持する
            if !matches!(&result, HandlerResult::StatusMessage(_)) {
                self.status_message.clear();
            }
        }
        self.prev_mode = self.mode_manager.current();

        self.cursor
            .scroll(self.editor_rows, self.editor.buffer().len());

        result
    }

    pub fn refresh(&self, stdout: &mut impl Write) -> io::Result<()> {
        Screen::refresh(
            stdout,
            &self.cursor,
            self.mode_manager.current(),
            &self.command_buffer,
            self.editor.buffer(),
            self.editor.filename(),
            &self.status_message,
            self.mode_manager.visual_start(),
        )
    }
}
