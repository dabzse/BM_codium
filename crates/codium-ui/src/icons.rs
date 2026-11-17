//! # Vector Icon Library (SVG) & Window Icon Generator
//!
//! Replaces text emojis with crisp, hardware-accelerated vector SVG icons
//! from Codicons, rendered natively with Floem's SVG subsystem.
//! Also provides native OS window icons for Linux/Windows taskbar integration.

use floem::prelude::*;
use floem::peniko::Color;

// 16x16 / 24x24 Vector SVG Icon Definitions

/// Explorer / Files Icon (Codicons)
pub const ICON_FILES: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M17.5 0h-9L7 1.5V6H2.5L1 7.5v15.07L2.5 24h12.07L16 22.57V18h4.7l1.3-1.43V4.5L17.5 0zm0 2.12l2.38 2.38H17.5V2.12zm-3 20.38h-12v-15H7v9.07L8.5 18h6v4.5zm6-6h-12v-15H16V6h4.5v10.5z"/></svg>"#;

/// Search Icon (Codicons)
pub const ICON_SEARCH: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M15.25 0a8.25 8.25 0 0 0-6.18 13.72L1 22.88l1.12 1 8.05-9.12A8.251 8.251 0 1 0 15.25.01V0zm0 15a6.75 6.75 0 1 1 0-13.5 6.75 6.75 0 0 1 0 13.5z"/></svg>"#;

/// Settings Gear Icon (Codicons)
pub const ICON_SETTINGS: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M19.85 8.75l4.15.83v4.84l-4.15.83 2.35 3.52-3.43 3.43-3.52-2.35-.83 4.15H9.58l-.83-4.15-3.52 2.35-3.43-3.43 2.35-3.52L0 14.42V9.58l4.15-.83L1.8 5.23 5.23 1.8l3.52 2.35L9.58 0h4.84l.83 4.15 3.52-2.35 3.43 3.43-2.35 3.52zm-1.57 5.07l4-.81v-2l-4-.81-.54-1.3 2.29-3.43-1.43-1.43-3.43 2.29-1.3-.54-.81-4h-2l-.81 4-1.3.54-3.43-2.29-1.43 1.43L6.38 8.9l-.54 1.3-4 .81v2l4 .81.54 1.3-2.29 3.43 1.43 1.43 3.43-2.29 1.3.54.81 4h2l.81-4 1.3-.54 3.43 2.29 1.43-1.43-2.29-3.43.54-1.3zm-8.186-4.672A3.43 3.43 0 0 1 12 8.57 3.44 3.44 0 0 1 15.43 12a3.43 3.43 0 1 1-5.336-2.852zm.956 4.274c.281.188.612.288.95.288A1.7 1.7 0 0 0 13.71 12a1.71 1.71 0 1 0-2.66 1.422z"/></svg>"#;

/// Run / Play Triangle Icon (Codicons)
pub const ICON_PLAY: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M4.25 3l1.166-.624 8 5.333v1.248l-8 5.334-1.166-.624V3zm1.5 1.401v7.864l5.898-3.932L5.75 4.401z"/></svg>"#;

/// Terminal Icon (Codicons)
pub const ICON_TERMINAL: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M3 1.5L1.5 3v18L3 22.5h18l1.5-1.5V3L21 1.5H3zM3 21V3h18v18H3zm5.656-4.01l1.038 1.061 5.26-5.243v-.912l-5.26-5.26-1.035 1.06 4.59 4.702-4.593 4.592z"/></svg>"#;

/// Warning / Problems Icon (Codicons)
pub const ICON_WARNING: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M7.56 1h.88l6.54 12.26-.44.74H1.44L1 13.26 7.56 1zM8 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm0-3.5a.75.75 0 0 0 .75-.75v-3.5a.75.75 0 0 0-1.5 0v3.5c0 .414.336.75.75.75z"/></svg>"#;

/// Graduation Cap / EDU Tutor Icon
pub const ICON_GRADUATION: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M12 3L1 9l11 6 9-4.91V17h2V9L12 3zm0 3.72L17.9 9 12 12.22 6.1 9 12 6.72zM5 13.18v4L12 21l7-3.82v-4L12 17l-7-3.82z"/></svg>"#;

/// Sparkle / AI Assistant Icon
pub const ICON_SPARKLE: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M12 2L9.5 8.5 3 11l6.5 2.5L12 20l2.5-6.5L21 11l-6.5-2.5L12 2zm6 14l-1.25 3.25L13.5 20.5l3.25 1.25L18 25l1.25-3.25L22.5 20.5l-3.25-1.25L18 16z"/></svg>"#;

/// Sidebar Toggle Icon
pub const ICON_SIDEBAR: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 2h12v12H2V2zm1 1v10h3V3H3zm4 10h6V3H7v10z"/></svg>"#;

