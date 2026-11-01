//! # Menubar View
//!
//! Provides a standard desktop IDE menu bar (Fájl/File, Szerkesztés/Edit, Nézet/View,
//! Eszközök/Tools, Súgó/Help) with hover-switchable menus and 2-box flex layout for labels and shortcuts.

use floem::action::{add_overlay, remove_overlay};
use floem::event::EventListener;
use floem::peniko::kurbo::{Point, Size};
use floem::prelude::*;
use floem::style::CursorStyle;
use floem::ViewId;
use std::path::PathBuf;
use crate::state::{ActiveMenu, AppState};
use codium_core::Command;

/// Standalone Menubar Component (placed directly below Titlebar)
pub fn menubar_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let locale = state.locale;

    h_stack((
        // Left: Menu Headers with hover-switchable dropdowns
        h_stack((
            menu_header_item(
                ActiveMenu::File,
                move || locale.get().tr("File"),
                state,
            ),
            menu_header_item(
                ActiveMenu::Edit,
                move || locale.get().tr("Edit"),
                state,
            ),
            menu_header_item(
                ActiveMenu::View,
                move || locale.get().tr("View"),
                state,
            ),
            menu_header_item(
                ActiveMenu::Tools,
                move || locale.get().tr("Tools"),
                state,
            ),
            menu_header_item(
                ActiveMenu::Help,
                move || locale.get().tr("Help"),
                state,
            ),
        ))
        .style(|s| s.flex_row().items_center()),

        // Right side subtle hints / status
        h_stack((
            dyn_container(
                move || state.status_message.get(),
                move |msg| {
                    if msg.is_empty() {
                        empty().into_any()
                    } else {
                        label(move || msg.clone()).style(move |s| {
                            s.font_size(11.0)
                                .color(theme.get().accent_edu)
                                .margin_right(12.0)
                        }).into_any()
                    }
                }
            ),
        ))
        .style(|s| s.flex_row().items_center()),
    ))
    .style(move |s| {
        s.height(26.0)
            .width_pct(100.0)
            .background(theme.get().bg_activity)
            .border_bottom(1.0)
            .border_color(theme.get().border)
            .items_center()
            .justify_between()
            .padding_horiz(8.0)
    })
}

/// Closes the currently active menubar dropdown overlay (if any)
pub fn close_active_menu(state: AppState) {
    if let Some(id) = state.active_menu_overlay.get_untracked() {
        remove_overlay(id);
        state.active_menu_overlay.set(None);
    }
    state.active_menu.set(None);
}

/// Opens a dropdown menu at a given screen position, removing any previously open overlay
pub fn open_menu(state: AppState, menu: ActiveMenu, pos: Point) {
    // 1. Remove previous overlay
    if let Some(id) = state.active_menu_overlay.get_untracked() {
        remove_overlay(id);
        state.active_menu_overlay.set(None);
    }

    // 2. Set active menu signal
    state.active_menu.set(Some(menu));

    // 3. Add new overlay with 2-box flex layout
    let overlay_id = add_overlay(pos, move |overlay_view_id| {
        match menu {
            ActiveMenu::File => create_file_dropdown(state, overlay_view_id).into_any(),
            ActiveMenu::Edit => create_edit_dropdown(state, overlay_view_id).into_any(),
            ActiveMenu::View => create_view_dropdown(state, overlay_view_id).into_any(),
            ActiveMenu::Tools => create_tools_dropdown(state, overlay_view_id).into_any(),
            ActiveMenu::Help => create_help_dropdown(state, overlay_view_id).into_any(),
        }
    });
    state.active_menu_overlay.set(Some(overlay_id));
}

