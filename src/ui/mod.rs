pub mod config_form;
pub mod form;
pub mod format;
pub mod setup;
pub mod tabs;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Block,
};

use crate::app::App;
use tabs::Tab;

const SIDEBAR_WIDTH: u16 = 22;
const HEADER_HEIGHT: u16 = 3;

pub fn draw(frame: &mut Frame, app: &App) {
    // Fondo de chapa uniforme.
    frame.render_widget(Block::default().style(app.theme.base_style()), frame.area());

    let help_bar_height = widgets::help_bar::height_for(app, frame.area().width);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(help_bar_height),
        ])
        .split(frame.area());

    widgets::header::render(frame, rows[0], app);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
        .split(rows[1]);

    widgets::sidebar::render(frame, cols[0], app);
    draw_body(frame, cols[1], app);
    widgets::help_bar::render(frame, rows[2], app);

    if let Some(form) = &app.config_form {
        config_form::render(frame, frame.area(), form, &app.theme);
    }
    if let Some(form) = &app.form {
        form::render(frame, frame.area(), form, &app.theme, app.lang);
    }
    if app.info_open {
        widgets::info::render(frame, frame.area(), app);
    }
    if app.help_open {
        widgets::help::render(frame, frame.area(), app);
    }
}

fn draw_body(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    match app.active_tab {
        Tab::Overview => tabs::overview::render(frame, area, app),
        Tab::Catalog => tabs::catalog::render(frame, area, app),
        Tab::Scan => tabs::scan::render(frame, area, app),
        Tab::Logs => tabs::logs::render(frame, area, app),
        Tab::SystemCheck => tabs::syscheck::render(frame, area, app),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Reload;
    use crate::config::Config;
    use crate::i18n::Lang;
    use crate::setup::SystemCheckState;
    use crate::tools::Env;
    use ratatui::{Terminal, backend::TestBackend};
    use tokio::sync::watch;

    fn app() -> App {
        let config = Config::default();
        let (config_tx, _rx) = watch::channel(config.clone());
        let env = Env { fixtures: true };
        let system_check = SystemCheckState::new(Lang::default(), env);
        App::new(config, Reload { config: config_tx }, env, system_check)
    }

    #[test]
    fn dibuja_el_tablero_en_varios_tamanos() {
        let app = app();
        for (w, h) in [(120, 40), (80, 24), (40, 12), (20, 6)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|frame| draw(frame, &app)).unwrap();
        }
    }

    #[test]
    fn dibuja_todas_las_pestanas() {
        let mut app = app();
        for tab in Tab::ALL {
            app.active_tab = tab;
            let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
            terminal.draw(|frame| draw(frame, &app)).unwrap();
        }
    }

    #[test]
    fn dibuja_los_overlays() {
        let mut app = app();
        app.active_tab = Tab::Catalog;
        app.info_open = true;
        app.info_tool = Some("nmap");
        app.help_open = true;
        app.config_form = Some(crate::config::ConfigForm::new(&app.config));
        for (w, h) in [(120, 40), (80, 24), (40, 12)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|frame| draw(frame, &app)).unwrap();
        }
    }

    #[test]
    fn dibuja_el_wizard_de_arranque() {
        let env = Env { fixtures: true };
        let state = SystemCheckState::new(Lang::default(), env);
        let theme = crate::theme::Theme::default();
        for (w, h) in [(120, 40), (60, 20)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|frame| setup::render_gate(frame, frame.area(), &state, &theme)).unwrap();
        }
    }
}
