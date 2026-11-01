//! # Primary Sidebar View
//!
//! Renders views based on the selected Activity Bar icon:
//! - File Explorer (collapsible directory tree, file icons, tab opener)
//! - Search in Files
//! - EDU Doc Tutor (pedagogic cheat sheets and language cards)
//! - AI Assistant (local model chat and suggestions)
//! - Settings & Policy Management

use floem::prelude::*;
use floem::peniko::Brush;
use floem::style::CursorStyle;
use floem::text::Weight;
use floem::taffy::style::Display;
use codium_core::FileNode;
use codium_edu::DocTutor;
use crate::state::{AppState, SidebarTab};

/// Root sidebar container with dynamic switching between views
pub fn sidebar_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_sidebar = state.show_sidebar;
    let active_tab = state.active_sidebar_tab;

    dyn_container(
        move || (active_tab.get(), show_sidebar.get()),
        move |(tab, is_shown)| {
            if !is_shown {
                return container(empty()).style(|s| s.display(Display::None)).into_any();
            }

            match tab {
                SidebarTab::Explorer => explorer_view(state).into_any(),
                SidebarTab::Search => search_view(state).into_any(),
                SidebarTab::DocTutor => doc_tutor_view(state).into_any(),
                SidebarTab::AiAssistant => ai_assistant_view(state).into_any(),
                SidebarTab::Settings => settings_view(state).into_any(),
            }
        },
    )
    .style(move |s| {
        s.width(state.sidebar_width.get())
            .height_pct(100.0)
            .background(theme.get().bg_sidebar)
            .display(if show_sidebar.get() { Display::Flex } else { Display::None })
    })
}

/// Native folder picker helper
pub fn pick_folder_dialog(initial: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = "Select Folder"
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.SelectedPath
}
"#;
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = std::path::PathBuf::from(s);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = std::process::Command::new("zenity");
        cmd.arg("--file-selection").arg("--directory").arg("--title=Select Folder");
        if let Some(init) = initial {
            cmd.arg(format!("--filename={}", init.display()));
        }
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = std::path::PathBuf::from(s);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }

        let mut kcmd = std::process::Command::new("kdialog");
        kcmd.arg("--getexistingdirectory");
        if let Some(init) = initial {
            kcmd.arg(init);
        }
        if let Ok(output) = kcmd.output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    let p = std::path::PathBuf::from(s);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Triggers folder open via native picker in background thread and updates workspace state
pub fn trigger_open_folder(state: AppState) {
    let ws = state.workspace;
    let settings = state.settings;
    let status = state.status_message;
    let locale = state.locale;

    let send = floem::ext_event::create_ext_action(
        floem::reactive::Scope::new(),
        move |path: Option<std::path::PathBuf>| {
            if let Some(path) = path {
                if path.is_dir() {
                    let path_str = path.to_string_lossy().to_string();
                    ws.update(|w| {
                        let _ = w.open_folder(path.clone());
                    });
                    settings.update(|s| {
                        s.last_workspace_folder = Some(path_str.clone());
                    });
                    let _ = settings.get_untracked().save();
                    status.set(format!("{}: {}", locale.get().tr("Opened folder"), path_str));
                }
            }
        },
    );

    let initial = ws.get_untracked().root_path;
    std::thread::spawn(move || {
        let picked = pick_folder_dialog(initial.as_deref());
        send(picked);
    });
}

