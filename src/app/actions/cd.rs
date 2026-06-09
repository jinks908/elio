use super::super::{
    App,
    state::CdOverlay,
    text_edit::{
        char_to_byte, next_delete_end, next_word_start, previous_delete_start, previous_word_start,
        remove_char_range,
    },
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::PathBuf;

impl App {
    pub(in crate::app) fn open_cd_overlay(&mut self) {
        self.overlays.help = false;
        self.overlays.search = None;
        self.overlays.create = None;
        self.overlays.trash = None;
        self.overlays.restore = None;
        self.overlays.rename = None;
        self.overlays.cd = Some(CdOverlay {
            input: String::new(),
            cursor_col: 0,
            error: None,
        });
    }

    pub fn cd_is_open(&self) -> bool {
        self.overlays.cd.is_some()
    }

    pub fn cd_input(&self) -> &str {
        self.overlays.cd.as_ref().map_or("", |c| &c.input)
    }

    pub fn cd_cursor_col(&self) -> usize {
        self.overlays.cd.as_ref().map_or(0, |c| c.cursor_col)
    }

    pub fn cd_error(&self) -> Option<&str> {
        self.overlays.cd.as_ref().and_then(|c| c.error.as_deref())
    }

    pub(in crate::app) fn handle_cd_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.overlays.cd = None;
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.overlays.cd = None;
            }
            KeyCode::Enter if key.modifiers == KeyModifiers::NONE => {
                self.confirm_cd()?;
            }
            KeyCode::Left
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd {
                    c.cursor_col = previous_word_start(&c.input, c.cursor_col);
                }
            }
            KeyCode::Right
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd {
                    c.cursor_col = next_word_start(&c.input, c.cursor_col);
                }
            }
            KeyCode::Left if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd {
                    c.cursor_col = c.cursor_col.saturating_sub(1);
                }
            }
            KeyCode::Right if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd {
                    let len = c.input.chars().count();
                    if c.cursor_col < len {
                        c.cursor_col += 1;
                    }
                }
            }
            KeyCode::Home if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd {
                    c.cursor_col = 0;
                }
            }
            KeyCode::End if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd {
                    c.cursor_col = c.input.chars().count();
                }
            }
            KeyCode::Backspace
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd
                    && c.cursor_col > 0
                {
                    let start = previous_delete_start(&c.input, c.cursor_col);
                    remove_char_range(&mut c.input, start, c.cursor_col);
                    c.cursor_col = start;
                    c.error = None;
                }
            }
            KeyCode::Char('h' | 'w')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd
                    && c.cursor_col > 0
                {
                    let start = previous_delete_start(&c.input, c.cursor_col);
                    remove_char_range(&mut c.input, start, c.cursor_col);
                    c.cursor_col = start;
                    c.error = None;
                }
            }
            KeyCode::Delete
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd {
                    let end = next_delete_end(&c.input, c.cursor_col);
                    remove_char_range(&mut c.input, c.cursor_col, end);
                    c.error = None;
                }
            }
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::ALT)
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if let Some(c) = &mut self.overlays.cd {
                    let end = next_delete_end(&c.input, c.cursor_col);
                    remove_char_range(&mut c.input, c.cursor_col, end);
                    c.error = None;
                }
            }
            KeyCode::Backspace if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd
                    && c.cursor_col > 0
                {
                    let start = char_to_byte(&c.input, c.cursor_col - 1);
                    let end = char_to_byte(&c.input, c.cursor_col);
                    c.input.replace_range(start..end, "");
                    c.cursor_col -= 1;
                    c.error = None;
                }
            }
            KeyCode::Delete if key.modifiers == KeyModifiers::NONE => {
                if let Some(c) = &mut self.overlays.cd {
                    let len = c.input.chars().count();
                    if c.cursor_col < len {
                        let start = char_to_byte(&c.input, c.cursor_col);
                        let end = char_to_byte(&c.input, c.cursor_col + 1);
                        c.input.replace_range(start..end, "");
                        c.error = None;
                    }
                }
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(c) = &mut self.overlays.cd {
                    let byte = char_to_byte(&c.input, c.cursor_col);
                    c.input.insert(byte, ch);
                    c.cursor_col += 1;
                    c.error = None;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(in crate::app) fn handle_cd_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let inside = self
                .input
                .frame_state
                .cd_panel
                .is_some_and(|panel| crate::fs::rect_contains(panel, mouse.column, mouse.row));
            if !inside {
                self.overlays.cd = None;
            }
        }
        Ok(())
    }

    fn confirm_cd(&mut self) -> Result<()> {
        let Some(c) = &self.overlays.cd else {
            return Ok(());
        };
        let raw = c.input.trim().to_string();

        if raw.is_empty() {
            self.overlays.cd = None;
            return Ok(());
        }

        let path = expand_tilde(&raw);

        // If the path points to a file, navigate to its parent instead.
        let target = if path.is_file() {
            path.parent().map(PathBuf::from).unwrap_or(path)
        } else {
            path
        };

        match self.set_dir(target) {
            Ok(()) => {
                self.overlays.cd = None;
            }
            Err(error) => {
                if let Some(c) = &mut self.overlays.cd {
                    c.error = Some(error.to_string());
                }
            }
        }
        Ok(())
    }
}

fn expand_tilde(input: &str) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if input == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    PathBuf::from(input)
}
