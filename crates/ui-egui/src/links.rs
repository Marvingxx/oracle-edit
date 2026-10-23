//! Project links: the Help menu, the About dialog and the start screen open these.

use serde_json::{Value, json};

use crate::OracleEditApp;

pub const GITHUB: &str = "https://github.com/Marvingxx/oracle-edit";
pub const ISSUES: &str = "https://github.com/Marvingxx/oracle-edit/issues";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] = &[("help.github", GITHUB), ("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut OracleEditApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// The project's GitHub link, centred. Clicks route through [`open`] (the platform browser
/// service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut OracleEditApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let label = tl!("GitHub");
    let font = egui::FontId::proportional(12.5);
    let width = ui.painter().layout_no_wrap(label.into(), font.clone(), t.text).size().x;
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        clicked = ui.link(egui::RichText::new(label).font(font.clone()).color(t.accent)).on_hover_text(GITHUB).clicked();
    });
    if clicked {
        open(app, ui.ctx(), GITHUB);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, start-page links) must open in the browser. They route
        // through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = OracleEditApp::new(oracle_edit_engine::Session::new(), services);
        let ctx = egui::Context::default();
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        open(&mut app, &ctx, GITHUB);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(GITHUB));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = OracleEditApp::new(oracle_edit_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.github", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = OracleEditApp::new(oracle_edit_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in COMMANDS {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], *url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }
}