/// File Explorer view rendering folder structure
fn explorer_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let ws = state.workspace;
    let locale = state.locale;

    v_stack((
        // Explorer Header
        h_stack((
            label(move || locale.get().explorer_title()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
            }),
            label(move || {
                ws.get().root_path
                    .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| locale.get().tr("NO FOLDER OPEN"))
                    .to_uppercase()
            }).style(move |s| {
                s.font_size(10.5)
                    .color(theme.get().fg_muted)
                    .margin_left(8.0)
            }),
            empty().style(|s| s.flex_grow(1.0)),
            // Folder open button in header
            h_stack((
                label(move || "📁 ").style(|s| s.font_size(11.0)),
                label(move || locale.get().tr("Open Folder")).style(move |s| {
                    s.font_size(10.5)
                        .color(theme.get().fg_primary)
                }),
            ))
            .on_click_stop(move |_| {
                trigger_open_folder(state);
            })
            .style(move |s| {
                s.items_center()
                    .padding_horiz(6.0)
                    .height(22.0)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(3.0)
                    .cursor(CursorStyle::Pointer)
                    .hover(move |s| s.background(theme.get().hover_bg).border_color(theme.get().accent_blue))
            }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(32.0)
                .padding_horiz(12.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(theme.get().border)
        }),

        // Interactive File Tree or Welcome "No Folder Open" View
        dyn_container(
            move || ws.get().root_path.is_some() && !ws.get().file_tree.is_empty(),
            move |has_folder| {
                if !has_folder {
                    let folder_input = create_rw_signal(String::new());
                    return v_stack((
                        label(move || "📁").style(|s| s.font_size(32.0).margin_bottom(8.0)),
                        label(move || locale.get().tr("You have not yet opened a folder.")).style(move |s| {
                            s.font_size(11.5)
                                .color(theme.get().fg_muted)
                                .margin_bottom(12.0)
                        }),
                        container(
                            label(move || locale.get().tr("Open Folder")).style(move |s| {
                                s.font_size(12.0)
                                    .font_weight(Weight::BOLD)
                                    .color(palette::css::WHITE)
                            }),
                        )
                        .on_click_stop(move |_| {
                            trigger_open_folder(state);
                        })
                        .style(move |s| {
                            s.background(theme.get().accent_blue)
                                .padding_horiz(14.0)
                                .height(28.0)
                                .items_center()
                                .border_radius(4.0)
                                .cursor(CursorStyle::Pointer)
                                .hover(move |s| s.background(theme.get().accent_ai))
                        }),
                        // Path text input row
                        h_stack((
                            text_input(folder_input)
                                .on_enter(move || {
                                    let p = folder_input.get().trim().to_string();
                                    if !p.is_empty() {
                                        let path = std::path::PathBuf::from(&p);
                                        if path.is_dir() {
                                            state.workspace.update(|w| { let _ = w.open_folder(path); });
                                            state.settings.update(|s| s.last_workspace_folder = Some(p));
                                            let _ = state.settings.get_untracked().save();
                                        }
                                    }
                                })
                                .style(move |s| {
                                    s.width_pct(100.0)
                                        .height(26.0)
                                        .background(theme.get().bg_editor)
                                        .border(1.0)
                                        .border_color(theme.get().border)
                                        .border_radius(3.0)
                                        .padding_horiz(6.0)
                                        .color(theme.get().fg_primary)
                                        .cursor_color(Brush::Solid(palette::css::WHITE))
                                        .font_size(11.0)
                                }),
                            container(
                                label(move || locale.get().tr("Open")).style(move |s| {
                                    s.font_size(10.5)
                                        .color(palette::css::WHITE)
                                }),
                            )
                            .on_click_stop(move |_| {
                                let p = folder_input.get().trim().to_string();
                                if !p.is_empty() {
                                    let path = std::path::PathBuf::from(&p);
                                    if path.is_dir() {
                                        state.workspace.update(|w| { let _ = w.open_folder(path); });
                                        state.settings.update(|s| s.last_workspace_folder = Some(p));
                                        let _ = state.settings.get_untracked().save();
                                    }
                                }
                            })
                            .style(move |s| {
                                s.background(theme.get().accent_edu)
                                    .padding_horiz(8.0)
                                    .height(26.0)
                                    .items_center()
                                    .border_radius(3.0)
                                    .margin_left(4.0)
                                    .cursor(CursorStyle::Pointer)
                            }),
                        ))
                        .style(|s| s.items_center().width_pct(100.0).margin_top(14.0).padding_horiz(12.0)),
                    ))
                    .style(move |s| {
                        s.flex_col()
                            .items_center()
                            .justify_center()
                            .padding(16.0)
                            .width_pct(100.0)
                    })
                    .into_any();
                }

                scroll(
                    dyn_container(
                        move || ws.get().file_tree.clone(),
                        move |tree| {
                            let mut items = Vec::new();
                            for node in tree {
                                render_file_node(&node, 0, state, &mut items);
                            }
                            v_stack_from_iter(items).style(|s| s.flex_col().width_pct(100.0))
                        },
                    )
                    .style(|s| s.width_pct(100.0)),
                )
                .style(|s| s.flex_grow(1.0).width_pct(100.0))
                .into_any()
            },
        )
        .style(|s| s.flex_grow(1.0).width_pct(100.0)),
    ))
    .style(|s| s.flex_col().size_pct(100.0, 100.0))
}

