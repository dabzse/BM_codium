//! # Resizable Splitters
//!
//! Interactive splitter bars for resizing:
//! - Left primary sidebar (width)
//! - Bottom dock panel (height)
//! - Right secondary documentation / tutor panel (width)
//!
//! Supports click-and-drag resizing, double-click to reset to default size,
//! hover/active highlight styling, and layout persistence to settings.json.

use floem::prelude::*;
use floem::event::{Event, EventListener};
use floem::kurbo::Point;
use floem::style::CursorStyle;
use floem::taffy::style::Display;
use crate::state::AppState;

pub const DEFAULT_SIDEBAR_WIDTH: f64 = 260.0;
pub const MIN_SIDEBAR_WIDTH: f64 = 160.0;
pub const MAX_SIDEBAR_WIDTH: f64 = 700.0;

pub const DEFAULT_BOTTOM_PANEL_HEIGHT: f64 = 180.0;
pub const MIN_BOTTOM_PANEL_HEIGHT: f64 = 80.0;
pub const MAX_BOTTOM_PANEL_HEIGHT: f64 = 600.0;

pub const DEFAULT_RIGHT_PANEL_WIDTH: f64 = 280.0;
pub const MIN_RIGHT_PANEL_WIDTH: f64 = 180.0;
pub const MAX_RIGHT_PANEL_WIDTH: f64 = 700.0;

/// Vertical splitter bar for resizing the Left Sidebar horizontally
pub fn left_sidebar_splitter(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_sidebar = state.show_sidebar;
    let drag_start: RwSignal<Option<Point>> = create_rw_signal(None);

    let view = empty();
    let view_id = view.id();

    view.on_event_stop(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(pointer_event) = event {
            if pointer_event.count == 2 {
                state.sidebar_width.set(DEFAULT_SIDEBAR_WIDTH);
                state.settings.update(|s| s.sidebar_width = DEFAULT_SIDEBAR_WIDTH);
                let _ = state.settings.get_untracked().save();
                drag_start.set(None);
                return;
            }
            view_id.request_active();
            drag_start.set(Some(pointer_event.pos));
        }
    })
    .on_event_stop(EventListener::PointerMove, move |event| {
        if let Event::PointerMove(pointer_event) = event {
            if let Some(drag_start_point) = drag_start.get_untracked() {
                let current_w = state.sidebar_width.get_untracked();
                let delta = pointer_event.pos.x - drag_start_point.x;
                let new_w = (current_w + delta).clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH);
                if (new_w - current_w).abs() > 0.1 {
                    state.sidebar_width.set(new_w);
                }
            }
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        if drag_start.get_untracked().is_some() {
            view_id.clear_active();
            drag_start.set(None);
            let final_w = state.sidebar_width.get_untracked();
            state.settings.update(|s| s.sidebar_width = final_w);
            let _ = state.settings.get_untracked().save();
        }
    })
    .style(move |s| {
        let is_dragging = drag_start.get().is_some();
        s.width(4.0)
            .height_pct(100.0)
            .cursor(CursorStyle::ColResize)
            .background(if is_dragging {
                theme.get().accent_blue
            } else {
                theme.get().border
            })
            .hover(move |s| {
                s.background(theme.get().accent_blue)
            })
            .display(if show_sidebar.get() { Display::Flex } else { Display::None })
    })
}

/// Horizontal splitter bar for resizing the Bottom Dock Panel vertically
pub fn bottom_panel_splitter(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_panel = state.show_bottom_panel;
    let drag_start: RwSignal<Option<Point>> = create_rw_signal(None);

    let view = empty();
    let view_id = view.id();

    view.on_event_stop(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(pointer_event) = event {
            if pointer_event.count == 2 {
                state.bottom_panel_height.set(DEFAULT_BOTTOM_PANEL_HEIGHT);
                state.settings.update(|s| s.bottom_panel_height = DEFAULT_BOTTOM_PANEL_HEIGHT);
                let _ = state.settings.get_untracked().save();
                drag_start.set(None);
                return;
            }
            view_id.request_active();
            drag_start.set(Some(pointer_event.pos));
        }
    })
    .on_event_stop(EventListener::PointerMove, move |event| {
        if let Event::PointerMove(pointer_event) = event {
            if let Some(drag_start_point) = drag_start.get_untracked() {
                let current_h = state.bottom_panel_height.get_untracked();
                let delta = drag_start_point.y - pointer_event.pos.y;
                let new_h = (current_h + delta).clamp(MIN_BOTTOM_PANEL_HEIGHT, MAX_BOTTOM_PANEL_HEIGHT);
                if (new_h - current_h).abs() > 0.1 {
                    state.bottom_panel_height.set(new_h);
                }
            }
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        if drag_start.get_untracked().is_some() {
            view_id.clear_active();
            drag_start.set(None);
            let final_h = state.bottom_panel_height.get_untracked();
            state.settings.update(|s| s.bottom_panel_height = final_h);
            let _ = state.settings.get_untracked().save();
        }
    })
    .style(move |s| {
        let is_dragging = drag_start.get().is_some();
        s.width_pct(100.0)
            .height(4.0)
            .cursor(CursorStyle::RowResize)
            .background(if is_dragging {
                theme.get().accent_blue
            } else {
                theme.get().border
            })
            .hover(move |s| {
                s.background(theme.get().accent_blue)
            })
            .display(if show_panel.get() { Display::Flex } else { Display::None })
    })
}

/// Vertical splitter bar for resizing the Right Secondary Panel horizontally
pub fn right_panel_splitter(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_panel = state.show_right_panel;
    let drag_start: RwSignal<Option<Point>> = create_rw_signal(None);

    let view = empty();
    let view_id = view.id();

    view.on_event_stop(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(pointer_event) = event {
            if pointer_event.count == 2 {
                state.right_panel_width.set(DEFAULT_RIGHT_PANEL_WIDTH);
                state.settings.update(|s| s.right_panel_width = DEFAULT_RIGHT_PANEL_WIDTH);
                let _ = state.settings.get_untracked().save();
                drag_start.set(None);
                return;
            }
            view_id.request_active();
            drag_start.set(Some(pointer_event.pos));
        }
    })
    .on_event_stop(EventListener::PointerMove, move |event| {
        if let Event::PointerMove(pointer_event) = event {
            if let Some(drag_start_point) = drag_start.get_untracked() {
                let current_w = state.right_panel_width.get_untracked();
                let delta = drag_start_point.x - pointer_event.pos.x;
                let new_w = (current_w + delta).clamp(MIN_RIGHT_PANEL_WIDTH, MAX_RIGHT_PANEL_WIDTH);
                if (new_w - current_w).abs() > 0.1 {
                    state.right_panel_width.set(new_w);
                }
            }
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        if drag_start.get_untracked().is_some() {
            view_id.clear_active();
            drag_start.set(None);
            let final_w = state.right_panel_width.get_untracked();
            state.settings.update(|s| s.right_panel_width = final_w);
            let _ = state.settings.get_untracked().save();
        }
    })
    .style(move |s| {
        let is_dragging = drag_start.get().is_some();
        s.width(4.0)
            .height_pct(100.0)
            .cursor(CursorStyle::ColResize)
            .background(if is_dragging {
                theme.get().accent_blue
            } else {
                theme.get().border
            })
            .hover(move |s| {
                s.background(theme.get().accent_blue)
            })
            .display(if show_panel.get() { Display::Flex } else { Display::None })
    })
}