/// Single top-level menu button in the menubar strip
fn menu_header_item(
    menu_kind: ActiveMenu,
    label_fn: impl Fn() -> String + 'static,
    state: AppState,
) -> impl IntoView {
    let theme = state.theme;
    let pos_signal = create_rw_signal(Point::ZERO);
    let size_signal = create_rw_signal(Size::ZERO);

    let default_x = match menu_kind {
        ActiveMenu::File => 8.0,
        ActiveMenu::Edit => 52.0,
        ActiveMenu::View => 98.0,
        ActiveMenu::Tools => 148.0,
        ActiveMenu::Help => 210.0,
    };

    let trigger_open = move || {
        let pos = pos_signal.get_untracked();
        let size = size_signal.get_untracked();
        let x = if pos.x > 0.0 { pos.x } else { default_x };
        let y = if size.height > 0.0 { pos.y + size.height } else { 58.0 };
        open_menu(state, menu_kind, Point::new(x, y));
    };

    let on_click = trigger_open.clone();
    let on_enter = trigger_open.clone();
    let on_move_hover = trigger_open.clone();
    let on_drag = trigger_open;

    container(
        label(move || label_fn()).style(move |s| {
            s.font_size(11.5)
                .color(theme.get().fg_primary)
                .selectable(false)
        }),
    )
    .on_event_stop(EventListener::PointerDown, move |_| {
        if state.active_menu.get_untracked() == Some(menu_kind) {
            close_active_menu(state);
        } else {
            on_click();
        }
    })
    .on_event_stop(EventListener::PointerEnter, move |_| {
        if let Some(active) = state.active_menu.get_untracked() {
            if active != menu_kind {
                on_enter();
            }
        }
    })
    .on_event_stop(EventListener::PointerMove, move |_| {
        if let Some(active) = state.active_menu.get_untracked() {
            if active != menu_kind {
                on_move_hover();
            }
        }
    })
    .on_event_stop(EventListener::DragEnter, move |_| {
        if let Some(active) = state.active_menu.get_untracked() {
            if active != menu_kind {
                on_drag();
            }
        }
    })
    .on_move(move |pt| pos_signal.set(pt))
    .on_resize(move |rect| size_signal.set(rect.size()))
    .style(move |s| {
        let is_active = state.active_menu.get() == Some(menu_kind);
        s.padding_horiz(8.0)
            .padding_vert(3.0)
            .border_radius(3.0)
            .cursor(CursorStyle::Pointer)
            .apply_if(is_active, move |s| {
                s.background(theme.get().hover_bg).color(theme.get().fg_primary)
            })
            .hover(move |s| s.background(theme.get().hover_bg).color(theme.get().fg_primary))
    })
}

/// Helper for a standard 2-box flex menu item row:
/// Helper for a standard 2-box flex menu item row:
/// [ Box 1: Label (Left) ] ------ [ Box 2: Shortcut (Right) ]
fn menu_row(
    state: AppState,
    _overlay_id: ViewId,
    title: String,
    shortcut: Option<String>,
    action: impl Fn() + 'static,
) -> impl IntoView {
    let theme = state.theme;

    h_stack((
        // Box 1 (Left): Command title
        label(move || title.clone()).style(move |s| {
            s.font_size(12.0)
                .color(theme.get().fg_primary)
                .selectable(false)
                .margin_right(24.0)
        }),

        // Box 2 (Right): Keyboard shortcut
        if let Some(sc) = shortcut {
            label(move || sc.clone()).style(move |s| {
                s.font_size(11.0)
                    .color(theme.get().fg_muted)
                    .selectable(false)
            }).into_any()
        } else {
            empty().into_any()
        },
    ))
    .on_click_stop(move |_| {
        close_active_menu(state);
        action();
    })
    .style(move |s| {
        s.width_pct(100.0)
            .min_width_pct(100.0)
            .height(28.0)
            .items_center()
            .justify_between()
            .padding_horiz(14.0)
            .cursor(CursorStyle::Pointer)
            .hover(move |s| s.background(theme.get().hover_bg).border_radius(4.0))
    })
}

/// Command menu row helper resolving shortcut automatically from keymap
fn cmd_row(state: AppState, overlay_id: ViewId, title: String, cmd: Command) -> impl IntoView {
    let sc = state.keymap.get().shortcut_for(cmd);
    menu_row(state, overlay_id, title, sc, move || {
        crate::keymap::execute_command(cmd, state);
    })
}

/// Horizontal divider separator between menu item groups
fn menu_separator(state: AppState) -> impl IntoView {
    let theme = state.theme;
    container(empty()).style(move |s| {
        s.height(1.0)
            .width_pct(100.0)
            .background(theme.get().border)
            .margin_vert(4.0)
            .margin_horiz(4.0)
    })
}

/// Wrapper styling for all dropdown menus: fixed min_width, themed border, dark shadow
fn dropdown_container<VT: ViewTuple + 'static>(state: AppState, children: VT) -> impl IntoView {
    let theme = state.theme;
    v_stack(children).style(move |s| {
        s.min_width(280.0)
            .background(theme.get().bg_sidebar)
            .border(1.0)
            .border_color(theme.get().border)
            .border_radius(6.0)
            .padding_vert(4.0)
            .padding_horiz(2.0)
            .box_shadow_blur(12.0)
            .box_shadow_color(Color::from_rgba8(0, 0, 0, 160))
            .z_index(9999)
    })
}

