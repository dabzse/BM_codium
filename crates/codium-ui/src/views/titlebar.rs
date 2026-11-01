//! # Titlebar View
//!
//! Provides branding, JetBrains-inspired Search Everywhere pill (`Shift+Shift`),
//! Gutter Run button, Language switch toggle (EN/HU), and panel toggles.

use floem::prelude::*;
use floem::style::CursorStyle;
use floem::text::Weight;
use crate::icons::{render_svg_colored, ICON_BOOK, ICON_PANEL, ICON_PLAY, ICON_SEARCH, ICON_SIDEBAR};
use crate::state::{AppState, CodiumEdition};

/// Top application titlebar component
pub fn titlebar_view(state: AppState) -> impl IntoView {
    let edition = state.edition;
    let show_sidebar = state.show_sidebar;
    let show_bottom_panel = state.show_bottom_panel;
    let show_right_panel = state.show_right_panel;
    let theme = state.theme;
    let locale = state.locale;

    let brand_label = match edition {
        CodiumEdition::Edu => "EDU::Codium",
        CodiumEdition::Ai => "AI::Codium",
    };

    let brand_color = match edition {
        CodiumEdition::Edu => theme.get().accent_edu,
        CodiumEdition::Ai => theme.get().accent_ai,
    };

    h_stack((
        // Left Section: Branding & Localized Subtitle
        h_stack((
            label(move || brand_label).style(move |s| {
                s.font_size(13.5)
                    .font_weight(Weight::BOLD)
                    .color(brand_color)
            }),
            label(move || edition.subtitle(locale.get())).style(move |s| {
                s.font_size(11.0)
                    .color(theme.get().fg_muted)
                    .margin_left(12.0)
            }),
        ))
        .style(|s| s.items_center().margin_left(12.0)),

        // Center Section: JetBrains Search Everywhere Pill (if enabled)
        dyn_container(
            move || state.show_titlebar_search.get(),
            move |is_shown| {
                if !is_shown {
                    return empty().into_any();
                }
                h_stack((
                    render_svg_colored(ICON_SEARCH, 12.0, move || theme.get().fg_muted),
                    label(move || format!(" {}", locale.get().search_everywhere())).style(move |s| {
                        s.font_size(11.5)
                            .color(theme.get().fg_muted)
                            .margin_left(4.0)
                    }),
                ))
                .on_click_stop(move |_| {
                    state.active_sidebar_tab.set(crate::state::SidebarTab::Search);
                    state.show_sidebar.set(true);
                })
                .style(move |s| {
                    s.flex_row()
                        .items_center()
                        .justify_center()
                        .height(26.0)
                        .width(360.0)
                        .background(theme.get().bg_editor)
                        .border(1.0)
                        .border_color(theme.get().border)
                        .border_radius(5.0)
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().hover_bg).border_color(theme.get().accent_blue))
                })
                .into_any()
            },
        ),

        // Right Section: Run button + Language Switcher + Search Toggle + Panel Toggles
        h_stack((
            // JetBrains style Run button (with SVG vector play icon)
            h_stack((
                render_svg_colored(ICON_PLAY, 12.0, move || theme.get().accent_edu),
                label(move || format!(" {}", locale.get().run_btn())).style(move |s| {
                    s.font_size(11.5)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().accent_edu)
                        .margin_left(4.0)
                }),
            ))
            .on_click_stop(move |_| {
                state.show_bottom_panel.set(true);
                state.active_bottom_tab.set(crate::state::BottomTab::Terminal);
                state.terminal_logs.update(|logs| {
                    logs.push("▶ Compiling & Running project...".to_string());
                    logs.push("[Build]: Checking source code...".to_string());
                    logs.push("[Build]: Finished successfully (0 errors).".to_string());
                });
            })
            .style(move |s| {
                s.height(24.0)
                    .padding_horiz(10.0)
                    .items_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().accent_edu)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .margin_right(10.0)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),

            // Toggle Search Pill Visibility in titlebar
            container(
                render_svg_colored(ICON_SEARCH, 13.0, move || {
                    if state.show_titlebar_search.get() {
                        theme.get().fg_primary
                    } else {
                        theme.get().fg_muted
                    }
                }),
            )
            .on_click_stop(move |_| {
                state.show_titlebar_search.update(|v| *v = !*v);
                let is_shown = state.show_titlebar_search.get_untracked();
                state.settings.update(|s| s.show_titlebar_search = is_shown);
                let _ = state.settings.get_untracked().save();
            })
            .style(move |s| {
                s.height(24.0)
                    .width(28.0)
                    .items_center()
                    .justify_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .margin_right(8.0)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),

            // Language Switcher Toggle Pill (EN / HU)
            container(
                label(move || format!("🌐 {}", locale.get().code())).style(move |s| {
                    s.font_size(11.0)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().fg_primary)
                }),
            )
            .on_click_stop(move |_| {
                locale.update(|loc| *loc = loc.toggle());
            })
            .style(move |s| {
                s.height(24.0)
                    .padding_horiz(8.0)
                    .items_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .margin_right(8.0)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),

            // Toggle Primary Sidebar
            h_stack((
                render_svg_colored(ICON_SIDEBAR, 13.0, move || {
                    if show_sidebar.get() { theme.get().fg_primary } else { theme.get().fg_muted }
                }),
                label(move || format!(" {}", locale.get().toggle_sidebar())).style(move |s| {
                    s.font_size(11.0)
                        .color(if show_sidebar.get() { theme.get().fg_primary } else { theme.get().fg_muted })
                        .margin_left(3.0)
                }),
            ))
            .on_click_stop(move |_| {
                show_sidebar.update(|v| *v = !*v);
            })
            .style(move |s| {
                s.height(24.0)
                    .padding_horiz(8.0)
                    .items_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .margin_right(6.0)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),

            // Toggle Bottom Panel
            h_stack((
                render_svg_colored(ICON_PANEL, 13.0, move || {
                    if show_bottom_panel.get() { theme.get().fg_primary } else { theme.get().fg_muted }
                }),
                label(move || format!(" {}", locale.get().toggle_panel())).style(move |s| {
                    s.font_size(11.0)
                        .color(if show_bottom_panel.get() { theme.get().fg_primary } else { theme.get().fg_muted })
                        .margin_left(3.0)
                }),
            ))
            .on_click_stop(move |_| {
                show_bottom_panel.update(|v| *v = !*v);
            })
            .style(move |s| {
                s.height(24.0)
                    .padding_horiz(8.0)
                    .items_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .margin_right(6.0)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),

            // Toggle Right Manuals Panel
            h_stack((
                render_svg_colored(ICON_BOOK, 13.0, move || {
                    if show_right_panel.get() { theme.get().accent_edu } else { theme.get().fg_muted }
                }),
                label(move || format!(" {}", locale.get().tr("Manuals"))).style(move |s| {
                    s.font_size(11.0)
                        .color(if show_right_panel.get() { theme.get().accent_edu } else { theme.get().fg_muted })
                        .margin_left(3.0)
                }),
            ))
            .on_click_stop(move |_| {
                show_right_panel.update(|v| *v = !*v);
            })
            .style(move |s| {
                s.height(24.0)
                    .padding_horiz(8.0)
                    .items_center()
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .cursor(CursorStyle::Pointer)
                    .hover(|s| s.background(theme.get().hover_bg))
            }),
        ))
        .style(|s| s.items_center().margin_right(12.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .height(38.0)
            .items_center()
            .justify_between()
            .background(theme.get().bg_titlebar)
            .border_bottom(1.0)
            .border_color(theme.get().border)
    })
}
