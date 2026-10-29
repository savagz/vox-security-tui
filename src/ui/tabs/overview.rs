use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::App;
use crate::i18n::{self, Key};
use crate::theme::Status;
use crate::tools::{self, Category};
use crate::ui::format;
use crate::ui::widgets::{gauge, panel};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_gauges(frame, rows[0], app);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_tools(frame, cols[0], app);
    render_side(frame, cols[1], app);
}

fn render_gauges(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    gauge::render(frame, cols[0], theme, i18n::t(lang, Key::OverviewGaugeCpu), "", app.host.cpu_pct);
    gauge::render(
        frame,
        cols[1],
        theme,
        i18n::t(lang, Key::OverviewGaugeRam),
        &format!("{:.1}/{:.1} GiB", app.host.mem_used_gb, app.host.mem_total_gb),
        app.host.mem_pct,
    );
    gauge::render(
        frame,
        cols[2],
        theme,
        i18n::t(lang, Key::OverviewGaugeDisk),
        &format!("{:.0}/{:.0} GiB", app.host.disk_used_gb, app.host.disk_total_gb),
        app.host.disk_pct,
    );
}

fn render_tools(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let block = panel::block(theme, i18n::t(lang, Key::OverviewToolsTitle), theme.accent_primary, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let installed = app.installed_count();
    let total = tools::catalog::ALL.len();

    let mut lines = vec![
        Line::from(Span::styled(
            i18n::t(lang, Key::OverviewToolsCountTemplate)
                .replacen("{}", &installed.to_string(), 1)
                .replacen("{}", &total.to_string(), 1),
            theme.readout_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            i18n::t(lang, Key::OverviewCategoriesTitle),
            theme.plate_style(theme.accent_secondary),
        )),
    ];

    for cat in Category::ALL {
        let total_cat = tools::catalog::ALL.iter().filter(|t| t.category == cat).count();
        let inst_cat = tools::catalog::ALL
            .iter()
            .filter(|t| t.category == cat && app.tool_installed(t.id))
            .count();
        let status = if inst_cat > 0 { Status::Ok } else { Status::Info };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<18}", cat.label(lang)), theme.label_style()),
            Span::styled(format!("{inst_cat}/{total_cat}"), theme.style_for(status)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_side(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    // Ficha del host.
    let host_block = panel::block(theme, i18n::t(lang, Key::TabOverview), theme.accent_secondary, false);
    let host_inner = host_block.inner(rows[0]);
    frame.render_widget(host_block, rows[0]);
    let host_lines = vec![
        Line::from(vec![
            Span::styled(i18n::t(lang, Key::OverviewLabelStorage), theme.label_style()),
            Span::styled(
                format!("{} / {}", format::bytes((app.host.disk_used_gb * 1e9) as u64), format::bytes((app.host.disk_total_gb * 1e9) as u64)),
                theme.readout_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(i18n::t(lang, Key::OverviewLabelUptime), theme.label_style()),
            Span::styled(format::uptime(app.host.uptime_secs), theme.readout_style()),
        ]),
    ];
    frame.render_widget(Paragraph::new(host_lines), host_inner);

    // Aviso legal.
    let legal_block = panel::block(theme, i18n::t(lang, Key::OverviewLegalTitle), theme.error, true);
    let legal_inner = legal_block.inner(rows[1]);
    frame.render_widget(legal_block, rows[1]);
    frame.render_widget(
        Paragraph::new(i18n::t(lang, Key::OverviewLegalBody))
            .style(theme.style_for(Status::Warning))
            .wrap(Wrap { trim: true }),
        legal_inner,
    );
}
