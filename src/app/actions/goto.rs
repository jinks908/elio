use super::super::{
    App,
    state::{GoToDestination, GoToOverlay, GoToOverlayRow},
};
use crate::fs::rect_contains;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

impl App {
    pub fn goto_is_open(&self) -> bool {
        self.overlays.goto.is_some()
    }

    pub fn goto_title(&self) -> &str {
        self.overlays
            .goto
            .as_ref()
            .map(|overlay| overlay.title.as_str())
            .unwrap_or("")
    }

    pub fn goto_row_count(&self) -> usize {
        self.overlays
            .goto
            .as_ref()
            .map(|overlay| overlay.rows.len())
            .unwrap_or(0)
    }

    pub fn goto_row_label(&self, index: usize) -> &str {
        self.overlays
            .goto
            .as_ref()
            .and_then(|overlay| overlay.rows.get(index))
            .map(|row| row.label.as_str())
            .unwrap_or("")
    }

    pub fn goto_row_shortcut(&self, index: usize) -> Option<char> {
        self.overlays
            .goto
            .as_ref()
            .and_then(|overlay| overlay.rows.get(index))
            .map(|row| row.shortcut)
    }
}

impl App {
    pub(in crate::app) fn open_goto_overlay(&mut self) {
        self.overlays.help = false;
        self.overlays.goto = Some(build_goto_overlay(self));
        self.status.clear();
    }

    pub(in crate::app) fn handle_goto_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.overlays.goto = None;
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.overlays.goto = None;
            }
            KeyCode::Char('0')
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.overlays.goto = None;
                let start_dir = self.navigation.start_dir.clone();
                self.set_dir(start_dir)?;
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(index) = self.goto_row_index_for_shortcut(ch) {
                    self.confirm_goto_index(index)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub(in crate::app) fn handle_goto_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let inside = self
                .input
                .frame_state
                .goto_panel
                .is_some_and(|panel| rect_contains(panel, mouse.column, mouse.row));
            if !inside {
                self.overlays.goto = None;
                return Ok(());
            }

            if let Some(hit) = self
                .input
                .frame_state
                .goto_hits
                .iter()
                .find(|hit| rect_contains(hit.rect, mouse.column, mouse.row))
                .cloned()
            {
                self.confirm_goto_index(hit.index)?;
            }
        }

        Ok(())
    }

    fn goto_row_index_for_shortcut(&self, ch: char) -> Option<usize> {
        let needle = ch.to_ascii_lowercase();
        self.overlays.goto.as_ref().and_then(|overlay| {
            overlay
                .rows
                .iter()
                .position(|row| row.shortcut.to_ascii_lowercase() == needle)
        })
    }

    fn confirm_goto_index(&mut self, index: usize) -> Result<()> {
        let Some(destination) = self
            .overlays
            .goto
            .as_ref()
            .and_then(|overlay| overlay.rows.get(index).map(|row| row.destination.clone()))
        else {
            return Ok(());
        };

        match destination {
            GoToDestination::Path(path) => {
                self.overlays.goto = None;
                self.set_dir(path)?;
            }
            GoToDestination::Missing(status) => {
                self.status = status;
            }
        }

        Ok(())
    }
}

fn build_goto_overlay(app: &App) -> GoToOverlay {
    let config_entries = &crate::config::go_to().entries;

    let rows: Vec<GoToOverlayRow> = if config_entries.is_empty() {
        app.navigation
            .sidebar
            .iter()
            .filter_map(|row| row.item())
            .take(5)
            .enumerate()
            .map(|(i, item)| {
                let shortcut = char::from_digit((i + 1) as u32, 10).unwrap_or('?');
                let destination = if item.path.exists() {
                    GoToDestination::Path(item.path.clone())
                } else {
                    GoToDestination::Missing(format!("{} not available", item.title))
                };
                GoToOverlayRow {
                    shortcut,
                    label: item.title.clone(),
                    destination,
                }
            })
            .collect()
    } else {
        config_entries
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, entry)| {
                let shortcut = char::from_digit((i + 1) as u32, 10).unwrap_or('?');
                let destination = if entry.path.exists() {
                    GoToDestination::Path(entry.path.clone())
                } else {
                    GoToDestination::Missing(format!("{} not available", entry.title))
                };
                GoToOverlayRow {
                    shortcut,
                    label: entry.title.clone(),
                    destination,
                }
            })
            .collect()
    };

    GoToOverlay {
        title: "Go to".to_string(),
        rows,
    }
}
