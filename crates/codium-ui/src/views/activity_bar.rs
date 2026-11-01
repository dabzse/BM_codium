//! # Activity Bar View
//!
//! Left vertical strip holding vector icons for Explorer, Search, Doc Tutor (or AI Assistant),
//! and Settings, with active border accent indicators.

use floem::prelude::*;
use floem::style::CursorStyle;
use crate::icons::{
    render_svg, ICON_FILES, ICON_SEARCH, ICON_GRADUATION, ICON_SPARKLE, ICON_SETTINGS,
};
use crate::state::{AppState, CodiumEdition, SidebarTab};

/// Primary vertical activity icon strip
pub fn activity_bar_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let active_tab = state.active_sidebar_tab;
    let show_sidebar = state.show_sidebar;
    let edition = state.edition;

    let make_svg_btn = move |tab: SidebarTab, svg_content: &'static str| {
        let is_active = move || active_tab.get() == tab && show_sidebar.get();
        let accent = match edition {
            CodiumEdition::Edu => theme.get().accent_edu,
            CodiumEdition::Ai => theme.get().accent_ai,
        };

        container(
            render_svg(svg_content, 18.0).style(move |s| {
                s.color(if is_active() {
                    theme.get().fg_primary
                } else {
                    theme.get().fg_muted
                })
            }),
        )
        .on_click_stop(move |_| {
            if active_tab.get() == tab && show_sidebar.get() {
                // If user clicks the currently active tab, collapse sidebar (VS Code behavior)
                show_sidebar.set(false);
            } else {
                active_tab.set(tab);
                show_sidebar.set(true);
            }
        })
        .style(move |s| {
            s.width(48.0)
                .height(44.0)
                .items_center()
                .justify_center()
                .cursor(CursorStyle::Pointer)
                .apply_if(is_active(), |s| {
                    s.border_left(2.0).border_color(accent)
                })
                .hover(|s| s.background(theme.get().hover_bg))
        })
    };

    v_stack((
        // Top Group: Primary Navigation
        v_stack((
            make_svg_btn(SidebarTab::Explorer, ICON_FILES),
            make_svg_btn(SidebarTab::Search, ICON_SEARCH),
            if edition == CodiumEdition::Edu {
                make_svg_btn(SidebarTab::DocTutor, ICON_GRADUATION)
            } else {
                make_svg_btn(SidebarTab::AiAssistant, ICON_SPARKLE)
            },
        ))
        .style(|s| s.flex_col().items_center()),

        // Bottom Group: Settings
        v_stack((
            make_svg_btn(SidebarTab::Settings, ICON_SETTINGS),
        ))
        .style(|s| s.flex_col().items_center().margin_bottom(8.0)),
    ))
    .style(move |s| {
        s.width(48.0)
            .height_pct(100.0)
            .justify_between()
            .background(theme.get().bg_activity)
            .border_right(1.0)
            .border_color(theme.get().border)
    })
}
