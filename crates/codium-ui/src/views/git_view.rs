//! # OmniGit View for BM_codium
//!
//! Visual Source Control and Git Graph integration powered by `dabzse/omnigit`.
//! Provides repository status tracking, staging/unstaging, commit box,
//! and commit history visualization.

use codium_core::{
    format_timestamp_relative, GitCommit, GitExecutor, GitFileChange, GitGraphNode,
};
use floem::event::{Event, EventListener};
use floem::kurbo::Point;
use floem::peniko::{Brush, Color};
use floem::prelude::*;
use floem::style::{CursorStyle, Display};
use floem::text::Weight;
use std::path::PathBuf;
use std::str::FromStr;

use crate::icons::{
    render_svg, ICON_ARROW_DOWN, ICON_ARROW_UP, ICON_CHECK, ICON_CHEVRON_DOWN, ICON_CHEVRON_RIGHT,
    ICON_COPY, ICON_DOCK_PANEL, ICON_GIT_BRANCH, ICON_HISTORY, ICON_LIST_FLAT, ICON_LIST_TREE,
    ICON_MINUS, ICON_PLUS, ICON_REFRESH, ICON_SOURCE_CONTROL,
};
use crate::state::{AppState, BottomTab};

/// Helper to get the canonical Git repository root directory
fn resolve_repo_dir(state: AppState) -> PathBuf {
    if let Some(ref root) = state.git_status.get().root_dir {
        return root.clone();
    }
    if let Some(ref ws_root) = state.workspace.get().root_path {
        if let Some(git_root) = GitExecutor::get_repo_root(ws_root) {
            return git_root;
        }
        return ws_root.clone();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(git_root) = GitExecutor::get_repo_root(&cwd) {
        return git_root;
    }
    cwd
}

/// Main Source Control (OmniGit) view in sidebar
pub fn git_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let locale = state.locale;
    let git_status = state.git_status;
    let commit_msg = state.git_commit_message;

    v_stack((
        // 1. Header Bar: Title, Branch Pill & Refresh Action
        h_stack((
            // Title
            h_stack((
                render_svg(ICON_SOURCE_CONTROL, 14.0),
                label(move || "OMNIGIT".to_string()).style(move |s| {
                    s.font_size(11.0)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().fg_primary)
                        .margin_left(6.0)
                }),
            ))
            .style(|s| s.items_center()),

            // Actions & Branch
            h_stack((
                // Current Branch indicator
                container(
                    h_stack((
                        render_svg(ICON_GIT_BRANCH, 12.0),
                        label(move || {
                            let st = git_status.get();
                            if st.is_repo {
                                st.current_branch.clone()
                            } else {
                                "No repo".to_string()
                            }
                        })
                        .style(move |s| {
                            s.font_size(10.5)
                                .font_weight(Weight::SEMIBOLD)
                                .color(theme.get().fg_primary)
                                .margin_left(4.0)
                        }),
                    ))
                    .style(|s| s.items_center()),
                )
                .style(move |s| {
                    s.padding_horiz(6.0)
                        .height(20.0)
                        .background(theme.get().hover_bg)
                        .border_radius(3.0)
                        .items_center()
                        .margin_right(6.0)
                }),

                // View as Tree / Flat List Toggle Button
                container(
                    dyn_container(
                        move || state.git_view_as_tree.get(),
                        move |as_tree| {
                            if as_tree {
                                render_svg(ICON_LIST_FLAT, 13.0).into_any()
                            } else {
                                render_svg(ICON_LIST_TREE, 13.0).into_any()
                            }
                        },
                    )
                )
                .on_click_stop(move |_| {
                    let new_val = !state.git_view_as_tree.get_untracked();
                    state.git_view_as_tree.set(new_val);
                    state.settings.update(|s| s.git_view_as_tree = new_val);
                    let _ = state.settings.get_untracked().save();
                    let msg = if new_val { "OmniGit: View as Tree" } else { "OmniGit: View as List" };
                    state.status_message.set(msg.to_string());
                })
                .style(move |s| {
                    s.size(22.0, 22.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
                        .background(palette::css::TRANSPARENT)
                        .cursor(CursorStyle::Pointer)
                        .margin_right(4.0)
                        .hover(move |s| s.background(theme.get().hover_bg))
                }),

                // Refresh Button
                container(render_svg(ICON_REFRESH, 13.0))
                    .on_click_stop(move |_| {
                        state.refresh_git();
                        state.status_message.set("OmniGit: Status refreshed".to_string());
                    })
                    .style(move |s| {
                        s.size(22.0, 22.0)
                            .items_center()
                            .justify_center()
                            .border_radius(3.0)
                            .background(palette::css::TRANSPARENT)
                            .cursor(CursorStyle::Pointer)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    }),
            ))
            .style(|s| s.items_center()),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(35.0)
                .justify_between()
                .padding_horiz(12.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(theme.get().border)
        }),

        // 2. Body: Commit input + Scrollable changes and commits
        dyn_container(
            move || git_status.get().is_repo,
            move |is_repo| {
                if !is_repo {
                    return v_stack((
                        label(move || "🌿").style(|s| s.font_size(28.0).margin_bottom(8.0)),
                        label(move || locale.get().tr("This folder is not a Git repository.")).style(
                            move |s| {
                                s.font_size(11.5)
                                    .color(theme.get().fg_muted)
                                    .margin_bottom(12.0)
                            },
                        ),
                        container(
                            label(move || locale.get().tr("Initialize Repository")).style(move |s| {
                                s.font_size(11.5)
                                    .font_weight(Weight::BOLD)
                                    .color(theme.get().fg_button)
                            }),
                        )
                        .on_click_stop(move |_| {
                            let repo_dir = resolve_repo_dir(state);
                            let _ = GitExecutor::exec(&repo_dir, &["init"]);
                            state.refresh_git();
                        })
                        .style(move |s| {
                            s.background(theme.get().bg_button)
                                .padding_horiz(14.0)
                                .height(28.0)
                                .items_center()
                                .border_radius(4.0)
                                .cursor(CursorStyle::Pointer)
                                .hover(move |s| s.background(theme.get().hover_button))
                        }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .padding(16.0)
                            .items_center()
                            .justify_center()
                    })
                    .into_any();
                }

                // Repo view
                v_stack((
                    // Commit Box Section
                    v_stack((
                        text_input(commit_msg)
                            .placeholder("Message (Enter to commit)")
                            .on_enter(move || {
                                trigger_commit(state);
                            })
                            .style(move |s| {
                                s.width_pct(100.0)
                                    .height(28.0)
                                    .background(theme.get().bg_editor)
                                    .border(1.0)
                                    .border_color(theme.get().border)
                                    .border_radius(4.0)
                                    .padding_horiz(8.0)
                                    .color(theme.get().fg_primary)
                                    .cursor_color(Brush::Solid(palette::css::WHITE))
                                    .font_size(11.0)
                                    .outline(0.0)
                                    .outline_color(palette::css::TRANSPARENT)
                                    .hover(move |s| {
                                        s.background(theme.get().bg_editor)
                                            .border_color(theme.get().accent_blue)
                                    })
                                    .focus(move |s| {
                                        s.background(theme.get().bg_editor)
                                            .border_color(theme.get().accent_blue)
                                            .outline(0.0)
                                            .hover(move |s| {
                                                s.background(theme.get().bg_editor)
                                                    .border_color(theme.get().accent_blue)
                                            })
                                    })
                            }),
                        // Commit Button
                        container(
                            h_stack((
                                render_svg(ICON_CHECK, 13.0),
                                label(move || locale.get().tr("Commit")).style(move |s| {
                                    s.font_size(11.0)
                                        .font_weight(Weight::BOLD)
                                        .color(theme.get().fg_button)
                                        .margin_left(6.0)
                                }),
                            ))
                            .style(|s| s.items_center().justify_center()),
                        )
                        .on_click_stop(move |_| {
                            trigger_commit(state);
                        })
                        .style(move |s| {
                            s.width_pct(100.0)
                                .height(28.0)
                                .margin_top(6.0)
                                .background(theme.get().bg_button)
                                .items_center()
                                .justify_center()
                                .border_radius(4.0)
                                .cursor(CursorStyle::Pointer)
                                .hover(move |s| s.background(theme.get().hover_button))
                        }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .padding(10.0)
                            .border_bottom(1.0)
                            .border_color(theme.get().border)
                    }),

                    // Dynamic Reorderable Layout: Commits at Top vs Commits at Bottom
                    dyn_container(
                        move || state.git_graph_at_top.get(),
                        move |at_top| {
                            if at_top {
                                // Commits Graph at the top of the sidebar!
                                v_stack((
                                    render_commits_section(state),
                                    commit_graph_splitter(state),
                                    scroll(
                                        v_stack((
                                            render_staged_section(state),
                                            render_unstaged_section(state),
                                        ))
                                        .style(|s| s.width_pct(100.0)),
                                    )
                                    .scroll_style(|s| s.shrink_to_fit().handle_thickness(8.0).handle_rounded(true))
                                    .style(|s| {
                                        s.flex_grow(1.0)
                                            .flex_shrink(1.0)
                                            .flex_basis(0.0)
                                            .min_height(60.0)
                                            .width_pct(100.0)
                                    }),
                                ))
                                .style(|s| {
                                    s.flex_col()
                                        .flex_grow(1.0)
                                        .flex_shrink(1.0)
                                        .flex_basis(0.0)
                                        .min_height(0.0)
                                        .width_pct(100.0)
                                })
                                .into_any()
                            } else {
                                // Default: Staged & Changes first, Commits Graph below!
                                v_stack((
                                    scroll(
                                        v_stack((
                                            render_staged_section(state),
                                            render_unstaged_section(state),
                                        ))
                                        .style(|s| s.width_pct(100.0)),
                                    )
                                    .scroll_style(|s| s.shrink_to_fit().handle_thickness(8.0).handle_rounded(true))
                                    .style(|s| {
                                        s.flex_grow(1.0)
                                            .flex_shrink(1.0)
                                            .flex_basis(0.0)
                                            .min_height(60.0)
                                            .width_pct(100.0)
                                    }),
                                    commit_graph_splitter(state),
                                    render_commits_section(state),
                                ))
                                .style(|s| {
                                    s.flex_col()
                                        .flex_grow(1.0)
                                        .flex_shrink(1.0)
                                        .flex_basis(0.0)
                                        .min_height(0.0)
                                        .width_pct(100.0)
                                })
                                .into_any()
                            }
                        },
                    )
                    .style(|s| {
                        s.flex_col()
                            .flex_grow(1.0)
                            .flex_shrink(1.0)
                            .flex_basis(0.0)
                            .min_height(0.0)
                            .width_pct(100.0)
                    }),
                ))
                .style(|s| {
                    s.flex_col()
                        .flex_grow(1.0)
                        .flex_shrink(1.0)
                        .flex_basis(0.0)
                        .min_height(0.0)
                        .width_pct(100.0)
                })
                .into_any()
            },
        )
        .style(|s| {
            s.flex_col()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .width_pct(100.0)
        }),
    ))
    .style(move |s| {
        s.flex_col()
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .flex_basis(0.0)
            .size_pct(100.0, 100.0)
            .min_height(0.0)
            .background(theme.get().bg_sidebar)
    })
}

fn trigger_commit(state: AppState) {
    let repo_dir = resolve_repo_dir(state);
    let msg = state.git_commit_message.get().trim().to_string();
    if msg.is_empty() {
        state.status_message.set("OmniGit: Please enter a commit message".to_string());
        return;
    }

    let status = state.git_status.get();
    // If nothing staged, but there are unstaged changes, stage all first (like VS Code prompt)
    if status.staged.is_empty() && (status.unstaged.len() + status.untracked.len()) > 0 {
        let _ = GitExecutor::stage_all(&repo_dir);
    }

    match GitExecutor::commit(&repo_dir, &msg) {
        Ok(_) => {
            state.git_commit_message.set(String::new());
            state.status_message.set("OmniGit: Committed successfully".to_string());
            state.refresh_git();
        }
        Err(e) => {
            state.status_message.set(format!("OmniGit commit error: {}", e));
        }
    }
}

/// Horizontal splitter bar for resizing the Git Graph section vertically in the sidebar
pub fn commit_graph_splitter(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_commits = state.show_commits_section;
    let drag_start: RwSignal<Option<Point>> = create_rw_signal(None);

    let view = empty();
    let view_id = view.id();

    view.on_event_stop(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(pointer_event) = event {
            if pointer_event.count == 2 {
                state.git_graph_height.set(320.0);
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
                let current_h = state.git_graph_height.get_untracked();
                let at_top = state.git_graph_at_top.get_untracked();
                let delta = if at_top {
                    pointer_event.pos.y - drag_start_point.y
                } else {
                    drag_start_point.y - pointer_event.pos.y
                };
                let new_h = (current_h + delta).clamp(60.0, 500.0);
                if (new_h - current_h).abs() > 0.1 {
                    state.git_graph_height.set(new_h);
                }
            }
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        if drag_start.get_untracked().is_some() {
            view_id.clear_active();
            drag_start.set(None);
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
            .display(if show_commits.get() { Display::Flex } else { Display::None })
    })
}

/// A node in the visual OmniGit file tree
#[derive(Debug, Clone)]
pub struct GitTreeItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub change: Option<GitFileChange>,
    pub children: Vec<GitTreeItem>,
}

/// Recursively builds a tree from a list of GitFileChange
pub fn build_git_tree(files: &[GitFileChange]) -> Vec<GitTreeItem> {
    let mut root_items: Vec<GitTreeItem> = Vec::new();

    for file in files {
        let parts: Vec<&str> = file.path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        let mut current_items = &mut root_items;
        let mut curr_path = String::new();

        for (i, part) in parts.iter().enumerate() {
            if !curr_path.is_empty() {
                curr_path.push('/');
            }
            curr_path.push_str(part);

            let is_last = i == parts.len() - 1;
            if is_last {
                current_items.push(GitTreeItem {
                    name: part.to_string(),
                    path: curr_path.clone(),
                    is_dir: false,
                    change: Some(file.clone()),
                    children: Vec::new(),
                });
            } else {
                let dir_name = part.to_string();
                let dir_path = curr_path.clone();
                let idx = if let Some(pos) = current_items.iter().position(|it| it.is_dir && it.name == dir_name) {
                    pos
                } else {
                    current_items.push(GitTreeItem {
                        name: dir_name,
                        path: dir_path,
                        is_dir: true,
                        change: None,
                        children: Vec::new(),
                    });
                    current_items.len() - 1
                };
                current_items = &mut current_items[idx].children;
            }
        }
    }

    fn sort_and_compact(items: &mut Vec<GitTreeItem>) {
        for item in items.iter_mut() {
            if item.is_dir {
                sort_and_compact(&mut item.children);
                // Compact single-child directory chains (e.g. crates/codium-ui/src)
                while item.children.len() == 1 && item.children[0].is_dir {
                    let single_child = item.children.remove(0);
                    item.name = format!("{}/{}", item.name, single_child.name);
                    item.path = single_child.path;
                    item.children = single_child.children;
                }
            }
        }
        items.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }

    sort_and_compact(&mut root_items);
    root_items
}

fn count_leaf_files(item: &GitTreeItem) -> usize {
    if !item.is_dir {
        1
    } else {
        item.children.iter().map(count_leaf_files).sum()
    }
}

fn render_git_tree_item(
    state: AppState,
    item: &GitTreeItem,
    depth: usize,
    is_staged: bool,
    out: &mut Vec<Box<dyn View>>,
) {
    let theme = state.theme;
    let icon_theme = state.icon_theme.get();
    let indent = (depth as f64) * 12.0 + 8.0;

    if item.is_dir {
        // Directory row
        let dir_path = item.path.clone();
        let dir_name = item.name.clone();
        let collapse_key = format!("{}:{}", if is_staged { "staged" } else { "unstaged" }, dir_path);
        let is_collapsed = state.git_collapsed_dirs.get().contains(&collapse_key);
        let folder_svg = icon_theme.get_folder_icon(!is_collapsed).to_string();
        let child_count = count_leaf_files(item);
        let toggle_key = collapse_key.clone();

        let row = h_stack((
            // Left: Chevron + Folder Icon + Compact Dir Name + Count (clickable to collapse/expand)
            h_stack((
                render_svg(if is_collapsed { ICON_CHEVRON_RIGHT } else { ICON_CHEVRON_DOWN }, 10.0)
                    .style(|s| s.margin_right(3.0).items_center().justify_center().flex_shrink(0.0)),
                svg(move || folder_svg.clone()).style(|s| {
                    s.size(14.0, 14.0)
                        .margin_right(5.0)
                        .items_center()
                        .justify_center()
                        .flex_shrink(0.0)
                }),
                label(move || dir_name.clone()).style(move |s| {
                    s.font_size(11.0)
                        .font_weight(Weight::SEMIBOLD)
                        .color(theme.get().fg_primary)
                        .text_ellipsis()
                }),
                label(move || format!("{}", child_count)).style(move |s| {
                    s.font_size(9.0)
                        .color(theme.get().fg_muted)
                        .background(theme.get().hover_bg)
                        .padding_horiz(4.0)
                        .border_radius(6.0)
                        .margin_left(4.0)
                        .flex_shrink(0.0)
                }),
            ))
            .on_click_stop(move |_| {
                state.git_collapsed_dirs.update(|set| {
                    if set.contains(&toggle_key) {
                        set.remove(&toggle_key);
                    } else {
                        set.insert(toggle_key.clone());
                    }
                });
            })
            .style(|s| s.items_center().flex_grow(1.0).min_width(0.0).cursor(CursorStyle::Pointer)),

            // Right: Stage/Unstage directory action
            if is_staged {
                container(render_svg(ICON_MINUS, 12.0))
                    .on_click_stop(move |_| {
                        let repo_dir = resolve_repo_dir(state);
                        let _ = GitExecutor::unstage_file(&repo_dir, &dir_path);
                        state.refresh_git();
                    })
                    .style(move |s| {
                        s.width(22.0)
                            .height(18.0)
                            .items_center()
                            .justify_center()
                            .border_radius(3.0)
                            .background(palette::css::TRANSPARENT)
                            .cursor(CursorStyle::Pointer)
                            .flex_shrink(0.0)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    })
                    .into_any()
            } else {
                container(render_svg(ICON_PLUS, 12.0))
                    .on_click_stop(move |_| {
                        let repo_dir = resolve_repo_dir(state);
                        let _ = GitExecutor::stage_file(&repo_dir, &dir_path);
                        state.refresh_git();
                    })
                    .style(move |s| {
                        s.width(22.0)
                            .height(18.0)
                            .items_center()
                            .justify_center()
                            .border_radius(3.0)
                            .background(palette::css::TRANSPARENT)
                            .cursor(CursorStyle::Pointer)
                            .flex_shrink(0.0)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    })
                    .into_any()
            },
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(22.0)
                .padding_left(indent)
                .padding_right(6.0)
                .items_center()
                .justify_between()
                .flex_shrink(0.0)
                .hover(move |s| s.background(theme.get().hover_bg))
        });

        out.push(Box::new(row));

        if !is_collapsed {
            for child in &item.children {
                render_git_tree_item(state, child, depth + 1, is_staged, out);
            }
        }
    } else {
        // File row
        let target_path = item.path.clone();
        let action_path = item.path.clone();
        let file_name = item.name.clone();
        let file_svg = icon_theme.get_file_icon(&item.name).to_string();
        let st = item
            .change
            .as_ref()
            .map(|c| c.status)
            .unwrap_or(codium_core::FileChangeStatus::Modified);
        let badge_color = Color::from_str(st.color_hex()).unwrap_or(Color::from_rgb8(150, 150, 150));
        let badge_text = st.badge().to_string();

        let row = h_stack((
            // Left: Spacer (aligns with folder chevron) + File icon + File name (click to open)
            container(
                h_stack((
                    empty().style(|s| s.width(13.0).flex_shrink(0.0)),
                    svg(move || file_svg.clone()).style(|s| {
                        s.size(14.0, 14.0)
                            .margin_right(5.0)
                            .items_center()
                            .justify_center()
                            .flex_shrink(0.0)
                    }),
                    label(move || file_name.clone()).style(move |s| {
                        s.font_size(11.0)
                            .color(theme.get().fg_primary)
                            .text_ellipsis()
                    }),
                ))
                .style(|s| s.items_center().flex_grow(1.0).min_width(0.0)),
            )
            .on_click_stop(move |_| {
                let repo_dir = resolve_repo_dir(state);
                let target = repo_dir.join(&target_path);
                state.workspace.update(|w| {
                    let _ = w.open_file(target);
                });
            })
            .style(|s| s.flex_grow(1.0).min_width(0.0).cursor(CursorStyle::Pointer)),

            // Right: Status badge (S/A/M/D/U) + Action button (+ / -)
            h_stack((
                // Status badge
                label(move || badge_text.clone()).style(move |s| {
                    s.font_size(9.5)
                        .font_weight(Weight::BOLD)
                        .color(badge_color)
                        .margin_right(4.0)
                        .flex_shrink(0.0)
                }),
                // Button: - for staged, + for unstaged
                if is_staged {
                    container(render_svg(ICON_MINUS, 12.0))
                        .on_click_stop(move |_| {
                            let repo_dir = resolve_repo_dir(state);
                            match GitExecutor::unstage_file(&repo_dir, &action_path) {
                                Ok(_) => {
                                    state
                                        .status_message
                                        .set(format!("OmniGit: Unstaged {}", action_path));
                                    state.refresh_git();
                                }
                                Err(e) => {
                                    state.status_message.set(format!("OmniGit error: {}", e));
                                }
                            }
                        })
                        .style(move |s| {
                            s.width(22.0)
                                .height(18.0)
                                .items_center()
                                .justify_center()
                                .border_radius(3.0)
                                .background(palette::css::TRANSPARENT)
                                .cursor(CursorStyle::Pointer)
                                .flex_shrink(0.0)
                                .hover(move |s| s.background(theme.get().hover_bg))
                        })
                        .into_any()
                } else {
                    container(render_svg(ICON_PLUS, 12.0))
                        .on_click_stop(move |_| {
                            let repo_dir = resolve_repo_dir(state);
                            match GitExecutor::stage_file(&repo_dir, &action_path) {
                                Ok(_) => {
                                    state
                                        .status_message
                                        .set(format!("OmniGit: Staged {}", action_path));
                                    state.refresh_git();
                                }
                                Err(e) => {
                                    state.status_message.set(format!("OmniGit error: {}", e));
                                }
                            }
                        })
                        .style(move |s| {
                            s.width(22.0)
                                .height(18.0)
                                .items_center()
                                .justify_center()
                                .border_radius(3.0)
                                .background(palette::css::TRANSPARENT)
                                .cursor(CursorStyle::Pointer)
                                .flex_shrink(0.0)
                                .hover(move |s| s.background(theme.get().hover_bg))
                        })
                        .into_any()
                },
            ))
            .style(|s| s.items_center().flex_shrink(0.0)),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(22.0)
                .padding_left(indent)
                .padding_right(6.0)
                .items_center()
                .justify_between()
                .flex_shrink(0.0)
                .hover(move |s| s.background(theme.get().hover_bg))
        });

        out.push(Box::new(row));
    }
}

/// Staged Changes header and list (supports both Tree and Flat List modes)
fn render_staged_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let git_status = state.git_status;
    let show_section = state.show_staged_section;

    v_stack((
        // Section Header
        h_stack((
            // Chevron toggle + Title + Count badge
            container(
                h_stack((
                    dyn_container(
                        move || show_section.get(),
                        move |is_open| {
                            if is_open {
                                render_svg(ICON_CHEVRON_DOWN, 11.0).into_any()
                            } else {
                                render_svg(ICON_CHEVRON_RIGHT, 11.0).into_any()
                            }
                        },
                    )
                    .style(|s| s.items_center().justify_center().margin_right(4.0)),
                    label(move || "STAGED CHANGES".to_string()).style(move |s| {
                        s.font_size(10.5)
                            .font_weight(Weight::BOLD)
                            .color(theme.get().fg_primary)
                    }),
                    // Count badge
                    label(move || format!("{}", git_status.get().staged.len())).style(move |s| {
                        s.font_size(9.5)
                            .color(theme.get().fg_muted)
                            .background(theme.get().hover_bg)
                            .padding_horiz(5.0)
                            .border_radius(8.0)
                            .margin_left(6.0)
                    }),
                ))
                .style(|s| s.items_center()),
            )
            .on_click_stop(move |_| {
                show_section.update(|v| *v = !*v);
            })
            .style(|s| s.items_center().cursor(CursorStyle::Pointer)),

            // Unstage All action
            container(render_svg(ICON_MINUS, 12.0))
                .on_click_stop(move |_| {
                    let repo_dir = resolve_repo_dir(state);
                    match GitExecutor::unstage_all(&repo_dir) {
                        Ok(_) => {
                            state.status_message.set("OmniGit: Unstaged all changes".to_string());
                            state.refresh_git();
                        }
                        Err(e) => {
                            state.status_message.set(format!("OmniGit error: {}", e));
                        }
                    }
                })
                .style(move |s| {
                    s.size(20.0, 20.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
                        .background(palette::css::TRANSPARENT)
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().hover_bg))
                }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(24.0)
                .padding_horiz(10.0)
                .items_center()
                .justify_between()
                .background(theme.get().bg_activity)
        }),

        // Staged Files list / tree
        dyn_container(
            move || (
                show_section.get(),
                state.git_view_as_tree.get(),
                state.git_collapsed_dirs.get(),
                git_status.get().staged,
            ),
            move |(is_open, as_tree, _, staged_files)| {
                if !is_open || staged_files.is_empty() {
                    return empty().into_any();
                }

                if as_tree {
                    let tree = build_git_tree(&staged_files);
                    let mut items = Vec::new();
                    for node in &tree {
                        render_git_tree_item(state, node, 0, true, &mut items);
                    }
                    v_stack_from_iter(items).style(|s| s.flex_col().width_pct(100.0)).into_any()
                } else {
                    let items = staged_files.into_iter().map(move |file| {
                        let file_path = file.path.clone();
                        let unstage_path = file.path.clone();
                        let click_open_path = file.path.clone();
                        let st = file.status;
                        let badge_color = Color::from_str(st.color_hex()).unwrap_or(Color::from_rgb8(150, 150, 150));

                        h_stack((
                            // Status badge (S, A, D, R, etc.)
                            label(move || st.badge().to_string()).style(move |s| {
                                s.font_size(10.0)
                                    .font_weight(Weight::BOLD)
                                    .color(badge_color)
                                    .margin_right(6.0)
                                    .flex_shrink(0.0)
                            }),
                            // File path (opens file when clicked)
                            container(
                                label(move || file_path.clone()).style(move |s| {
                                    s.font_size(11.0)
                                        .color(theme.get().fg_primary)
                                        .text_ellipsis()
                                }),
                            )
                            .on_click_stop(move |_| {
                                let repo_dir = resolve_repo_dir(state);
                                let target = repo_dir.join(&click_open_path);
                                state.workspace.update(|w| { let _ = w.open_file(target); });
                            })
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .flex_shrink(1.0)
                                    .min_width(0.0)
                                    .cursor(CursorStyle::Pointer)
                            }),
                            // Unstage button (-)
                            container(render_svg(ICON_MINUS, 12.0))
                                .on_click_stop(move |_| {
                                    let repo_dir = resolve_repo_dir(state);
                                    match GitExecutor::unstage_file(&repo_dir, &unstage_path) {
                                        Ok(_) => {
                                            state.status_message.set(format!("OmniGit: Unstaged {}", unstage_path));
                                            state.refresh_git();
                                        }
                                        Err(e) => {
                                            state.status_message.set(format!("OmniGit error: {}", e));
                                        }
                                    }
                                })
                                .style(move |s| {
                                    s.width(24.0)
                                        .height(20.0)
                                        .items_center()
                                        .justify_center()
                                        .border_radius(3.0)
                                        .background(palette::css::TRANSPARENT)
                                        .cursor(CursorStyle::Pointer)
                                        .flex_shrink(0.0)
                                        .hover(move |s| s.background(theme.get().hover_bg))
                                }),
                        ))
                        .style(move |s| {
                            s.width_pct(100.0)
                                .height(22.0)
                                .padding_horiz(10.0)
                                .items_center()
                                .flex_shrink(0.0)
                                .hover(move |s| s.background(theme.get().hover_bg))
                        })
                    });

                    v_stack_from_iter(items).style(|s| s.flex_col().width_pct(100.0)).into_any()
                }
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}

/// Changes (Unstaged + Untracked) header and list (supports both Tree and Flat List modes)
fn render_unstaged_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let git_status = state.git_status;
    let show_section = state.show_unstaged_section;

    v_stack((
        // Section Header
        h_stack((
            // Chevron toggle + Title + Count badge
            container(
                h_stack((
                    dyn_container(
                        move || show_section.get(),
                        move |is_open| {
                            if is_open {
                                render_svg(ICON_CHEVRON_DOWN, 11.0).into_any()
                            } else {
                                render_svg(ICON_CHEVRON_RIGHT, 11.0).into_any()
                            }
                        },
                    )
                    .style(|s| s.items_center().justify_center().margin_right(4.0)),
                    label(move || "CHANGES".to_string()).style(move |s| {
                        s.font_size(10.5)
                            .font_weight(Weight::BOLD)
                            .color(theme.get().fg_primary)
                    }),
                    // Count badge
                    label(move || {
                        let st = git_status.get();
                        format!("{}", st.unstaged.len() + st.untracked.len())
                    })
                    .style(move |s| {
                        s.font_size(9.5)
                            .color(theme.get().fg_muted)
                            .background(theme.get().hover_bg)
                            .padding_horiz(5.0)
                            .border_radius(8.0)
                            .margin_left(6.0)
                    }),
                ))
                .style(|s| s.items_center()),
            )
            .on_click_stop(move |_| {
                show_section.update(|v| *v = !*v);
            })
            .style(|s| s.items_center().cursor(CursorStyle::Pointer)),

            // Stage All action (+)
            container(render_svg(ICON_PLUS, 12.0))
                .on_click_stop(move |_| {
                    let repo_dir = resolve_repo_dir(state);
                    match GitExecutor::stage_all(&repo_dir) {
                        Ok(_) => {
                            state.status_message.set("OmniGit: Staged all changes".to_string());
                            state.refresh_git();
                        }
                        Err(e) => {
                            state.status_message.set(format!("OmniGit error: {}", e));
                        }
                    }
                })
                .style(move |s| {
                    s.size(20.0, 20.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
                        .background(palette::css::TRANSPARENT)
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().hover_bg))
                }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(24.0)
                .padding_horiz(10.0)
                .items_center()
                .justify_between()
                .background(theme.get().bg_activity)
                .margin_top(6.0)
        }),

        // Changes Files list / tree
        dyn_container(
            move || {
                let is_open = show_section.get();
                let as_tree = state.git_view_as_tree.get();
                let collapsed = state.git_collapsed_dirs.get();
                let st = git_status.get();
                let mut all = st.unstaged;
                all.extend(st.untracked);
                (is_open, as_tree, collapsed, all)
            },
            move |(is_open, as_tree, _, files)| {
                if !is_open || files.is_empty() {
                    return empty().into_any();
                }

                if as_tree {
                    let tree = build_git_tree(&files);
                    let mut items = Vec::new();
                    for node in &tree {
                        render_git_tree_item(state, node, 0, false, &mut items);
                    }
                    v_stack_from_iter(items).style(|s| s.flex_col().width_pct(100.0)).into_any()
                } else {
                    let items = files.into_iter().map(move |file| {
                        let file_path = file.path.clone();
                        let stage_path = file.path.clone();
                        let click_open_path = file.path.clone();
                        let st = file.status;
                        let badge_color = Color::from_str(st.color_hex()).unwrap_or(Color::from_rgb8(150, 150, 150));

                        h_stack((
                            // Status badge (M, D, U, etc.)
                            label(move || st.badge().to_string()).style(move |s| {
                                s.font_size(10.0)
                                    .font_weight(Weight::BOLD)
                                    .color(badge_color)
                                    .margin_right(6.0)
                                    .flex_shrink(0.0)
                            }),
                            // File path (opens file when clicked)
                            container(
                                label(move || file_path.clone()).style(move |s| {
                                    s.font_size(11.0)
                                        .color(theme.get().fg_primary)
                                        .text_ellipsis()
                                }),
                            )
                            .on_click_stop(move |_| {
                                let repo_dir = resolve_repo_dir(state);
                                let target = repo_dir.join(&click_open_path);
                                state.workspace.update(|w| { let _ = w.open_file(target); });
                            })
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .flex_shrink(1.0)
                                    .min_width(0.0)
                                    .cursor(CursorStyle::Pointer)
                            }),
                            // Stage button (+)
                            container(render_svg(ICON_PLUS, 12.0))
                                .on_click_stop(move |_| {
                                    let repo_dir = resolve_repo_dir(state);
                                    match GitExecutor::stage_file(&repo_dir, &stage_path) {
                                        Ok(_) => {
                                            state.status_message.set(format!("OmniGit: Staged {}", stage_path));
                                            state.refresh_git();
                                        }
                                        Err(e) => {
                                            state.status_message.set(format!("OmniGit error: {}", e));
                                        }
                                    }
                                })
                                .style(move |s| {
                                    s.width(24.0)
                                        .height(20.0)
                                        .items_center()
                                        .justify_center()
                                        .border_radius(3.0)
                                        .background(palette::css::TRANSPARENT)
                                        .cursor(CursorStyle::Pointer)
                                        .flex_shrink(0.0)
                                        .hover(move |s| s.background(theme.get().hover_bg))
                                }),
                        ))
                        .style(move |s| {
                            s.width_pct(100.0)
                                .height(22.0)
                                .padding_horiz(10.0)
                                .items_center()
                                .flex_shrink(0.0)
                                .hover(move |s| s.background(theme.get().hover_bg))
                        })
                    });

                    v_stack_from_iter(items).style(|s| s.flex_col().width_pct(100.0)).into_any()
                }
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}

/// OmniGit Commits / Graph section for the sidebar
fn render_commits_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let commits = state.git_commits;
    let selected_commit = state.selected_commit_hash;
    let show_commits = state.show_commits_section;
    let git_graph_at_top = state.git_graph_at_top;

    v_stack((
        // Section Header
        h_stack((
            // Left: Chevron + Title + Count badge
            container(
                h_stack((
                    dyn_container(
                        move || show_commits.get(),
                        move |is_open| {
                            if is_open {
                                render_svg(ICON_CHEVRON_DOWN, 11.0).into_any()
                            } else {
                                render_svg(ICON_CHEVRON_RIGHT, 11.0).into_any()
                            }
                        },
                    )
                    .style(|s| s.items_center().justify_center().margin_right(4.0)),
                    render_svg(ICON_HISTORY, 12.0),
                    label(move || "COMMITS / GRAPH".to_string()).style(move |s| {
                        s.font_size(10.5)
                            .font_weight(Weight::BOLD)
                            .color(theme.get().fg_primary)
                            .margin_left(4.0)
                    }),
                    // Commit count badge
                    label(move || format!("{}", commits.get().len())).style(move |s| {
                        s.font_size(9.5)
                            .color(theme.get().fg_muted)
                            .background(theme.get().hover_bg)
                            .padding_horiz(5.0)
                            .border_radius(8.0)
                            .margin_left(6.0)
                    }),
                ))
                .style(|s| s.items_center()),
            )
            .on_click_stop(move |_| {
                show_commits.update(|v| *v = !*v);
            })
            .style(|s| s.items_center().cursor(CursorStyle::Pointer)),

            // Right actions: Reorder (Move Up/Down) + Dock to Bottom + Refresh
            h_stack((
                // Reorder button: Move Up or Move Down
                container(
                    dyn_container(
                        move || git_graph_at_top.get(),
                        move |at_top| {
                            if at_top {
                                render_svg(ICON_ARROW_DOWN, 12.0).into_any()
                            } else {
                                render_svg(ICON_ARROW_UP, 12.0).into_any()
                            }
                        },
                    )
                )
                .on_click_stop(move |_| {
                    let new_val = !git_graph_at_top.get_untracked();
                    git_graph_at_top.set(new_val);
                    if new_val {
                        state.status_message.set("OmniGit: Commit graph moved to top".to_string());
                    } else {
                        state.status_message.set("OmniGit: Commit graph moved to bottom".to_string());
                    }
                })
                .style(move |s| {
                    s.size(20.0, 20.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
                        .background(palette::css::TRANSPARENT)
                        .cursor(CursorStyle::Pointer)
                        .margin_right(2.0)
                        .hover(move |s| s.background(theme.get().hover_bg))
                }),

                // Dock to Bottom panel button
                container(render_svg(ICON_DOCK_PANEL, 12.0))
                    .on_click_stop(move |_| {
                        state.active_bottom_tab.set(BottomTab::GitGraph);
                        state.show_bottom_panel.set(true);
                        state.status_message.set("OmniGit: Opened Git Graph in Bottom Panel".to_string());
                    })
                    .style(move |s| {
                        s.size(20.0, 20.0)
                            .items_center()
                            .justify_center()
                            .border_radius(3.0)
                            .background(palette::css::TRANSPARENT)
                            .cursor(CursorStyle::Pointer)
                            .margin_right(2.0)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    }),

                // Refresh action
                container(render_svg(ICON_REFRESH, 12.0))
                    .on_click_stop(move |_| {
                        state.refresh_git();
                        state.status_message.set("OmniGit: Commit graph refreshed".to_string());
                    })
                    .style(move |s| {
                        s.size(20.0, 20.0)
                            .items_center()
                            .justify_center()
                            .border_radius(3.0)
                            .background(palette::css::TRANSPARENT)
                            .cursor(CursorStyle::Pointer)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    }),
            ))
            .style(|s| s.items_center()),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(24.0)
                .padding_horiz(10.0)
                .items_center()
                .justify_between()
                .background(theme.get().bg_activity)
                .margin_top(4.0)
        }),

        // Commits Graph List
        dyn_container(
            move || (show_commits.get(), commits.get()),
            move |(is_open, commit_list)| {
                if !is_open {
                    return empty().into_any();
                }

                if commit_list.is_empty() {
                    return container(
                        label(move || "No commits in repository yet".to_string()).style(move |s| {
                            s.font_size(11.0)
                                .color(theme.get().fg_muted)
                                .padding(12.0)
                        }),
                    )
                    .style(|s| s.width_pct(100.0))
                    .into_any();
                }

                let nodes = GitExecutor::compute_graph_nodes(&commit_list);
                let items = nodes.into_iter().enumerate().map(move |(idx, node)| {
                    render_commit_graph_row(state, idx, node, selected_commit, false)
                });

                scroll(
                    v_stack_from_iter(items)
                        .style(|s| s.flex_col().width_pct(100.0))
                )
                .scroll_style(|s| s.shrink_to_fit().handle_thickness(8.0).handle_rounded(true))
                .style(move |s| {
                    s.width_pct(100.0)
                        .height(state.git_graph_height.get())
                        .min_height(60.0)
                        .flex_shrink(1.0)
                })
                .into_any()
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}

/// Render a single commit row with lanes, badges, subject and metadata
fn render_commit_graph_row(
    state: AppState,
    idx: usize,
    node: GitGraphNode,
    selected_commit: RwSignal<Option<String>>,
    is_bottom_view: bool,
) -> impl IntoView {
    let theme = state.theme;
    let commit = node.commit.clone();
    let hash = commit.hash.clone();
    let click_hash = commit.hash.clone();
    let short_hash = commit.short_hash.clone();
    let subject = commit.subject.clone();
    let author = commit.author_name.clone();
    let time = format_timestamp_relative(commit.author_date);
    let refs = commit.refs.clone();
    let lane = node.lane;
    let total_lanes = node.total_lanes;
    let active_lanes = node.active_lanes.clone();
    let is_head = node.is_head;
    let is_merge = node.is_merge;
    let is_root = node.is_root;
    let is_first = idx == 0;

    let node_color = Color::from_str(node.lane_color())
        .unwrap_or(Color::from_rgb8(59, 130, 246));

    // Graph lane strip width: 16px per lane column
    let graph_width = (total_lanes as f64 * 16.0).max(18.0);

    // Build the lane columns for this commit row
    let lane_cols: Vec<_> = (0..total_lanes)
        .map(|l| {
            let l_color = Color::from_str(GitGraphNode::color_for_lane(l))
                .unwrap_or(Color::from_rgb8(59, 130, 246));

            if l == lane {
                // This commit's column: top line + bullet dot + bottom line
                v_stack((
                    // Top line (connects to parent or child above)
                    container(empty()).style(move |s| {
                        s.width(2.0).height(7.0).background(if is_first {
                            palette::css::TRANSPARENT
                        } else {
                            node_color
                        })
                    }),
                    // Bullet node
                    if is_head {
                        // Outer colored circle with inner white dot for HEAD
                        container(
                            container(empty()).style(|s| {
                                s.size(3.0, 3.0)
                                    .border_radius(1.5)
                                    .background(palette::css::WHITE)
                            }),
                        )
                        .style(move |s| {
                            s.size(9.0, 9.0)
                                .border_radius(4.5)
                                .background(node_color)
                                .items_center()
                                .justify_center()
                        })
                        .into_any()
                    } else if is_merge {
                        // Merge node: ring with colored border
                        container(empty())
                            .style(move |s| {
                                s.size(9.0, 9.0)
                                    .border_radius(4.5)
                                    .border(2.0)
                                    .border_color(node_color)
                                    .background(theme.get().bg_sidebar)
                            })
                            .into_any()
                    } else {
                        // Standard commit node: solid colored circle
                        container(empty())
                            .style(move |s| {
                                s.size(7.0, 7.0)
                                    .border_radius(3.5)
                                    .background(node_color)
                            })
                            .into_any()
                    },
                    // Bottom line (connects to parent below, unless root)
                    container(empty()).style(move |s| {
                        s.width(2.0).height(7.0).background(if is_root {
                            palette::css::TRANSPARENT
                        } else {
                            node_color
                        })
                    }),
                ))
                .style(|s| {
                    s.width(16.0)
                        .height(24.0)
                        .items_center()
                        .justify_center()
                        .flex_shrink(0.0)
                })
                .into_any()
            } else if active_lanes.get(l).copied().unwrap_or(false) {
                // Passing lane: continuous vertical line
                container(container(empty()).style(move |s| {
                    s.width(2.0).height_pct(100.0).background(l_color)
                }))
                .style(|s| {
                    s.width(16.0)
                        .height(24.0)
                        .items_center()
                        .justify_center()
                        .flex_shrink(0.0)
                })
                .into_any()
            } else {
                // Empty column
                container(empty())
                    .style(|s| s.width(16.0).height(24.0).flex_shrink(0.0))
                    .into_any()
            }
        })
        .collect();

    // Ref pill badges (branches, tags)
    let ref_badges: Vec<_> = refs
        .into_iter()
        .map(move |r| {
            let is_head_ref = r.starts_with("HEAD") || r.contains("HEAD ->");
            let is_tag = r.starts_with("tag:");
            let is_remote = r.starts_with("origin/") || r.contains('/');

            let display_text = if let Some(stripped) = r.strip_prefix("HEAD -> ") {
                format!("⎇ {}", stripped)
            } else if let Some(stripped) = r.strip_prefix("tag: ") {
                format!("🏷 {}", stripped)
            } else if is_head_ref {
                "HEAD".to_string()
            } else if is_remote {
                format!("☁ {}", r)
            } else {
                format!("⎇ {}", r)
            };

            let (bg, fg, border_c) = if is_head_ref {
                (
                    Color::from_rgb8(20, 50, 35),
                    Color::from_rgb8(82, 183, 136),
                    Color::from_rgb8(45, 106, 79),
                )
            } else if is_tag {
                (
                    Color::from_rgb8(50, 40, 15),
                    Color::from_rgb8(251, 191, 36),
                    Color::from_rgb8(120, 85, 20),
                )
            } else if is_remote {
                (
                    Color::from_rgb8(30, 27, 60),
                    Color::from_rgb8(129, 140, 248),
                    Color::from_rgb8(55, 48, 163),
                )
            } else {
                (
                    Color::from_rgb8(20, 40, 60),
                    Color::from_rgb8(96, 165, 250),
                    Color::from_rgb8(30, 64, 110),
                )
            };

            container(label(move || display_text.clone()).style(move |s| {
                s.font_size(9.0)
                    .font_weight(Weight::BOLD)
                    .color(fg)
            }))
            .style(move |s| {
                s.padding_horiz(5.0)
                    .height(16.0)
                    .background(bg)
                    .border(1.0)
                    .border_color(border_c)
                    .border_radius(3.0)
                    .margin_right(4.0)
                    .items_center()
                    .flex_shrink(0.0)
            })
        })
        .collect();

    let is_selected_style = {
        let h = hash.clone();
        move || selected_commit.get().as_deref() == Some(&h)
    };
    let is_selected_container = {
        let h = hash.clone();
        move || selected_commit.get().as_deref() == Some(&h)
    };

    v_stack((
        // Row: Commit summary row
        container(
            if is_bottom_view {
                // Wide bottom panel layout: Lanes | Badges | Short Hash | Subject | Author | Time
                h_stack((
                    h_stack_from_iter(lane_cols).style(move |s| {
                        s.width(graph_width)
                            .height(24.0)
                            .items_center()
                            .flex_shrink(0.0)
                            .margin_right(6.0)
                    }),
                    h_stack_from_iter(ref_badges)
                        .style(|s| s.items_center().flex_shrink(0.0)),
                    label(move || short_hash.clone()).style(move |s| {
                        s.font_size(10.5)
                            .font_weight(Weight::BOLD)
                            .color(theme.get().accent_blue)
                            .width(65.0)
                            .flex_shrink(0.0)
                    }),
                    label(move || subject.clone()).style(move |s| {
                        s.font_size(11.0)
                            .color(theme.get().fg_primary)
                            .min_width(0.0)
                            .flex_grow(1.0)
                            .flex_shrink(1.0)
                            .text_ellipsis()
                    }),
                    label(move || author.clone()).style(move |s| {
                        s.font_size(10.0)
                            .color(theme.get().fg_muted)
                            .width(130.0)
                            .flex_shrink(0.0)
                            .text_ellipsis()
                            .margin_left(12.0)
                    }),
                    label(move || time.clone()).style(move |s| {
                        s.font_size(10.0)
                            .color(theme.get().fg_muted)
                            .width(100.0)
                            .flex_shrink(0.0)
                            .margin_left(8.0)
                    }),
                ))
                .style(|s| s.width_pct(100.0).height(24.0).items_center())
                .into_any()
            } else {
                // Compact sidebar layout (2 rows)
                h_stack((
                    // Graph Lanes visualization strip
                    h_stack_from_iter(lane_cols).style(move |s| {
                        s.width(graph_width)
                            .height(24.0)
                            .items_center()
                            .flex_shrink(0.0)
                            .margin_right(4.0)
                    }),
                    // Commit text content
                    v_stack((
                        // Row 1: Badges + Short Hash + Subject
                        h_stack((
                            h_stack_from_iter(ref_badges)
                                .style(|s| s.items_center().flex_shrink(0.0)),
                            label(move || short_hash.clone()).style(move |s| {
                                s.font_size(10.0)
                                    .font_weight(Weight::BOLD)
                                    .color(theme.get().accent_blue)
                                    .margin_right(6.0)
                                    .flex_shrink(0.0)
                            }),
                            label(move || subject.clone()).style(move |s| {
                                s.font_size(11.0)
                                    .color(theme.get().fg_primary)
                                    .min_width(0.0)
                                    .flex_grow(1.0)
                                    .flex_shrink(1.0)
                                    .text_ellipsis()
                            }),
                        ))
                        .style(|s| s.width_pct(100.0).items_center()),
                        // Row 2: Author & Relative Date
                        h_stack((label(move || format!("{} • {}", author, time)).style(
                            move |s| {
                                s.font_size(9.5).color(theme.get().fg_muted)
                            },
                        ),))
                        .style(|s| s.margin_top(1.0)),
                    ))
                    .style(|s| {
                        s.min_width(0.0)
                            .flex_grow(1.0)
                            .flex_shrink(1.0)
                    }),
                ))
                .style(|s| s.width_pct(100.0).items_center())
                .into_any()
            },
        )
        .on_click_stop(move |_| {
            selected_commit.update(|sel| {
                if sel.as_deref() == Some(&click_hash) {
                    *sel = None;
                } else {
                    *sel = Some(click_hash.clone());
                }
            });
        })
        .style(move |s| {
            s.width_pct(100.0)
                .padding_horiz(8.0)
                .padding_vert(3.0)
                .cursor(CursorStyle::Pointer)
                .flex_shrink(0.0)
                .apply_if(is_selected_style(), |s| {
                    s.background(theme.get().hover_bg)
                        .border_left(2.0)
                        .border_color(theme.get().accent_blue)
                })
                .hover(move |s| s.background(theme.get().hover_bg))
        }),

        // Expanded Commit Details Card (shown when selected)
        dyn_container(is_selected_container, move |selected| {
            if !selected {
                return empty().into_any();
            }
            render_commit_detail_card(state, commit.clone()).into_any()
        })
        .style(|s| s.width_pct(100.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .flex_shrink(0.0)
            .border_bottom(1.0)
            .border_color(theme.get().border)
    })
}

/// Detailed view of a selected commit in the commit graph
fn render_commit_detail_card(state: AppState, commit: GitCommit) -> impl IntoView {
    let theme = state.theme;
    let repo_dir = resolve_repo_dir(state);
    let full_hash = commit.hash.clone();
    let copy_hash = commit.hash.clone();
    let author_line = format!("{} <{}>", commit.author_name, commit.author_email);
    let body = commit.body.clone();
    let parents_str = if commit.parents.is_empty() {
        "Root commit (no parent)".to_string()
    } else {
        commit
            .parents
            .iter()
            .map(|p| p.chars().take(7).collect::<String>())
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Changed files for this commit
    let files = GitExecutor::get_commit_files(&repo_dir, &commit.hash).unwrap_or_default();

    v_stack((
        // Header: Hash & Copy Button
        h_stack((
            label(move || format!("Commit {}", &full_hash[..full_hash.len().min(12)])).style(
                move |s| {
                    s.font_size(10.5)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().accent_blue)
                },
            ),
            // Copy full hash button
            container(
                h_stack((
                    render_svg(ICON_COPY, 10.0),
                    label(move || "Copy".to_string()).style(move |s| {
                        s.font_size(9.5)
                            .color(theme.get().fg_muted)
                            .margin_left(3.0)
                    }),
                ))
                .style(|s| s.items_center()),
            )
            .on_click_stop(move |_| {
                let _ = floem::Clipboard::set_contents(copy_hash.clone());
                state
                    .status_message
                    .set("OmniGit: Commit hash copied".to_string());
            })
            .style(move |s| {
                s.padding_horiz(5.0)
                    .height(18.0)
                    .items_center()
                    .border_radius(3.0)
                    .background(theme.get().hover_bg)
                    .cursor(CursorStyle::Pointer)
                    .hover(move |s| s.background(theme.get().bg_button))
            }),
        ))
        .style(|s| s.width_pct(100.0).items_center().justify_between()),

        // Author & Parents metadata
        label(move || format!("Author: {}", author_line)).style(move |s| {
            s.font_size(9.5)
                .color(theme.get().fg_muted)
                .margin_top(4.0)
        }),
        label(move || format!("Parents: {}", parents_str)).style(move |s| {
            s.font_size(9.5)
                .color(theme.get().fg_muted)
                .margin_top(2.0)
        }),

        // Body (if non-empty)
        if !body.trim().is_empty() {
            let body_text = body.trim().to_string();
            container(
                label(move || body_text.clone()).style(move |s| {
                    s.font_size(10.0).color(theme.get().fg_primary)
                }),
            )
            .style(move |s| {
                s.width_pct(100.0)
                    .padding(6.0)
                    .margin_top(4.0)
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(3.0)
            })
            .into_any()
        } else {
            empty().into_any()
        },

        // Changed Files list header & entries
        if !files.is_empty() {
            let file_items: Vec<_> = files
                .into_iter()
                .map({
                    let dir = repo_dir.clone();
                    move |(status_str, file_path)| {
                        let click_target = dir.join(&file_path);
                        let (badge_text, badge_color) = match status_str.chars().next().unwrap_or('M') {
                            'A' => ("A", Color::from_rgb8(82, 183, 136)),
                            'D' => ("D", Color::from_rgb8(239, 68, 68)),
                            'R' => ("R", Color::from_rgb8(245, 158, 11)),
                            _ => ("M", Color::from_rgb8(96, 165, 250)),
                        };

                        h_stack((
                            label(move || badge_text.to_string()).style(move |s| {
                                s.font_size(9.5)
                                    .font_weight(Weight::BOLD)
                                    .color(badge_color)
                                    .margin_right(6.0)
                                    .flex_shrink(0.0)
                            }),
                            label(move || file_path.clone()).style(move |s| {
                                s.font_size(10.0)
                                    .color(theme.get().fg_primary)
                                    .min_width(0.0)
                                    .flex_grow(1.0)
                                    .flex_shrink(1.0)
                                    .text_ellipsis()
                            }),
                        ))
                        .on_click_stop(move |_| {
                            let target = click_target.clone();
                            state.workspace.update(|w| {
                                let _ = w.open_file(target);
                            });
                        })
                        .style(move |s| {
                            s.width_pct(100.0)
                                .height(20.0)
                                .padding_horiz(4.0)
                                .items_center()
                                .cursor(CursorStyle::Pointer)
                                .border_radius(2.0)
                                .hover(move |s| s.background(theme.get().hover_bg))
                        })
                    }
                })
                .collect();

            v_stack((
                label(move || "CHANGED FILES:".to_string()).style(move |s| {
                    s.font_size(9.5)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().fg_muted)
                        .margin_top(6.0)
                        .margin_bottom(2.0)
                }),
                v_stack_from_iter(file_items).style(|s| s.flex_col().width_pct(100.0)),
            ))
            .style(|s| s.width_pct(100.0))
            .into_any()
        } else {
            empty().into_any()
        },
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .padding(8.0)
            .background(theme.get().bg_activity)
            .border_top(1.0)
            .border_color(theme.get().border)
    })
}

/// OmniGit Full Git Graph view for the Bottom Dock Panel
pub fn render_bottom_git_graph(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let git_status = state.git_status;
    let commits = state.git_commits;
    let selected_commit = state.selected_commit_hash;

    v_stack((
        // Header Toolbar
        h_stack((
            h_stack((
                render_svg(ICON_GIT_BRANCH, 13.0),
                label(move || "GIT GRAPH".to_string()).style(move |s| {
                    s.font_size(11.0)
                        .font_weight(Weight::BOLD)
                        .color(theme.get().fg_primary)
                        .margin_left(6.0)
                }),
                // Branch pill
                container(
                    label(move || {
                        let st = git_status.get();
                        if st.is_repo {
                            st.current_branch.clone()
                        } else {
                            "No repo".to_string()
                        }
                    })
                    .style(move |s| {
                        s.font_size(10.0)
                            .font_weight(Weight::SEMIBOLD)
                            .color(theme.get().fg_primary)
                    }),
                )
                .style(move |s| {
                    s.padding_horiz(6.0)
                        .height(18.0)
                        .background(theme.get().hover_bg)
                        .border_radius(3.0)
                        .items_center()
                        .margin_left(8.0)
                }),
                // Commits count badge
                label(move || format!("{} commits", commits.get().len())).style(move |s| {
                    s.font_size(10.0)
                        .color(theme.get().fg_muted)
                        .margin_left(8.0)
                }),
            ))
            .style(|s| s.items_center()),

            // Actions
            h_stack((
                // Refresh Button
                container(
                    h_stack((
                        render_svg(ICON_REFRESH, 12.0),
                        label(move || "Refresh".to_string()).style(move |s| {
                            s.font_size(10.0)
                                .color(theme.get().fg_primary)
                                .margin_left(4.0)
                        }),
                    ))
                    .style(|s| s.items_center()),
                )
                .on_click_stop(move |_| {
                    state.refresh_git();
                    state.status_message.set("OmniGit: Status & graph refreshed".to_string());
                })
                .style(move |s| {
                    s.padding_horiz(6.0)
                        .height(20.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
                        .background(palette::css::TRANSPARENT)
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().hover_bg))
                }),
            ))
            .style(|s| s.items_center()),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(28.0)
                .padding_horiz(10.0)
                .items_center()
                .justify_between()
                .border_bottom(1.0)
                .border_color(theme.get().border)
                .background(theme.get().bg_activity)
        }),

        // Column Titles header for wide view
        h_stack((
            label(move || "GRAPH".to_string()).style(move |s| {
                s.font_size(9.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_muted)
                    .width(60.0)
                    .flex_shrink(0.0)
            }),
            label(move || "HASH".to_string()).style(move |s| {
                s.font_size(9.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_muted)
                    .width(65.0)
                    .flex_shrink(0.0)
            }),
            label(move || "COMMIT MESSAGE".to_string()).style(move |s| {
                s.font_size(9.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_muted)
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
            }),
            label(move || "AUTHOR".to_string()).style(move |s| {
                s.font_size(9.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_muted)
                    .width(130.0)
                    .flex_shrink(0.0)
                    .margin_left(12.0)
            }),
            label(move || "DATE".to_string()).style(move |s| {
                s.font_size(9.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_muted)
                    .width(100.0)
                    .flex_shrink(0.0)
                    .margin_left(8.0)
            }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(20.0)
                .padding_horiz(10.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(theme.get().border)
                .background(theme.get().bg_sidebar)
        }),

        // Commits List
        scroll(
            dyn_container(
                move || commits.get(),
                move |commit_list| {
                    if commit_list.is_empty() {
                        return container(
                            label(move || "No commits in repository yet".to_string()).style(move |s| {
                                s.font_size(11.5)
                                    .color(theme.get().fg_muted)
                                    .padding(16.0)
                            }),
                        )
                        .style(|s| s.width_pct(100.0))
                        .into_any();
                    }

                    let nodes = GitExecutor::compute_graph_nodes(&commit_list);
                    let items = nodes.into_iter().enumerate().map(move |(idx, node)| {
                        render_commit_graph_row(state, idx, node, selected_commit, true)
                    });

                    v_stack_from_iter(items)
                        .style(|s| s.flex_col().width_pct(100.0))
                        .into_any()
                },
            )
            .style(|s| s.width_pct(100.0)),
        )
        .style(|s| s.flex_grow(1.0).width_pct(100.0)),
    ))
    .style(move |s| {
        s.size_pct(100.0, 100.0)
            .background(theme.get().bg_panel)
    })
}