/// Recursively builds file tree rows
fn render_file_node(node: &FileNode, depth: usize, state: AppState, items: &mut Vec<Box<dyn View>>) {
    let path = node.path.clone();
    let is_dir = node.is_dir;
    let is_expanded = node.is_expanded;
    let name = node.name.clone();
    let theme = state.theme;
    let ws = state.workspace;

    let indent = (depth as f64) * 14.0 + 10.0;
    let icon = if is_dir {
        if is_expanded { "📂 " } else { "📁 " }
    } else {
        if name.ends_with(".rs") { "🦀 " }
        else if name.ends_with(".c") || name.ends_with(".h") { "🇨 " }
        else if name.ends_with(".cpp") || name.ends_with(".hpp") || name.ends_with(".cc") { "➕ " }
        else if name.ends_with(".cs") { "🔷 " }
        else if name.ends_with(".py") || name.ends_with(".pyw") { "🐍 " }
        else if name.ends_with(".php") || name.ends_with(".phtml") { "🐘 " }
        else if name.ends_with(".java") { "☕ " }
        else if name.ends_with(".kt") || name.ends_with(".kts") { "🎯 " }
        else if name.ends_with(".erl") || name.ends_with(".hrl") { "🔴 " }
        else if name.ends_with(".scala") || name.ends_with(".sc") { "🔴 " }
        else if name.ends_with(".ex") || name.ends_with(".exs") { "💧 " }
        else if name.ends_with(".go") { "🐹 " }
        else if name.ends_with(".ts") || name.ends_with(".tsx") { "🔷 " }
        else if name.ends_with(".js") || name.ends_with(".mjs") { "🟨 " }
        else if name.ends_with(".rb") || name.ends_with(".rake") { "💎 " }
        else if name.ends_with(".zig") { "⚡ " }
        else if name.ends_with(".swift") { "🕊️ " }
        else if name.ends_with(".m") || name.ends_with(".mm") { "🍏 " }
        else if name.ends_with(".html") || name.ends_with(".htm") { "🌐 " }
        else if name.ends_with(".css") || name.ends_with(".scss") { "🎨 " }
        else if name.ends_with(".md") { "📝 " }
        else if name.ends_with(".sh") || name.ends_with(".bash") || name.ends_with(".ps1") { "🐚 " }
        else if name.ends_with(".sql") { "🗄️ " }
        else if name.ends_with(".toml") || name.ends_with(".json") { "⚙️ " }
        else { "📄 " }
    };

    let item_view = h_stack((
        label(move || icon).style(|s| s.font_size(12.0)),
        label(move || name.clone()).style(move |s| {
            s.font_size(12.0)
                .color(theme.get().fg_primary)
                .margin_left(4.0)
        }),
    ))
    .on_click_stop(move |_| {
        if is_dir {
            ws.update(|w| {
                toggle_in_tree(&mut w.file_tree, &path);
            });
        } else {
            ws.update(|w| {
                let _ = w.open_file(path.clone());
            });
        }
    })
    .style(move |s| {
        s.height(24.0)
            .width_pct(100.0)
            .items_center()
            .padding_left(indent)
            .cursor(CursorStyle::Pointer)
            .hover(|s| s.background(theme.get().hover_bg))
    });

    items.push(Box::new(item_view));

    if is_dir && is_expanded {
        for child in &node.children {
            render_file_node(child, depth + 1, state, items);
        }
    }
}

fn toggle_in_tree(tree: &mut [FileNode], target: &std::path::Path) -> bool {
    for node in tree.iter_mut() {
        if node.path == target {
            node.toggle_expanded();
            return true;
        }
        if toggle_in_tree(&mut node.children, target) {
            return true;
        }
    }
    false
}

