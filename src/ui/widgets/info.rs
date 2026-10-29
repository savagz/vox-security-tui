use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::app::App;
use crate::i18n::{self, Key};
use crate::tools;

use super::panel;

/// Popup de detalle de la herramienta seleccionada en el Catálogo (tecla `i`):
/// categoría, licencia, binario, estado, propósito y paquetes por distro.
/// Scrolleable con ↑↓, cualquier otra tecla cierra.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lang = app.lang;

    let Some(tool) = app.info_tool.and_then(tools::catalog::by_id) else {
        return;
    };

    let modal = panel::centered(area, 70, 70);
    frame.render_widget(Clear, modal);

    let title = i18n::t(lang, Key::InfoTitleTemplate).replacen("{}", tool.name, 1);
    let block = panel::block(theme, &title, theme.accent_primary, true)
        .title_bottom(panel::readout(theme, i18n::t(lang, Key::InfoHint), theme.fg_muted));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let installed = app.tool_installed(tool.id);
    let status_text = if installed {
        i18n::t(lang, Key::StatusInstalled)
    } else {
        i18n::t(lang, Key::StatusMissing)
    };
    let status_color = if installed { theme.ok } else { theme.warning };

    let field = |k: Key, v: String| -> Line {
        Line::from(vec![
            Span::styled(format!("{:<14}", i18n::t(lang, k)), theme.label_style()),
            Span::styled(v, theme.value_style()),
        ])
    };

    let mut lines = vec![
        field(Key::InfoFieldCategory, tool.category.label(lang).to_string()),
        field(Key::InfoFieldLicense, tool.license.to_string()),
        field(Key::InfoFieldBinary, tool.binary.to_string()),
        Line::from(vec![
            Span::styled(format!("{:<14}", i18n::t(lang, Key::InfoFieldStatus)), theme.label_style()),
            Span::styled(status_text.to_string(), theme.style_for_color(status_color)),
        ]),
        Line::from(""),
        Line::from(Span::styled(i18n::t(lang, Key::InfoFieldPurpose), theme.plate_style(theme.accent_secondary))),
        Line::from(Span::styled(tool.purpose(lang).to_string(), theme.value_style())),
    ];

    // Plantillas de uso: nombre + línea de argumentos.
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        i18n::t(lang, Key::InfoFieldTemplates),
        theme.plate_style(theme.accent_secondary),
    )));
    for t in tool.templates {
        lines.push(Line::from(Span::styled(format!("  · {}", t.name(lang)), theme.label_style())));
        lines.push(Line::from(Span::styled(format!("    {} {}", tool.binary, t.args), theme.readout_style())));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        i18n::t(lang, Key::InfoFieldInstall),
        theme.plate_style(theme.accent_secondary),
    )));
    lines.push(Line::from(Span::styled(format!("  debian/kali : {}", tool.pkg_debian), theme.value_style())));
    lines.push(Line::from(Span::styled(format!("  arch        : {}", tool.pkg_arch), theme.value_style())));
    lines.push(Line::from(Span::styled(format!("  fedora      : {}", tool.pkg_fedora), theme.value_style())));

    // Vías alternativas (fuera de repos oficiales), si la herramienta las define.
    if tool.go_install.is_some() || tool.copr.is_some() {
        lines.push(Line::from(""));
        if let Some(path) = tool.go_install {
            lines.push(Line::from(vec![
                Span::styled(format!("{:<14}", i18n::t(lang, Key::InfoFieldGo)), theme.label_style()),
                Span::styled(format!("go install {path}"), theme.value_style()),
            ]));
        }
        if let Some(copr) = tool.copr {
            lines.push(Line::from(vec![
                Span::styled(format!("{:<14}", i18n::t(lang, Key::InfoFieldCopr)), theme.label_style()),
                Span::styled(format!("dnf copr enable {copr}"), theme.value_style()),
            ]));
        }
    }

    if let Some(path) = app.tool_states.iter().find(|s| s.id == tool.id).and_then(|s| s.path.clone()) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!("  {path}"), theme.readout_style())));
    }

    let line_count = lines.len() as u16;
    let max_scroll = line_count.saturating_sub(inner.height);
    let scroll = app.info_scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(lines).style(theme.value_style()).wrap(Wrap { trim: false }).scroll((scroll, 0)),
        inner,
    );
}
