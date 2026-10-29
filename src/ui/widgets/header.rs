use chrono::Local;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::i18n::{self, Key};
use crate::theme::{Status, glyph};
use crate::tools;

use super::panel;

/// Franja superior: identificación del producto a la izquierda, pilotos de
/// estado (modo, herramientas, reloj) a la derecha.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let block = panel::block(theme, i18n::t(app.lang, Key::HeaderTitle), theme.accent_primary, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    let mode = if app.env.fixtures { "FIXTURES" } else { "REAL" };
    let target = Line::from(vec![
        Span::styled("MODO ", theme.label_style()),
        Span::styled(mode, theme.readout_style()),
    ]);
    frame.render_widget(Paragraph::new(target), cols[0]);

    let mode_status = if app.env.fixtures { Status::Info } else { Status::Ok };
    let installed = app.installed_count();
    let total = tools::catalog::ALL.len();

    let pilots = Line::from(vec![
        led(theme, mode_status),
        Span::styled(
            format!("{}  ", i18n::t(app.lang, Key::OverviewToolsTitle).to_uppercase()),
            theme.style_for(mode_status),
        ),
        Span::styled(format!("{installed}/{total}  "), theme.style_for(Status::Ok)),
        Span::styled(Local::now().format("%H:%M:%S").to_string(), theme.readout_style()),
    ]);
    frame.render_widget(Paragraph::new(pilots).alignment(Alignment::Right), cols[1]);
}

fn led<'a>(theme: &crate::theme::Theme, status: Status) -> Span<'a> {
    Span::styled(format!("{} ", glyph::LED_ON), theme.style_for(status))
}
