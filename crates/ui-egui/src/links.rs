//! Opening web links (update notes, contributor profiles) in the system browser. The product has
//! no community or project links of its own yet, so there are no Help-menu link commands.

use serde_json::{Value, json};

use crate::PhotocraftApp;

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links must open in the browser through the `open_url` service rather than
        // the unreliable `ctx.open_url`.
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
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        let url = "https://example.com/";
        assert_eq!(open(&mut app, &ctx, url)["url"], url);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(url));
        assert_eq!(app.ui.status, format!("Opened {url}"));
    }

    #[test]
    fn help_menu_has_no_upstream_links() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        for gone in ["help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue"] {
            assert!(!help.iter().any(|id| id == gone), "{gone} in {help:?}");
        }
        assert!(help.iter().any(|id| id == "help.about"), "{help:?}");
    }
}