/// Workspace Search & Replace panel (VS Code style)
fn search_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let query = state.search_query;
    let replace_q = state.replace_query;
    let show_replace = state.show_replace_row;
    let results = state.search_results;
    let active_match_idx = state.active_search_match_idx;
    let ws = state.workspace;
    let locale = state.locale;

    v_stack((
        // Title Bar with Prominent Replace Toggle
        h_stack((
            label(move || locale.get().search_title()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
            }),
            // High-visibility Replace toggle button
            container(
                h_stack((
                    label(move || if show_replace.get() { "▼ " } else { "▶ " }).style(move |s| {
                        s.font_size(10.0).color(theme.get().accent_blue)
                    }),
                    label(move || locale.get().tr("Replace")).style(move |s| {
                        s.font_size(11.0)
                            .font_weight(Weight::BOLD)
                            .color(if show_replace.get() { theme.get().accent_blue } else { theme.get().fg_primary })
                    }),
                ))
            )
            .on_click_stop(move |_| {
                show_replace.update(|v| *v = !*v);
            })
            .style(move |s| {
                s.padding_horiz(8.0)
                    .height(24.0)
                    .items_center()
                    .border_radius(4.0)
                    .background(if show_replace.get() { theme.get().bg_editor } else { palette::css::TRANSPARENT })
                    .border(1.0)
                    .border_color(if show_replace.get() { theme.get().accent_blue } else { theme.get().border })
                    .cursor(CursorStyle::Pointer)
                    .hover(move |s| s.background(theme.get().hover_bg))
            }),
        ))
        .style(move |s| {
            s.height(34.0).padding_horiz(12.0).items_center().justify_between()
                .border_bottom(1.0).border_color(theme.get().border)
        }),

        // Inputs Container
        v_stack((
            // Search Input Row
            h_stack((
                text_input(query)
                    .on_enter(move || {
                        let q = query.get();
                        let current_ws = ws.get();
                        let matches = perform_workspace_search(&current_ws, &q);
                        let len = matches.len();
                        results.set(matches);
                        active_match_idx.set(if len > 0 { Some(0) } else { None });
                    })
                    .style(move |s| {
                        s.flex_grow(1.0)
                            .height(26.0)
                            .background(theme.get().bg_editor)
                            .border(1.0)
                            .border_color(theme.get().border)
                            .border_radius(4.0)
                            .padding_horiz(8.0)
                            .color(theme.get().fg_primary)
                            .cursor_color(Brush::Solid(palette::css::WHITE))
                            .outline(0.0)
                            .outline_color(palette::css::TRANSPARENT)
                            .hover(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                            .focus(move |s| {
                                s.background(theme.get().bg_editor)
                                    .border_color(theme.get().accent_blue)
                                    .outline(0.0)
                                    .hover(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                            })
                            .active(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                    }),
                container(
                    label(move || locale.get().tr("Search")).style(move |s| {
                        s.font_size(11.0).font_weight(Weight::BOLD).color(palette::css::WHITE)
                    }),
                )
                .on_click_stop(move |_| {
                    let q = query.get();
                    let current_ws = ws.get();
                    let matches = perform_workspace_search(&current_ws, &q);
                    let len = matches.len();
                    results.set(matches);
                    active_match_idx.set(if len > 0 { Some(0) } else { None });
                })
                .style(move |s| {
                    s.background(theme.get().accent_blue)
                        .padding_horiz(10.0)
                        .height(26.0)
                        .margin_left(6.0)
                        .items_center()
                        .border_radius(4.0)
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().accent_ai))
                }),
            ))
            .style(|s| s.width_pct(100.0).margin_top(8.0).margin_bottom(6.0)),

            // Replace Input & Action Buttons Row (collapsible)
            dyn_container(
                move || show_replace.get(),
                move |is_shown| {
                    if !is_shown {
                        return empty().into_any();
                    }
                    v_stack((
                        // Replace Input
                        text_input(replace_q).style(move |s| {
                            s.width_pct(100.0)
                                .height(26.0)
                                .background(theme.get().bg_editor)
                                .border(1.0)
                                .border_color(theme.get().border)
                                .border_radius(4.0)
                                .padding_horiz(8.0)
                                .color(theme.get().fg_primary)
                                .cursor_color(Brush::Solid(palette::css::WHITE))
                                .outline(0.0)
                                .outline_color(palette::css::TRANSPARENT)
                                .hover(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                                .focus(move |s| {
                                    s.background(theme.get().bg_editor)
                                        .border_color(theme.get().accent_blue)
                                        .outline(0.0)
                                        .hover(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                                })
                                .active(move |s| s.background(theme.get().bg_editor).border_color(theme.get().accent_blue))
                        }),

                        // Action Buttons: Prev, Next, Replace, Replace All
                        h_stack((
                            // Previous occurrence (↑)
                            container(
                                label(move || locale.get().tr("↑ Prev")).style(move |s| {
                                    s.font_size(10.5).color(theme.get().fg_primary)
                                }),
                            )
                            .on_click_stop(move |_| {
                                let matches = results.get();
                                if matches.is_empty() { return; }
                                let len = matches.len();
                                let new_idx = match active_match_idx.get() {
                                    Some(curr) => (curr + len - 1) % len,
                                    None => len - 1,
                                };
                                active_match_idx.set(Some(new_idx));
                                let target_path = matches[new_idx].file_path.clone();
                                ws.update(|w| { let _ = w.open_file(target_path); });
                            })
                            .style(move |s| {
                                s.background(theme.get().bg_editor)
                                    .border(1.0).border_color(theme.get().border)
                                    .padding_horiz(6.0).height(24.0)
                                    .items_center().border_radius(4.0)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().hover_bg))
                            }),

                            // Next occurrence (↓)
                            container(
                                label(move || locale.get().tr("↓ Next")).style(move |s| {
                                    s.font_size(10.5).color(theme.get().fg_primary)
                                }),
                            )
                            .on_click_stop(move |_| {
                                let matches = results.get();
                                if matches.is_empty() { return; }
                                let len = matches.len();
                                let new_idx = match active_match_idx.get() {
                                    Some(curr) => (curr + 1) % len,
                                    None => 0,
                                };
                                active_match_idx.set(Some(new_idx));
                                let target_path = matches[new_idx].file_path.clone();
                                ws.update(|w| { let _ = w.open_file(target_path); });
                            })
                            .style(move |s| {
                                s.background(theme.get().bg_editor)
                                    .border(1.0).border_color(theme.get().border)
                                    .padding_horiz(6.0).height(24.0).margin_left(4.0)
                                    .items_center().border_radius(4.0)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().hover_bg))
                            }),

                            // Replace single active occurrence
                            container(
                                label(move || locale.get().tr("Replace")).style(move |s| {
                                    s.font_size(10.5).font_weight(Weight::BOLD).color(palette::css::WHITE)
                                }),
                            )
                            .on_click_stop(move |_| {
                                let matches = results.get();
                                if matches.is_empty() { return; }
                                let curr_idx = active_match_idx.get().unwrap_or(0).min(matches.len() - 1);
                                let target_m = matches[curr_idx].clone();
                                let s_str = query.get();
                                let r_str = replace_q.get();
                                let mut current_ws = ws.get();
                                let success = perform_replace_single_occurrence(&mut current_ws, &s_str, &r_str, &target_m);
                                if success {
                                    ws.set(current_ws);
                                    state.status_message.set(locale.get().tr("1 occurrence replaced."));
                                    let current_ws2 = ws.get();
                                    let new_matches = perform_workspace_search(&current_ws2, &s_str);
                                    let new_len = new_matches.len();
                                    results.set(new_matches);
                                    if new_len > 0 {
                                        active_match_idx.set(Some(curr_idx.min(new_len - 1)));
                                    } else {
                                        active_match_idx.set(None);
                                    }
                                }
                            })
                            .style(move |s| {
                                s.background(theme.get().accent_blue)
                                    .padding_horiz(7.0).height(24.0).margin_left(4.0)
                                    .items_center().border_radius(4.0)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().accent_ai))
                            }),

                            // Replace All
                            container(
                                label(move || locale.get().tr("Replace All")).style(move |s| {
                                    s.font_size(10.5).font_weight(Weight::BOLD).color(palette::css::WHITE)
                                }),
                            )
                            .on_click_stop(move |_| {
                                let s_str = query.get();
                                let r_str = replace_q.get();
                                let mut current_ws = ws.get();
                                let (count, files) = perform_workspace_replace_all(&mut current_ws, &s_str, &r_str);
                                ws.set(current_ws);
                                state.status_message.set(format!("{count} occurrences replaced across {files} files."));
                                let current_ws2 = ws.get();
                                let matches = perform_workspace_search(&current_ws2, &s_str);
                                results.set(matches);
                                active_match_idx.set(None);
                            })
                            .style(move |s| {
                                s.background(theme.get().accent_edu)
                                    .padding_horiz(7.0).height(24.0).margin_left(4.0)
                                    .items_center().border_radius(4.0)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().accent_blue))
                            }),
                        ))
                        .style(|s| s.flex_row().items_center().margin_top(6.0).margin_bottom(8.0)),
                    ))
                    .style(|s| s.flex_col().width_pct(100.0))
                    .into_any()
                },
            ),
        ))
        .style(|s| s.flex_col().padding_horiz(10.0)),

        // Results Header
        label(move || {
            let res = results.get();
            if res.is_empty() {
                if query.get().is_empty() {
                    locale.get().tr("Type search expression...")
                } else {
                    locale.get().tr("No results found in workspace.")
                }
            } else {
                format!("{} {}", res.len(), locale.get().tr("results found:"))
            }
        })
        .style(move |s| {
            s.font_size(11.0).color(theme.get().fg_muted).margin_left(12.0).margin_bottom(6.0)
        }),

        // Results List with Working Scrolling
        scroll(
            dyn_container(
                move || (results.get(), active_match_idx.get()),
                move |(matches, active_idx)| {
                    v_stack_from_iter(
                        matches.into_iter().enumerate().map(|(idx, m)| {
                            let is_active = active_idx == Some(idx);
                            let file_str = m.file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let line_num = m.line_number;
                            let line_text = m.line_content.clone();
                            let target_path = m.file_path.clone();

                            h_stack((
                                label(move || format!("{file_str}:{line_num}")).style(move |s| {
                                    s.font_size(11.0)
                                        .font_weight(Weight::BOLD)
                                        .color(if is_active { theme.get().accent_blue } else { theme.get().fg_primary })
                                        .width(90.0)
                                }),
                                label(move || line_text.clone()).style(move |s| {
                                    s.font_size(11.0)
                                        .color(if is_active { theme.get().fg_primary } else { theme.get().fg_muted })
                                        .margin_left(4.0)
                                }),
                            ))
                            .on_click_stop(move |_| {
                                active_match_idx.set(Some(idx));
                                ws.update(|w| {
                                    let _ = w.open_file(target_path.clone());
                                });
                            })
                            .style(move |s| {
                                s.padding(5.0)
                                    .border_radius(4.0)
                                    .cursor(CursorStyle::Pointer)
                                    .background(if is_active { theme.get().hover_bg } else { palette::css::TRANSPARENT })
                                    .border_left(if is_active { 3.0 } else { 0.0 })
                                    .border_color(theme.get().accent_blue)
                                    .hover(move |s| s.background(theme.get().hover_bg))
                            })
                        })
                    )
                    .style(|s| s.flex_col().width_pct(100.0))
                },
            )
            .style(|s| s.width_pct(100.0))
        )
        .style(|s| {
            s.flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .width_pct(100.0)
                .padding_horiz(10.0)
        }),
    ))
    .style(|s| s.flex_col().size_pct(100.0, 100.0))
}