/// Bottom Panel Toggle Icon
pub const ICON_PANEL: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 2h12v12H2V2zm1 1v7h10V3H3zm0 8v2h10v-2H3z"/></svg>"#;

/// Book / Documentation / Manuals Icon (Codicons)
pub const ICON_BOOK: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M1 2.5l.5-.5H7l.5.5v10l-.5.5H1.5l-.5-.5v-10zM2 3v9h4.5V3H2zm7.5-.5l.5-.5h5.5l.5.5v10l-.5.5H10l-.5-.5v-10zm1 .5v9H15V3h-4.5z"/></svg>"#;

/// Replace All Icon (Codicons)
pub const ICON_REPLACE_ALL: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M1.5 2h13l.5.5v2l-.5.5h-13L1 4.5v-2l.5-.5zM2 4h12V3H2v1zm-.5 4h13l.5.5v2l-.5.5h-13l-.5-.5v-2l.5-.5zM2 10h12V9H2v1zm-.5 4h13l.5.5v2l-.5.5h-13l-.5-.5v-2l.5-.5zM2 16h12v-1H2v1z"/></svg>"#;

/// Source Control / OmniGit Branch Icon (Codicons)
pub const ICON_SOURCE_CONTROL: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M12.5 5a2.5 2.5 0 1 0-1.954 4.056A4.002 4.002 0 0 1 7 12.83V9.83a2.5 2.5 0 1 0-2 0v4.34A2.5 2.5 0 1 0 6.5 15a4.004 4.004 0 0 0 3.82-2.8A2.502 2.502 0 0 0 12.5 5zM5 8a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm6.5-1.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zm-6 8a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z"/></svg>"#;

/// Checkmark / Commit Icon (Codicons)
pub const ICON_CHECK: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0z"/></svg>"#;

/// Refresh Icon (Codicons)
pub const ICON_REFRESH: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M4.681 3H2V2h3.5l.5.5V6H5V4a5 5 0 1 0 4.53-.761l.302-.954A6 6 0 1 1 4.681 3z"/></svg>"#;

/// Plus / Stage Icon (Codicons)
pub const ICON_PLUS: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/></svg>"#;

/// Minus / Unstage Icon (Codicons)
pub const ICON_MINUS: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M2.75 7.25h10.5a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5z"/></svg>"#;

/// Git Branch Icon (Codicons)
pub const ICON_GIT_BRANCH: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M11.75 2a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5zm-7.5 0a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5zm0 8.75a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5zm4.825-4.875a3.25 3.25 0 0 1 2.675-2.625V4.75a1.75 1.75 0 0 0-1.75 1.75v2.5a1.75 1.75 0 0 1-1.75 1.75H6.25v1.5h1.925a3.25 3.25 0 0 0 3.25-3.25v-2.5a3.24 3.24 0 0 0-.625-1.875zM5 5.5v5h-1.5v-5H5z"/></svg>"#;

/// History / Commits Icon (Codicons)
pub const ICON_HISTORY: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M8.5 2a6.5 6.5 0 1 0 5.485 10.024l-.84-.543A5.5 5.5 0 1 1 8.5 3a5.474 5.474 0 0 1 3.86 1.583L10.5 6.5H15V2l-1.929 1.929A6.474 6.474 0 0 0 8.5 2zm-.5 3v3.707l2.646 2.647.708-.708L9 8.293V5H8z"/></svg>"#;

/// Tag Icon (Codicons)
pub const ICON_TAG: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M1 1.75l.75-.75h4.69L14.7 9.2a1.75 1.75 0 0 1 0 2.47l-2.37 2.37a1.75 1.75 0 0 1-2.47 0L1.69 5.87V1.75H1zm1.5.75v3.31l8.17 8.17.88-.88-8.17-8.17H2.5zm2.5 1a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"/></svg>"#;

/// Copy Icon (Codicons)
pub const ICON_COPY: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M4 4h8v8H4V4zm-1 9a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V4a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v9zm-1-8V2a1 1 0 0 1 1-1h8v1H3v8H2a1 1 0 0 1-1-1V5h1z"/></svg>"#;

/// Chevron Down Icon (Codicons)
pub const ICON_CHEVRON_DOWN: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M7.976 10.072l4.357-4.357.62.618L8.284 11h-.618L3 6.333l.619-.618 4.357 4.357z"/></svg>"#;

/// Chevron Right Icon (Codicons)
pub const ICON_CHEVRON_RIGHT: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M10.072 8.024L5.715 3.667l.618-.62L11 7.716v.618L6.333 13l-.618-.619 4.357-4.357z"/></svg>"#;

