use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::{App, RunStatus};
use crate::i18n::{self, Key};
use crate::theme::Status;
use crate::tools;
use crate::ui::widgets::panel;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let title = match app.selected_tool.and_then(tools::catalog::by_id) {
        Some(tool) => tool.name.to_string(),
        None => i18n::t(lang, Key::ScanTitle).to_string(),
    };

    let (status_text, status_color) = status_line(app);
    let block = panel::block(theme, &title, theme.tab_accent(2), false)
        .title_bottom(panel::readout(theme, &status_text, status_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Sin corrida: mensaje guía según haya o no herramienta elegida.
    let Some(run) = app.current_run() else {
        let msg = match app.selected_tool.and_then(tools::catalog::by_id) {
            Some(tool) if !app.tool_installed(tool.id) => {
                i18n::t(lang, Key::ScanNotInstalledTemplate).replacen("{}", tool.name, 1)
            }
            Some(tool) => i18n::t(lang, Key::ScanIdleTemplate).replacen("{}", tool.name, 1),
            None => i18n::t(lang, Key::ScanNoToolSelected).to_string(),
        };
        frame.render_widget(
            Paragraph::new(msg).style(theme.style_for(Status::Info)).wrap(Wrap { trim: true }),
            inner,
        );
        return;
    };

    let running = matches!(run.status, RunStatus::Running);
    let spin = crate::theme::glyph::spinner(app.tick);

    let mut lines: Vec<Line> = Vec::with_capacity(run.output.len() + 2);
    let prompt = if running {
        Span::styled(format!("{spin} $ {}", run.command), theme.style_for_color(theme.accent_secondary))
    } else {
        Span::styled(format!("$ {}", run.command), theme.readout_style())
    };
    lines.push(Line::from(prompt));
    lines.push(Line::from(""));
    if run.output.is_empty() {
        let empty = if running {
            Span::styled(
                format!("{spin} {}", i18n::t(lang, Key::ScanWaitingOutput)),
                theme.style_for_color(theme.accent_secondary),
            )
        } else {
            Span::styled(i18n::t(lang, Key::ScanOutputEmpty), theme.label_style())
        };
        lines.push(Line::from(empty));
    } else {
        for out in &run.output {
            lines.push(Line::from(Span::styled(out.clone(), theme.value_style())));
        }
    }

    let line_count = lines.len() as u16;
    let max_scroll = line_count.saturating_sub(inner.height);
    let scroll = app.scan_scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(lines).style(theme.value_style()).wrap(Wrap { trim: false }).scroll((scroll, 0)),
        inner,
    );
}

fn status_line(app: &App) -> (String, ratatui::style::Color) {
    let theme = &app.theme;
    let lang = app.lang;
    match app.current_run() {
        None => (i18n::t(lang, Key::ScanPickFromCatalog).to_string(), theme.fg_muted),
        Some(run) => match run.status {
            RunStatus::Running => (
                format!(
                    "{} {}",
                    crate::theme::glyph::spinner(app.tick),
                    i18n::t(lang, Key::ScanRunningTemplate).replacen("{}", &run.tool_name, 1),
                ),
                theme.accent_secondary,
            ),
            RunStatus::Finished { code, ok } => {
                let key = if ok { Key::ScanFinishedOkTemplate } else { Key::ScanFinishedErrTemplate };
                let text = i18n::t(lang, key)
                    .replacen("{}", &run.tool_name, 1)
                    .replacen("{}", &code.to_string(), 1);
                (text, if ok { theme.ok } else { theme.error })
            }
        },
    }
}
