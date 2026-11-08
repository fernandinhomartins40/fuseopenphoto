//! Shared studio widgets and measures, so every region draws the same pills, cards and icons.
//! Colours come from `theme::Tokens` (the Studio theme is the amber house look).

/// Top bar height.
pub const TOP_BAR_H: f32 = 52.0;
/// Mode bar height.
pub const MODE_BAR_H: f32 = 54.0;
/// Corner radius of cards and pills.
pub const RADIUS: f32 = 10.0;

// Product identity. A rename changes these lines (plus the installer's `photocraft.wxs` defines,
// `install-policy.json` and the Windows version resources in `apps/*/build.rs`).
/// The product name: top bar, window titles, About, menus.
pub const APP_NAME: &str = "Fuse Open Photo";
/// Reverse-DNS app id: the Wayland/X11 window class and the Linux settings folder name.
pub const APP_ID: &str = "com.fuse.openphoto";
/// The company behind the product, shown in About.
pub const COMPANY: &str = "Fuse";
