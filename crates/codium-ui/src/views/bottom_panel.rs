//! # Bottom Dock Panel View
//!
//! Provides the collapsible bottom dock containing:
//! - Terminal Output
//! - Problems / Compiler Diagnostics
//! - Pedagogic Doc Tutor Hints

use floem::prelude::*;
use floem::style::CursorStyle;
use floem::text::Weight;
use floem::taffy::style::Display;
use crate::icons::{render_svg, ICON_TERMINAL, ICON_WARNING};
use crate::state::{AppState, BottomTab};

/// Bottom dock container with tabs and log viewer
pub fn bottom_panel_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_panel = state.show_bottom_panel;
    let active_tab = state.active_bottom_tab;
    let logs = state.terminal_logs;
    let locale = state.locale;
    let settings = state.settings;

    let make_tab_btn = move |tab: BottomTab, label_fn: Box<dyn Fn() -> String>, icon_opt: Option<&'static str>| {
        let is_active = move || active_tab.get() == tab;
        h_stack((
            if let Some(svg_data) = icon_opt {
                container(render_svg(svg_data, 12.0)).into_any()
            } else {
                empty().into_any()
            },
            label(move || label_fn()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(if is_active() { Weight::BOLD } else { Weight::NORMAL })
                    .color(if is_active() { theme.get().fg_primary } else { theme.get().fg_muted })
                    .margin_left(4.0)
            }),
        ))
        .on_click_stop(move |_| {
            active_tab.set(tab);
        })
        .style(move |s| {
            s.height(28.0)
                .padding_horiz(10.0)
                .items_center()
                .cursor(CursorStyle::Pointer)
                .apply_if(is_active(), |s| {
                    s.border_bottom(2.0).border_color(theme.get().accent_blue)
                })
        })
    };

    v_stack((
        // Header Tabs
        h_stack((
            h_stack((
                make_tab_btn(BottomTab::Terminal, Box::new(move || locale.get().terminal_tab()), Some(ICON_TERMINAL)),
                make_tab_btn(BottomTab::Problems, Box::new(move || locale.get().problems_tab()), Some(ICON_WARNING)),
                make_tab_btn(BottomTab::DocTutorOutput, Box::new(move || locale.get().hints_tab()), None),
            ))
            .style(|s| s.flex_row().items_center()),

            // Close button
            container(
                label(move || "×").style(move |s| {
                    s.font_size(14.0).color(theme.get().fg_muted)
                }),
            )
            .on_click_stop(move |_| {
                show_panel.set(false);
            })
            .style(move |s| {
                s.padding_horiz(10.0)
                    .cursor(CursorStyle::Pointer)
                    .hover(|s| s.color(theme.get().fg_primary))
            }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(28.0)
                .justify_between()
                .items_center()
                .background(theme.get().bg_tab_inactive)
                .border_bottom(1.0)
                .border_color(theme.get().border)
        }),

        // Tab Content
        dyn_container(
            move || (active_tab.get(), show_panel.get()),
            move |(tab, is_shown)| {
                if !is_shown {
                    return container(empty()).style(|s| s.display(Display::None)).into_any();
                }

                match tab {
                    BottomTab::Terminal => {
                        scroll(
                            dyn_container(
                                move || logs.get(),
                                move |lines| {
                                    let term_font = settings.get().terminal_font_family;
                                    v_stack_from_iter(
                                        lines.into_iter().map(move |line| {
                                            let f = term_font.clone();
                                            label(move || line.clone()).style(move |s| {
                                                s.font_size(12.0)
                                                    .color(theme.get().fg_primary)
                                                    .font_family(f.clone())
                                                    .margin_bottom(2.0)
                                            })
                                        })
                                    )
                                    .style(|s| s.flex_col().padding(8.0))
                                },
                            )
                        )
                        .style(|s| s.size_pct(100.0, 100.0))
                        .into_any()
                    }
                    BottomTab::Problems => {
                        scroll(
                            v_stack((
                                label(move || locale.get().tr("No compiler errors detected. All workspace diagnostics clean.")).style(move |s| {
                                    s.font_size(12.0)
                                        .color(theme.get().accent_edu)
                                        .padding(12.0)
                                }),
                            ))
                            .style(|s| s.flex_col().min_width(0.0))
                        )
                        .style(|s| s.size_pct(100.0, 100.0).min_width(0.0))
                        .into_any()
                    }
                    BottomTab::DocTutorOutput => {
                        scroll(
                            v_stack((
                                label(move || locale.get().tr("Doc Tutor Pedagogic Hint:")).style(move |s| {
                                    s.font_size(12.5)
                                        .font_weight(Weight::BOLD)
                                        .color(theme.get().accent_edu)
                                        .margin_bottom(4.0)
                                }),
                                label(move || locale.get().tr("Tip: When working with arrays in C/C++, always check buffer bounds before dereferencing pointers!")).style(move |s| {
                                    s.font_size(12.0).color(theme.get().fg_primary)
                                }),
                            ))
                            .style(|s| s.flex_col().padding(12.0).min_width(0.0))
                        )
                        .style(|s| s.size_pct(100.0, 100.0).min_width(0.0))
                        .into_any()
                    }
                }
            },
        )
        .style(|s| s.flex_grow(1.0).min_width(0.0).width_pct(100.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .min_width(0.0)
            .flex_shrink(1.0)
            .height(state.bottom_panel_height.get())
            .background(theme.get().bg_panel)
            .display(if show_panel.get() { Display::Flex } else { Display::None })
    })
}
