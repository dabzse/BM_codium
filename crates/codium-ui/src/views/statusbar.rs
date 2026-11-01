//! # Status Bar View
//!
//! Bottom status bar showing:
//! - Left: Mode badge (EDU vs AI) and active status message
//! - Right: Cursor coordinates (Ln/Col), Spaces, Encoding, Language, and Quick Locale switch

use floem::prelude::*;
use floem::style::CursorStyle;
use floem::text::Weight;
use crate::state::{AppState, CodiumEdition};

/// Bottom status bar component
pub fn statusbar_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let edition = state.edition;
    let status = state.status_message;
    let ws = state.workspace;
    let locale = state.locale;

    let mode_pill_bg = match edition {
        CodiumEdition::Edu => theme.get().accent_edu,
        CodiumEdition::Ai => theme.get().accent_ai,
    };

    h_stack((
        // Left Status Indicators
        h_stack((
            // Mode Pill Badge
            container(
                label(move || {
                    match edition {
                        CodiumEdition::Edu => locale.get().edu_mode_pill(),
                        CodiumEdition::Ai => locale.get().ai_mode_pill(),
                    }
                }).style(|s| {
                    s.font_size(10.5)
                        .font_weight(Weight::BOLD)
                        .color(palette::css::WHITE)
                }),
            )
            .style(move |s| {
                s.background(mode_pill_bg)
                    .padding_horiz(8.0)
                    .height(20.0)
                    .border_radius(3.0)
                    .items_center()
                    .margin_right(10.0)
            }),

            // Status message
            label(move || status.get()).style(move |s| {
                s.font_size(11.0).color(theme.get().fg_primary)
            }),
        ))
        .style(|s| s.items_center().margin_left(8.0)),

        // Right Editor Info & Language switch
        h_stack((
            // Cursor Coordinates
            label(move || {
                let w = ws.get();
                if let Some(doc) = w.active_document() {
                    let (line, col) = doc.line_col(doc.cursor());
                    format!("Ln {}, Col {}", line, col)
                } else {
                    "Ln 1, Col 1".to_string()
                }
            }).style(move |s| {
                s.font_size(11.0).color(theme.get().fg_muted).margin_right(12.0)
            }),

            // Spaces Indent
            label(move || "Spaces: 4").style(move |s| {
                s.font_size(11.0).color(theme.get().fg_muted).margin_right(12.0)
            }),

            // File Encoding
            label(move || "UTF-8").style(move |s| {
                s.font_size(11.0).color(theme.get().fg_muted).margin_right(12.0)
            }),

            // Active Language
            label(move || {
                let w = ws.get();
                if let Some(doc) = w.active_document() {
                    doc.language().name().to_string()
                } else {
                    "Plain Text".to_string()
                }
            }).style(move |s| {
                s.font_size(11.0).color(theme.get().fg_primary).margin_right(12.0)
            }),

            // Quick Locale Switcher (EN / HU)
            container(
                label(move || format!("🌐 {}", locale.get().code())).style(move |s| {
                    s.font_size(10.5)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().fg_primary)
                }),
            )
            .on_click_stop(move |_| {
                locale.update(|loc| *loc = loc.toggle());
            })
            .style(move |s| {
                s.padding_horiz(6.0)
                    .height(18.0)
                    .items_center()
                    .background(theme.get().hover_bg)
                    .border_radius(3.0)
                    .cursor(CursorStyle::Pointer)
            }),
        ))
        .style(|s| s.items_center().margin_right(8.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .height(24.0)
            .items_center()
            .justify_between()
            .background(theme.get().bg_titlebar)
            .border_top(1.0)
            .border_color(theme.get().border)
    })
}
