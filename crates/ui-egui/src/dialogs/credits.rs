//! The About window's "Open-source licenses" tab: the code's copyright and licence notices
//! (NOTICE, LICENSE-MIT, LICENSE-APACHE), baked in from the repository root so they always match
//! what ships. The per-file third-party list (ATTRIBUTION.md) is installed beside the app.

use egui::RichText;

/// The notices, in the order shown: (file name, text).
pub const LICENSES: [(&str, &str); 3] = [
    ("NOTICE", include_str!("../../../../NOTICE")),
    ("LICENSE-MIT", include_str!("../../../../LICENSE-MIT")),
    ("LICENSE-APACHE", include_str!("../../../../LICENSE-APACHE")),
];

/// The licences tab: a short explanation, then every notice in one scrolling, selectable column.
pub fn licenses_ui(ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.label(crate::i18n::fmt(
        tl!("{app} is built on open-source software. Its licenses and copyright notices are below; the full third-party list is in ATTRIBUTION.md, installed with the app."),
        &[("app", crate::studio::style::APP_NAME)],
    ));
    ui.add_space(6.0);
    egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show(ui, |ui| {
        for (name, text) in LICENSES {
            ui.label(RichText::new(name).strong().color(t.text));
            ui.add(egui::Label::new(RichText::new(text).font(crate::theme::mono(11.0)).color(t.text_faint)).selectable(true));
            ui.add_space(10.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_licence_texts_are_baked_in() {
        assert!(LICENSES[0].1.contains("MIT License or the Apache License"));
        assert!(LICENSES[1].1.starts_with("MIT License"));
        assert!(LICENSES[2].1.contains("Apache License"));
    }
}
