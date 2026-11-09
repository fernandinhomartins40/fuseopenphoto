use super::*;
use crate::state::{DialogKind, UiState};
use egui_kittest::{Harness, kittest::Queryable};

fn harness(size: egui::Vec2, classic: bool) -> Harness<'static, PhotocraftApp> {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.studio.classic = classic;
    let mut h = Harness::builder().with_size(size).build_ui_state(|ui, app| crate::dialogs::show(app, ui.ctx()), app);
    PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::ALL[0]);
    h.state_mut().ui.open_dialog(DialogKind::NewDocument, UiState::new_document_fields());
    h.run_steps(3);
    h
}

fn fields(h: &Harness<'static, PhotocraftApp>) -> Map<String, Value> {
    h.state().ui.dialogs.first().map(|d| d.fields.clone()).expect("the page is open")
}

#[test]
fn the_formats_are_the_dialogs_categories_then_saved() {
    let keys: Vec<&str> = FORMATS.iter().map(|f| f.0).collect();
    let mut expected: Vec<&str> = super::super::CATEGORIES.iter().map(|c| c.0).collect();
    expected.push("Saved");
    assert_eq!(keys, expected);
    for (_, icon) in FORMATS.iter().filter(|f| !f.1.is_empty()) {
        assert!(icons::exists(icon), "{icon}");
    }
}

#[test]
fn summary_reads_size_mode_depth_and_resolution() {
    let f = json!({"width": 1913, "height": 1029, "resolution": 72.0, "mode": "rgb", "depth": 8});
    assert_eq!(summary(f.as_object().unwrap()), "1913 × 1029 px · RGB · 8 bits · 72 ppi");
    let f = json!({"width": 10, "height": 20, "resolution": 300.0, "mode": "cmyk", "depth": 16});
    assert_eq!(summary(f.as_object().unwrap()), "10 × 20 px · CMYK · 16 bits · 300 ppi");
}

/// Pick a format and a preset on the page, then Create document: a document of that size.
#[test]
fn picking_a_preset_and_create_makes_that_document() {
    let mut h = harness(egui::vec2(1600.0, 1000.0), false);
    assert!(h.query_by_label("PRESET DETAILS").is_none(), "the studio shows the page, not the dialog");
    h.get_by_label("Photo").click();
    h.run_steps(2);
    h.get_by_label("Landscape, 6 x 4").click();
    h.run_steps(2);
    assert_eq!(fields(&h)["__preset"], "Landscape, 6 x 4");
    h.get_by_label("Create document").click();
    h.run_steps(3);
    assert!(h.state().ui.dialogs.is_empty(), "the page closed");
    let d = &h.state().session.active().expect("a new document").doc;
    assert_eq!((d.size.width, d.size.height, d.resolution_dpi), (1800, 1200, 300.0));
    assert!(h.state().ui.views[0].fit_pending, "made as the dialog's Create makes it");
}

#[test]
fn back_and_escape_close_without_creating() {
    let mut h = harness(egui::vec2(1600.0, 1000.0), false);
    h.get_by_label("Back").click();
    h.run_steps(2);
    assert!(h.state().ui.dialogs.is_empty());
    h.state_mut().ui.open_dialog(DialogKind::NewDocument, UiState::new_document_fields());
    h.run_steps(2);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(h.state().ui.dialogs.is_empty());
    assert!(h.state().session.documents().is_empty());
}

/// Every control is on screen in a small window and a large one.
#[test]
fn the_page_fits_720p_and_1080p() {
    for size in [egui::vec2(1280.0, 720.0), egui::vec2(1920.0, 1080.0)] {
        let h = harness(size, false);
        let screen = Rect::from_min_size(egui::Pos2::ZERO, size);
        // A field's label and the field itself share the name: check every node of it.
        for name in ["Back", "Create document", "Search command", "Recent", "Saved", "Name", "Width", "Height", "Resolution"] {
            let rects: Vec<Rect> = h.query_all_by_label(name).map(|n| n.rect()).collect();
            assert!(!rects.is_empty(), "{name} is on the page");
            for r in rects {
                assert!(screen.contains_rect(r), "{name} at {r:?} in {size:?}");
            }
        }
    }
}

#[test]
fn the_classic_layout_keeps_the_dialog() {
    let h = harness(egui::vec2(1400.0, 900.0), true);
    assert!(h.query_by_label("PRESET DETAILS").is_some());
    assert!(h.query_by_label("Create document").is_none());
}

#[test]
fn linked_size_keeps_the_aspect_and_orientation_swaps() {
    let mut f = json!({"width": 2000, "height": 1000}).as_object().unwrap().clone();
    keep_aspect(&mut f, "height", 1000.0 * 1000.0 / 2000.0);
    assert_eq!(f["height"], 500);
    super::super::set_orientation(&mut f, true);
    assert_eq!((f["width"].as_u64(), f["height"].as_u64()), (Some(500), Some(2000)));
}

#[test]
fn the_link_button_and_the_transparent_box_edit_the_fields() {
    let mut h = harness(egui::vec2(1600.0, 1000.0), false);
    h.get_by_label("Link width and height").click();
    h.run_steps(2);
    assert_eq!(fields(&h)["__linked"], true);
    h.get_by_label("Transparent").click();
    h.run_steps(2);
    assert_eq!(fields(&h)["background"], "transparent");
    h.get_by_label("Create document").click();
    h.run_steps(3);
    let pixel = h.state_mut().session.execute("document.pixel", json!({"x": 0, "y": 0})).unwrap();
    assert_eq!(pixel[3], 0.0);
}