fn perform_workspace_search(ws: &codium_core::Workspace, query: &str) -> Vec<crate::state::FileSearchMatch> {
    let mut matches = Vec::new();
    let q = query.trim();
    if q.is_empty() {
        return matches;
    }

    // 1. Check open documents first
    for doc in &ws.open_documents {
        let content = doc.text();
        for (line_idx, line) in content.lines().enumerate() {
            if line.contains(q) {
                matches.push(crate::state::FileSearchMatch {
                    file_path: doc.path().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from(doc.file_name())),
                    line_number: line_idx + 1,
                    line_content: line.trim().to_string(),
                });
                if matches.len() >= 100 {
                    return matches;
                }
            }
        }
    }

    // 2. Scan workspace folder if open
    if let Some(root) = &ws.root_path {
        scan_dir_for_search(root, q, &mut matches, 0);
    }

    matches
}

fn scan_dir_for_search(dir: &std::path::Path, query: &str, matches: &mut Vec<crate::state::FileSearchMatch>, depth: usize) {
    if depth > 6 || matches.len() >= 100 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "vendor-offline" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                scan_dir_for_search(&path, query, matches, depth + 1);
            } else if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(query) {
                            if !matches.iter().any(|m| m.file_path == path && m.line_number == line_idx + 1) {
                                matches.push(crate::state::FileSearchMatch {
                                    file_path: path.clone(),
                                    line_number: line_idx + 1,
                                    line_content: line.trim().to_string(),
                                });
                            }
                            if matches.len() >= 100 {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn perform_workspace_replace_all(ws: &mut codium_core::Workspace, search_query: &str, replace_query: &str) -> (usize, usize) {
    if search_query.is_empty() {
        return (0, 0);
    }
    let mut total_occurrences = 0;
    let mut files_changed = 0;

    for doc in &mut ws.open_documents {
        let text = doc.text();
        if text.contains(search_query) {
            let count = text.matches(search_query).count();
            if count > 0 {
                let new_text = text.replace(search_query, replace_query);
                doc.set_text(&new_text);
                total_occurrences += count;
                files_changed += 1;
            }
        }
    }

    if let Some(root) = &ws.root_path {
        replace_in_dir(root, search_query, replace_query, &mut total_occurrences, &mut files_changed, 0);
    }

    (total_occurrences, files_changed)
}

fn replace_in_dir(dir: &std::path::Path, search: &str, replace: &str, total: &mut usize, files: &mut usize, depth: usize) {
    if depth > 6 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "vendor-offline" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                replace_in_dir(&path, search, replace, total, files, depth + 1);
            } else if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains(search) {
                        let count = content.matches(search).count();
                        let new_content = content.replace(search, replace);
                        if std::fs::write(&path, new_content).is_ok() {
                            *total += count;
                            *files += 1;
                        }
                    }
                }
            }
        }
    }
}

