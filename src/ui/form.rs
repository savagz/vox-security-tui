use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::app::RunForm;
use crate::i18n::{self, Key, Lang};
use crate::theme::Theme;
use crate::tools;

use super::widgets::panel;

/// Modal para lanzar una herramienta: selector de plantilla + objetivo +
/// argumentos. `◂ ▸` cambia de plantilla (rellena el campo de argumentos).
pub fn render(frame: &mut Frame, area: Rect, form: &RunForm, theme: &Theme, lang: Lang) {
    let modal = panel::centered(area, 74, 55);
    frame.render_widget(Clear, modal);

    let tool = tools::catalog::by_id(form.tool_id);
    let tool_name = tool.map(|t| t.name).unwrap_or(form.tool_id);
    let title = i18n::t(lang, Key::FormRunTitleTemplate).replacen("{}", tool_name, 1);
    let block = panel::block(theme, &title, theme.accent_primary, true);
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // etiqueta plantilla
            Constraint::Length(1), // selector plantilla
            Constraint::Length(1), // espacio
            Constraint::Length(1), // etiqueta objetivo
            Constraint::Length(1), // valor objetivo
            Constraint::Length(1), // etiqueta argumentos
            Constraint::Length(1), // valor argumentos
            Constraint::Min(1),    // error
            Constraint::Length(1), // teclas
        ])
        .split(inner);

    // -- selector de plantilla --
    let templates = tool.map(|t| t.templates).unwrap_or(&[]);
    let idx = form.template_idx.min(templates.len().saturating_sub(1));
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {} ", i18n::t(lang, Key::FormLabelTemplate)),
            theme.plate_style(theme.accent_secondary),
        ))),
        rows[0],
    );
    if let Some(t) = templates.get(idx) {
        let selector = Line::from(vec![
            Span::styled("  ◂ ", theme.style_for_color(theme.accent_primary)),
            Span::styled(t.name(lang).to_string(), theme.selected_row_style()),
            Span::styled(" ▸", theme.style_for_color(theme.accent_primary)),
            Span::styled(format!("   ({}/{})", idx + 1, templates.len()), theme.label_style()),
        ]);
        frame.render_widget(Paragraph::new(selector), rows[1]);
    }

    // -- campos de texto: objetivo (focus 0) y argumentos (focus 1) --
    let args_label = i18n::t(lang, Key::FormLabelArgs).replacen("{}", "{target}", 1);
    let fields = [
        (i18n::t(lang, Key::FormLabelTarget).to_string(), &form.target, rows[3], rows[4]),
        (args_label, &form.args, rows[5], rows[6]),
    ];

    for (i, (label, value, label_row, value_row)) in fields.iter().enumerate() {
        let focused = form.focus == i;
        let marker = if focused { "▌" } else { " " };
        let value_style = if focused { theme.selected_row_style() } else { theme.value_style() };
        let line = Line::from(vec![
            Span::styled(marker, theme.style_for_color(theme.accent_primary)),
            Span::styled(format!("{label:<28}"), theme.label_style()),
        ]);
        frame.render_widget(Paragraph::new(line), *label_row);

        let value_line = Line::from(vec![
            Span::styled("  ", theme.label_style()),
            Span::styled((*value).clone(), value_style),
            if focused {
                Span::styled("▏", theme.style_for_color(theme.accent_primary))
            } else {
                Span::raw("")
            },
        ]);
        frame.render_widget(Paragraph::new(value_line), *value_row);
    }

    if let Some(error) = &form.error {
        frame.render_widget(
            Paragraph::new(Span::styled(error.clone(), theme.style_for(crate::theme::Status::Error))),
            rows[7],
        );
    }

    let keys = Line::from(vec![
        Span::styled(" ◂ ▸ ", theme.plate_style(theme.accent_primary)),
        Span::styled(i18n::t(lang, Key::FormTemplateHint), theme.label_style()),
        Span::styled(" TAB ", theme.plate_style(theme.accent_secondary)),
        Span::styled(i18n::t(lang, Key::FormNextField), theme.label_style()),
        Span::styled(" ENTER ", theme.plate_style(theme.ok)),
        Span::styled(i18n::t(lang, Key::Confirm), theme.label_style()),
        Span::styled(" ESC ", theme.plate_style(theme.accent_tertiary)),
        Span::styled(i18n::t(lang, Key::Cancel), theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(keys), rows[8]);
}