// 1. File Menu Dropdown
fn create_file_dropdown(state: AppState, overlay_id: ViewId) -> impl IntoView {
    let locale = state.locale.get();

    dropdown_container(
        state,
        (
            cmd_row(state, overlay_id, locale.tr("New File"), Command::NewFile),
            cmd_row(state, overlay_id, locale.tr("Open File..."), Command::OpenFile),
            cmd_row(state, overlay_id, locale.tr("Open Folder..."), Command::OpenFolder),
            menu_separator(state),
            cmd_row(state, overlay_id, locale.tr("Save"), Command::SaveFile),
            cmd_row(state, overlay_id, locale.tr("Close Tab"), Command::CloseActiveEditor),
            menu_separator(state),
            cmd_row(state, overlay_id, locale.tr("Exit"), Command::Quit),
        ),
    )
}

// 2. Edit Menu Dropdown
fn create_edit_dropdown(state: AppState, overlay_id: ViewId) -> impl IntoView {
    let locale = state.locale.get();

    dropdown_container(
        state,
        (
            cmd_row(state, overlay_id, locale.tr("Toggle Word Wrap"), Command::ToggleWordWrap),
            cmd_row(state, overlay_id, locale.tr("Find in Files"), Command::FindInFiles),
            menu_separator(state),
            cmd_row(state, overlay_id, locale.tr("Close All Tabs"), Command::CloseAllEditors),
        ),
    )
}

// 3. View Menu Dropdown
fn create_view_dropdown(state: AppState, overlay_id: ViewId) -> impl IntoView {
    let locale = state.locale.get();

    dropdown_container(
        state,
        (
            cmd_row(state, overlay_id, locale.tr("Toggle Sidebar"), Command::ToggleSidebar),
            cmd_row(state, overlay_id, locale.tr("Toggle Terminal Panel"), Command::ToggleTerminal),
            cmd_row(state, overlay_id, locale.tr("Toggle Helper / AI Panel"), Command::ToggleAuxiliaryBar),
            menu_separator(state),
            menu_row(
                state,
                overlay_id,
                locale.tr("Toggle Search in Header"),
                None,
                move || {
                    state.show_titlebar_search.update(|s| *s = !*s);
                },
            ),
        ),
    )
}

// 4. Tools Menu Dropdown
fn create_tools_dropdown(state: AppState, overlay_id: ViewId) -> impl IntoView {
    let locale = state.locale.get();

    dropdown_container(
        state,
        (
            cmd_row(state, overlay_id, locale.tr("Run Project"), Command::RunProject),
            cmd_row(state, overlay_id, locale.tr("Search Everywhere"), Command::SearchEverywhere),
            menu_separator(state),
            menu_row(
                state,
                overlay_id,
                format!("{} (HU / EN)", locale.tr("Language Switcher")),
                None,
                move || {
                    state.locale.update(|loc| *loc = loc.toggle());
                },
            ),
        ),
    )
}

// 5. Help Menu Dropdown
fn create_help_dropdown(state: AppState, overlay_id: ViewId) -> impl IntoView {
    let locale = state.locale.get();

    dropdown_container(
        state,
        (
            cmd_row(state, overlay_id, locale.tr("Documentation / Manuals"), Command::OpenDocumentation),
            menu_row(
                state,
                overlay_id,
                locale.tr("License: AGPL-3.0-or-later"),
                None,
                move || {
                    state.status_message.set("GNU AGPL-3.0-or-later | Commercial Dual License".to_string());
                },
            ),
            menu_separator(state),
            menu_row(
                state,
                overlay_id,
                locale.tr("About BM::Codium"),
                None,
                move || {
                    let msg = format!("BM::Codium v0.0.0 (Pre-Canary-0) • University & AI Edition");
                    state.status_message.set(msg);
                },
            ),
        ),
    )
}

fn pick_file_dialog() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.Title = "Open File in BM::Codium"
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.FileName
}
"#;
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = PathBuf::from(s);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = std::process::Command::new("zenity")
            .args(["--file-selection", "--title=Open File in BM::Codium"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = PathBuf::from(s);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }

        if let Ok(output) = std::process::Command::new("kdialog")
            .args(["--getopenfilename", "."])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = PathBuf::from(s);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Native file picker dialog using zenity/kdialog/powershell fallback
pub fn trigger_open_file(state: AppState) {
    let ws = state.workspace;
    let status = state.status_message;
    let locale = state.locale;

    let send = floem::ext_event::create_ext_action(
        floem::reactive::Scope::new(),
        move |path: Option<PathBuf>| {
            if let Some(path) = path {
                if path.is_file() {
                    let path_display = path.display().to_string();
                    ws.update(|w| {
                        let _ = w.open_file(path);
                    });
                    let tr_open = locale.get_untracked().tr("Opened file");
                    status.set(format!("{}: {}", tr_open, path_display));
                }
            }
        },
    );

    std::thread::spawn(move || {
        let p = pick_file_dialog();
        send(p);
    });
}