fn perform_replace_single_occurrence(
    ws: &mut codium_core::Workspace,
    search_query: &str,
    replace_query: &str,
    target_match: &crate::state::FileSearchMatch,
) -> bool {
    if search_query.is_empty() {
        return false;
    }
    // 1. If it's an open document in memory:
    for doc in &mut ws.open_documents {
        if let Some(doc_path) = doc.path() {
            if doc_path == target_match.file_path {
                let content = doc.text();
                let mut replaced = false;
                let mut new_lines = Vec::new();
                for (idx, line) in content.lines().enumerate() {
                    if idx + 1 == target_match.line_number && !replaced && line.contains(search_query) {
                        let new_line = line.replacen(search_query, replace_query, 1);
                        new_lines.push(new_line);
                        replaced = true;
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                if replaced {
                    let mut new_text = new_lines.join("\n");
                    if content.ends_with('\n') {
                        new_text.push('\n');
                    }
                    doc.set_text(&new_text);
                    return true;
                }
            }
        }
    }

    // 2. Otherwise read from disk and update:
    if target_match.file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&target_match.file_path) {
            let mut replaced = false;
            let mut new_lines = Vec::new();
            for (idx, line) in content.lines().enumerate() {
                if idx + 1 == target_match.line_number && !replaced && line.contains(search_query) {
                    let new_line = line.replacen(search_query, replace_query, 1);
                    new_lines.push(new_line);
                    replaced = true;
                } else {
                    new_lines.push(line.to_string());
                }
            }
            if replaced {
                let mut new_text = new_lines.join("\n");
                if content.ends_with('\n') {
                    new_text.push('\n');
                }
                let _ = std::fs::write(&target_match.file_path, new_text);
                return true;
            }
        }
    }
    false
}

