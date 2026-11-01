use floem::event::{Event, EventListener, EventPropagation};
use floem::prelude::*;
use crate::state::AppState;
use super::titlebar::titlebar_view;
use super::menubar::menubar_view;
use super::activity_bar::activity_bar_view;
use super::sidebar::sidebar_view;
use super::editor_view::editor_view;
use super::bottom_panel::bottom_panel_view;
use super::right_panel::right_panel_view;
use super::splitter::{bottom_panel_splitter, left_sidebar_splitter, right_panel_splitter};
use super::statusbar::statusbar_view;

pub fn root_app_view(state: AppState) -> impl IntoView {
    v_stack((
        // Top Titlebar
        titlebar_view(state),

        // Desktop IDE Menubar Strip (Fájl, Szerkesztés, Nézet, Eszközök, Súgó)
        menubar_view(state),

        // Main Center Area (Activity Bar + Left Sidebar + Splitter + Editor/Bottom Panel + Splitter + Right Manuals Panel)
        h_stack((
            activity_bar_view(state),
            sidebar_view(state),
            left_sidebar_splitter(state),
            v_stack((
                editor_view(state),
                bottom_panel_splitter(state),
                bottom_panel_view(state),
            ))
            .style(|s| {
                s.flex_col()
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
                    .flex_basis(0.0)
                    .min_width(0.0)
                    .height_pct(100.0)
            }),
            right_panel_splitter(state),
            right_panel_view(state),
        ))
        .style(|s| s.flex_row().flex_grow(1.0).width_pct(100.0)),

        // Bottom Statusbar
        statusbar_view(state),
    ))
    .keyboard_navigable()
    .on_event_cont(EventListener::PointerDown, move |_| {
        if state.active_menu.get_untracked().is_some() {
            super::menubar::close_active_menu(state);
        }
    })
    .on_event(EventListener::KeyDown, move |event| {
        if let Event::KeyDown(ke) = event {
            if ke.key.logical_key == floem::keyboard::Key::Named(floem::keyboard::NamedKey::Escape) {
                if state.active_menu.get_untracked().is_some() {
                    super::menubar::close_active_menu(state);
                    return EventPropagation::Stop;
                }
            }
        }
        let keymap = state.keymap.get();
        if let Some(cmd) = crate::keymap::match_keymap(event, &keymap) {
            crate::keymap::execute_command(cmd, state);
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    })
    .on_event_stop(EventListener::WindowResized, move |event| {
        if let Event::WindowResized(size) = event {
            if size.width > 300.0 && size.height > 200.0 {
                state.settings.update(|s| {
                    s.window_width = size.width;
                    s.window_height = size.height;
                });
                let _ = state.settings.get_untracked().save();
            }
        }
    })
    .style(move |s| {
        s.flex_col()
            .size_pct(100.0, 100.0)
            .font_family(state.settings.get().ui_font_family)
    })
}
