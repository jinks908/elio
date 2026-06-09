use crate::app::{App, FrameState};
use crate::ui::{
    helpers,
    theme::Palette,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

pub(super) fn render_cd_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    state: &mut FrameState,
    palette: Palette,
) {
    let popup_width = area.width.saturating_sub(8).clamp(40, 64);
    let popup_height = 6u16;
    let popup = helpers::centered_rect(area, popup_width, popup_height);
    state.cd_panel = Some(popup);

    frame.render_widget(Clear, popup);
    frame.render_widget(
        helpers::panel_block(" Go to Directory ", palette.chrome_alt, palette),
        popup,
    );

    let inner = helpers::inner_with_padding(popup);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        helpers::rounded_block(palette.path_bg, palette.border),
        rows[0],
    );
    let input_area = rows[0].inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let input = app.cd_input();
    let cursor_col = app.cd_cursor_col();
    let chars: Vec<char> = input.chars().collect();
    let col = cursor_col.min(chars.len());
    let available = input_area.width.saturating_sub(1) as usize;
    let h_start = col.saturating_sub(available);

    let mut visible_text: String = chars.iter().skip(h_start).take(available).collect();
    if h_start > 0 && !visible_text.is_empty() {
        visible_text.remove(0);
        visible_text.insert(0, '…');
    }

    let line = if input.is_empty() {
        Line::from(vec![Span::styled(
            "~/path/to/directory…",
            Style::default().fg(palette.muted),
        )])
    } else {
        Line::from(vec![Span::styled(
            visible_text,
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        )])
    };
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(palette.path_bg).fg(palette.text)),
        input_area,
    );

    let visible_cursor_col = col.saturating_sub(h_start);
    let cursor_x = (input_area.x + visible_cursor_col as u16)
        .min(input_area.x + input_area.width.saturating_sub(1));
    frame.set_cursor_position((cursor_x, input_area.y));

    if let Some(error) = app.cd_error() {
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                helpers::clamp_label(error, rows[1].width.saturating_sub(2) as usize),
                Style::default().fg(palette.accent),
            )]))
            .style(Style::default().bg(palette.chrome_alt).fg(palette.text)),
            rows[1],
        );
    }
}