/// Educational Doc Tutor panel for learning programming
fn doc_tutor_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let locale = state.locale;
    let tutor = DocTutor::with_assets(&state.settings.get().resolve_assets_dir());
    let cards = tutor.for_language(&codium_core::Language::C);

    v_stack((
        h_stack((
            label(move || locale.get().doc_tutor_title()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().accent_edu)
            }),
        ))
        .style(move |s| {
            s.height(32.0).padding_horiz(12.0).items_center()
                .border_bottom(1.0).border_color(theme.get().border)
        }),
        label(move || locale.get().doc_tutor_subtitle()).style(move |s| {
            s.font_size(11.0)
                .color(theme.get().fg_muted)
                .margin(10.0)
        }),
        scroll(
            v_stack_from_iter(
                cards.into_iter().map(|c| {
                    let title = c.title.clone();
                    let content = c.content.clone();
                    let example = c.example.clone();
                    v_stack((
                        label(move || title.clone()).style(move |s| {
                            s.font_size(12.0)
                                .font_weight(Weight::BOLD)
                                .color(theme.get().accent_edu)
                        }),
                        label(move || content.clone()).style(move |s| {
                            s.font_size(11.0)
                                .color(theme.get().fg_primary)
                                .margin_top(4.0)
                        }),
                        container(
                            label(move || example.clone()).style(move |s| {
                                s.font_size(10.5)
                                    .color(theme.get().fg_muted)
                            }),
                        )
                        .style(move |s| {
                            s.background(theme.get().bg_editor)
                                .padding(6.0)
                                .margin_top(6.0)
                                .border_radius(4.0)
                        }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .padding(10.0)
                            .margin_bottom(8.0)
                            .background(theme.get().hover_bg)
                            .border_radius(6.0)
                    })
                })
            ).style(|s| s.flex_col().padding(10.0))
        ).style(|s| s.flex_grow(1.0).width_pct(100.0)),
    ))
    .style(|s| s.flex_col().size_pct(100.0, 100.0))
}

