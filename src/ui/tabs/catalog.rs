use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

use crate::app::App;
use crate::i18n::{self, Key};
use crate::theme::{Status, glyph};
use crate::ui::widgets::panel;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_filter_line(frame, rows[0], app);
    render_table(frame, rows[1], app);
}

fn render_filter_line(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    if !app.filter_active && app.filter_input.is_empty() {
        return;
    }
    let line = Line::from(vec![
        Span::styled(i18n::t(app.lang, Key::CatalogFilterLabel), theme.plate_style(theme.accent_secondary)),
        Span::styled(format!(" {}", app.filter_input), theme.value_style()),
        if app.filter_active {
            Span::styled("▏", theme.style_for_color(theme.accent_secondary))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_table(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;
    let tools = app.filtered_tools();
    let installed = tools.iter().filter(|t| app.tool_installed(t.id)).count();

    let detail = i18n::t(lang, Key::CatalogDetailTemplate)
        .replacen("{}", &tools.len().to_string(), 1)
        .replacen("{}", &installed.to_string(), 1);
    let block = panel::block(theme, super::Tab::Catalog.title(lang), theme.tab_accent(1), false)
        .title_bottom(panel::readout(theme, &detail, theme.fg_muted));

    let header = Row::new([
        i18n::t(lang, Key::ColInstalled),
        i18n::t(lang, Key::ColName),
        i18n::t(lang, Key::ColCategory),
        i18n::t(lang, Key::ColPackage),
        i18n::t(lang, Key::ColLicense),
        i18n::t(lang, Key::ColStatus),
    ])
    .style(theme.table_header_style());

    let selected = app.catalog_row;
    let body = tools.iter().enumerate().map(|(i, tool)| {
        let is_installed = app.tool_installed(tool.id);
        let (led, status, status_text) = if is_installed {
            (glyph::LED_ON, Status::Ok, i18n::t(lang, Key::StatusInstalled))
        } else {
            (glyph::LED_OFF, Status::Warning, i18n::t(lang, Key::StatusMissing))
        };

        let row = Row::new([
            Span::styled(led, theme.style_for(status)),
            Span::styled(tool.name.to_string(), theme.style_for_bold()),
            Span::styled(tool.category.label(lang).to_string(), theme.label_style()),
            Span::styled(tool.pkg_debian.to_string(), theme.readout_style()),
            Span::styled(tool.license.to_string(), theme.label_style()),
            Span::styled(status_text.to_string(), theme.style_for(status)),
        ]);

        if i == selected {
            row.style(theme.selected_row_style())
        } else {
            row
        }
    });

    let widths = [
        Constraint::Length(1),
        Constraint::Percentage(22),
        Constraint::Percentage(20),
        Constraint::Percentage(26),
        Constraint::Percentage(18),
        Constraint::Length(10),
    ];

    let table = Table::new(body, widths).header(header).column_spacing(1).block(block);
    frame.render_widget(table, area);
}
