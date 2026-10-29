use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::app::App;
use crate::i18n::{self, Key, Lang};

use super::panel;

enum Entry {
    Section(&'static str),
    Row(&'static str, &'static str),
    Blank,
}

fn entries(lang: Lang) -> Vec<Entry> {
    use Entry::*;
    vec![
        Section(i18n::t(lang, Key::HelpSectionGlobal)),
        Row("TAB / 1-5", i18n::t(lang, Key::HbJump)),
        Row("↑↓ / j k", i18n::t(lang, Key::HbNavigate)),
        Row("Home/End / g G", i18n::t(lang, Key::HelpTopBottom)),
        Row("R", i18n::t(lang, Key::HbRefresh)),
        Row("C", i18n::t(lang, Key::HbConfig)),
        Row("H", i18n::t(lang, Key::SidebarHelp)),
        Row("Q", i18n::t(lang, Key::HbQuit)),
        Blank,
        Section(i18n::t(lang, Key::HelpSectionCatalog)),
        Row("/", i18n::t(lang, Key::HbFilter)),
        Row("I", i18n::t(lang, Key::HbInfo)),
        Row("⇧I", i18n::t(lang, Key::HbInstall)),
        Row("⇧A", i18n::t(lang, Key::HbInstallAlt)),
        Row("?", i18n::t(lang, Key::HbToolHelp)),
        Row("Enter", i18n::t(lang, Key::HbSelect)),
        Blank,
        Section(i18n::t(lang, Key::HelpSectionScan)),
        Row("N", i18n::t(lang, Key::HbRun)),
        Row("?", i18n::t(lang, Key::HbToolHelp)),
        Row("⇧R", i18n::t(lang, Key::HbRerun)),
        Row("X", i18n::t(lang, Key::HbClear)),
        Blank,
        Section(i18n::t(lang, Key::HelpSectionLogs)),
        Row("Enter", i18n::t(lang, Key::HbView)),
        Blank,
        Section(i18n::t(lang, Key::HelpSectionSystemCheck)),
        Row("R", i18n::t(lang, Key::HbRecheck)),
    ]
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let modal = panel::centered(area, 60, 80);
    frame.render_widget(Clear, modal);

    let block = panel::block(theme, i18n::t(lang, Key::HelpTitle), theme.accent_secondary, true)
        .title_bottom(panel::readout(theme, i18n::t(lang, Key::HelpClose), theme.fg_muted));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let lines: Vec<Line> = entries(lang)
        .into_iter()
        .map(|e| match e {
            Entry::Section(title) => Line::from(Span::styled(
                format!(" {} ", title.to_uppercase()),
                theme.plate_style(theme.accent_primary),
            )),
            Entry::Row(k, desc) => Line::from(vec![
                Span::styled(format!("  {k:<18}"), theme.readout_style()),
                Span::styled(desc.to_string(), theme.value_style()),
            ]),
            Entry::Blank => Line::from(""),
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).style(theme.value_style()), inner);
}