/// AI Assistant panel in AI edition
fn ai_assistant_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let locale = state.locale;

    v_stack((
        h_stack((
            label(move || locale.get().ai_assistant_title()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().accent_ai)
            }),
        ))
        .style(move |s| {
            s.height(32.0).padding_horiz(12.0).items_center()
                .border_bottom(1.0).border_color(theme.get().border)
        }),
        label(move || locale.get().tr("Local LLM: Connected (Offline)")).style(move |s| {
            s.font_size(11.0)
                .color(theme.get().accent_ai)
                .margin(10.0)
        }),
        container(
            label(move || locale.get().tr("Ask about your code, refactorings, or architecture. Running offline on background thread.")).style(move |s| {
                s.font_size(11.0).color(theme.get().fg_primary)
            }),
        )
        .style(move |s| {
            s.background(theme.get().hover_bg)
                .padding(10.0)
                .margin_horiz(10.0)
                .border_radius(6.0)
        }),
    ))
    .style(|s| s.flex_col().size_pct(100.0, 100.0))
}

/// Settings and university policy panel
fn settings_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let policy = state.edu_policy;
    let locale = state.locale;
    let settings = state.settings;

    v_stack((
        h_stack((
            label(move || locale.get().settings_title()).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
            }),
        ))
        .style(move |s| {
            s.height(32.0).padding_horiz(12.0).items_center()
                .border_bottom(1.0).border_color(theme.get().border)
        }),
        v_stack((
            label(move || format!("{} {}", locale.get().tr("Institution:"), policy.get().institution_name)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_primary).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("AI Code Gen Allowed:"), policy.get().allow_ai_generation)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Autocomplete Assistant Allowed:"), policy.get().allow_autocomplete_assistant)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Manuals Access Allowed:"), policy.get().allow_manuals_access)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Autocomplete Assistant Path:"), settings.get().autocomplete_assistant_path)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Manuals Directory:"), settings.get().manuals_dir)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Plugins Dir:"), settings.get().plugins_dir)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Offline Exam Mode:"), policy.get().exam_mode)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("UI Font:"), settings.get().ui_font_family)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Editor Font:"), settings.get().editor_font_family)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Terminal Font:"), settings.get().terminal_font_family)).style(move |s| {
                s.font_size(11.5).color(theme.get().fg_muted).margin_bottom(6.0)
            }),
            label(move || format!("{} {}", locale.get().tr("Language / Nyelv:"), locale.get().code())).style(move |s| {
                s.font_size(11.5).color(theme.get().accent_edu).margin_top(8.0)
            }),
        ))
        .style(|s| s.padding(12.0)),
    ))
    .style(|s| s.flex_col().size_pct(100.0, 100.0))
}