/// Arrow Up Icon (Codicons)
pub const ICON_ARROW_UP: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M8 3.293l4.146 4.147-.708.708L8.5 5.207V13h-1V5.207L4.561 8.148l-.707-.708L8 3.293z"/></svg>"#;

/// Arrow Down Icon (Codicons)
pub const ICON_ARROW_DOWN: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M8 12.707l-4.146-4.147.708-.708L7.5 10.793V3h1v7.793l2.939-2.941.707.708L8 12.707z"/></svg>"#;

/// Expand / Maximize Icon (Codicons)
pub const ICON_EXPAND: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="M3 3v4h1V4h3V3H3zm10 0h-4v1h3v3h1V3zm-1 10h-3v-1h3v-3h1v4h-1zm-8 0H3V9h1v3h3v1H4z"/></svg>"#;

/// Dock / Panel Split Icon (Codicons)
pub const ICON_DOCK_PANEL: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 2h12v12H2V2zm1 1v6h10V3H3zm0 7v3h10v-3H3z"/></svg>"#;

/// View as Tree Icon (Codicons list-tree)
pub const ICON_LIST_TREE: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M1 2h3v3H1V2zm4 1H2v1h2V3zm1 7h3v3H6v-3zm4 1H7v1h2v-1zm1-8h4v3h-4V3zm3 1h-2v1h2V4zM4 6.5A1.5 1.5 0 0 0 2.5 8v2.5H4V9h3.5v1.5H9V8a1.5 1.5 0 0 0-1.5-1.5h-3.5zM9 13.5A1.5 1.5 0 0 0 10.5 12V9.5H9V11H6.5V9.5H5V12a1.5 1.5 0 0 0 1.5 1.5h2z"/></svg>"#;

/// View as Flat List Icon (Codicons list-flat)
pub const ICON_LIST_FLAT: &str = r#"<svg width="14" height="14" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 3h12v1H2V3zm0 4h12v1H2V7zm0 4h12v1H2v-1zm0 4h12v1H2v-1z"/></svg>"#;

/// Helper to render an SVG icon with specified dimensions and crisp default light gray color
pub fn render_svg(svg_content: &'static str, size: f64) -> impl IntoView {
    svg(move || svg_content.to_string())
        .style(move |s| s.size(size, size).color(Color::from_rgb8(200, 200, 200)))
}

/// Helper to render an SVG icon with dynamic color reactively updated
pub fn render_svg_colored(
    svg_content: &'static str,
    size: f64,
    color_fn: impl Fn() -> Color + 'static,
) -> impl IntoView {
    svg(move || svg_content.to_string())
        .style(move |s| s.size(size, size).color(color_fn()))
}

/// Programmatically generates a crisp 32x32 RGBA window icon for native OS taskbar integration
pub fn codium_window_icon(is_edu: bool) -> Option<floem::window::Icon> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut rgba = vec![0u8; (W * H * 4) as usize];
    let (accent_r, accent_g, accent_b) = if is_edu {
        (26, 188, 156) // #1abc9c New Testament Crystal Jasper
    } else {
        (142, 68, 173) // #8e44ad Purple
    };

    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            // Rounded corners check (radius 6)
            let dx = if x < 6 { 6 - x } else if x >= W - 6 { x - (W - 7) } else { 0 };
            let dy = if y < 6 { 6 - y } else if y >= H - 6 { y - (H - 7) } else { 0 };
            if dx * dx + dy * dy > 36 {
                // Outside rounded corners: transparent
                continue;
            }

            // Border (1px)
            let is_border = x == 0 || x == W - 1 || y == 0 || y == H - 1
                || (dx * dx + dy * dy >= 25);

            if is_border {
                rgba[idx] = accent_r;
                rgba[idx + 1] = accent_g;
                rgba[idx + 2] = accent_b;
                rgba[idx + 3] = 255;
            } else {
                // Dark background #1e1e24
                rgba[idx] = 30;
                rgba[idx + 1] = 30;
                rgba[idx + 2] = 36;
                rgba[idx + 3] = 255;

                // Draw stylized 'C' in the center
                let in_c_box = (8..=23).contains(&y) && (9..=22).contains(&x);
                if in_c_box {
                    let is_c_top = (8..=11).contains(&y);
                    let is_c_bottom = (20..=23).contains(&y);
                    let is_c_left = (9..=13).contains(&x);
                    if is_c_top || is_c_bottom || is_c_left {
                        rgba[idx] = accent_r;
                        rgba[idx + 1] = accent_g;
                        rgba[idx + 2] = accent_b;
                        rgba[idx + 3] = 255;
                    }
                }
            }
        }
    }

    floem::window::Icon::from_rgba(rgba, W, H).ok()
}
