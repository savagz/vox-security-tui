use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::{App, RunStatus};
use crate::i18n::{self, Key};
use crate::theme::Status;
use crate::ui::widgets::panel;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    if app.runs.is_empty() {
        let block = panel::block(theme, super::Tab::Logs.title(lang), theme.tab_accent(3), false);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(i18n::t(lang, Key::LogsEmpty)).style(theme.label_style()).wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_list(frame, cols[0], app);
    render_detail(frame, cols[1], app);
}

fn render_list(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let detail = i18n::t(lang, Key::LogsDetailTemplate).replacen("{}", &app.runs.len().to_string(), 1);
    let block = panel::block(theme, super::Tab::Logs.title(lang), theme.tab_accent(3), false)
        .title_bottom(panel::readout(theme, &detail, theme.fg_muted));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = app
        .runs
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let (tag, status) = match run.status {
                RunStatus::Running => (
                    format!("{} {}", crate::theme::glyph::spinner(app.tick), i18n::t(lang, Key::LogRunningTag)),
                    Status::Warning,
                ),
                RunStatus::Finished { ok: true, .. } => (i18n::t(lang, Key::LogOkTag).to_string(), Status::Ok),
                RunStatus::Finished { ok: false, .. } => (i18n::t(lang, Key::LogErrTag).to_string(), Status::Error),
            };
            let marker = if i == app.logs_row { "▌" } else { " " };
            let name_style = if i == app.logs_row { theme.style_for_bold() } else { theme.value_style() };
            Line::from(vec![
                Span::styled(marker, theme.style_for_color(theme.accent_primary)),
                Span::styled(format!("{} ", run.started), theme.label_style()),
                Span::styled(format!("{:<8}", tag), theme.style_for(status)),
                Span::styled(run.tool_name.clone(), name_style),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let block = panel::block(theme, i18n::t(lang, Key::ScanTitle), theme.accent_secondary, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(run) = app.selected_run() else {
        return;
    };

    let mut lines = vec![
        Line::from(Span::styled(format!("$ {}", run.command), theme.readout_style())),
        Line::from(""),
    ];
    if run.output.is_empty() {
        lines.push(Line::from(Span::styled(i18n::t(lang, Key::ScanOutputEmpty), theme.label_style())));
    } else {
        for out in &run.output {
            lines.push(Line::from(Span::styled(out.clone(), theme.value_style())));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).style(theme.value_style()).wrap(Wrap { trim: false }),
        inner,
    );
}
