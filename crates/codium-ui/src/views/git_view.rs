//! # OmniGit View for BM_codium
//!
//! Visual Source Control and Git Graph integration powered by `dabzse/omnigit`.
//! Provides repository status tracking, staging/unstaging, commit box,
//! and commit history visualization.

use codium_core::{format_timestamp_relative, GitExecutor};
use floem::peniko::Brush;
use floem::prelude::*;
use floem::style::CursorStyle;
use floem::text::Weight;
use std::path::PathBuf;

use crate::icons::{
    render_svg, ICON_CHECK, ICON_GIT_BRANCH, ICON_HISTORY, ICON_MINUS, ICON_PLUS, ICON_REFRESH,
    ICON_SOURCE_CONTROL,
};
use crate::state::AppState;

/// Main Source Control (OmniGit) view in sidebar
pub fn git_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let locale = state.locale;
    let git_status = state.git_status;
    let commit_msg = state.git_commit_message;

    let get_cwd = || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

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
                                    .color(palette::css::WHITE)
                            }),
                        )
                        .on_click_stop(move |_| {
                            let cwd = get_cwd();
                            let _ = GitExecutor::exec(&cwd, &["init"]);
                            state.refresh_git();
                        })
                        .style(move |s| {
                            s.background(theme.get().accent_blue)
                                .padding_horiz(12.0)
                                .height(26.0)
                                .items_center()
                                .border_radius(3.0)
                                .cursor(CursorStyle::Pointer)
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
                                trigger_commit(state, get_cwd());
                            })
                            .style(move |s| {
                                s.width_pct(100.0)
                                    .height(28.0)
                                    .background(theme.get().bg_editor)
                                    .border(1.0)
                                    .border_color(theme.get().border)
                                    .border_radius(3.0)
                                    .padding_horiz(6.0)
                                    .color(theme.get().fg_primary)
                                    .cursor_color(Brush::Solid(palette::css::WHITE))
                                    .font_size(11.0)
                            }),
                        // Commit Button
                        container(
                            h_stack((
                                render_svg(ICON_CHECK, 13.0),
                                label(move || locale.get().tr("Commit")).style(move |s| {
                                    s.font_size(11.0)
                                        .font_weight(Weight::BOLD)
                                        .color(palette::css::WHITE)
                                        .margin_left(6.0)
                                }),
                            ))
                            .style(|s| s.items_center().justify_center()),
                        )
                        .on_click_stop(move |_| {
                            trigger_commit(state, get_cwd());
                        })
                        .style(move |s| {
                            s.width_pct(100.0)
                                .height(26.0)
                                .margin_top(6.0)
                                .background(theme.get().accent_blue)
                                .items_center()
                                .justify_center()
                                .border_radius(3.0)
                                .cursor(CursorStyle::Pointer)
                                .hover(move |s| s.background(theme.get().accent_ai))
                        }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .padding(10.0)
                            .border_bottom(1.0)
                            .border_color(theme.get().border)
                    }),

                    // Scrollable File Changes & History
                    scroll(
                        v_stack((
                            // Staged Changes Section
                            render_staged_section(state),
                            // Changes (Unstaged + Untracked) Section
                            render_unstaged_section(state),
                            // Recent Commits (Graph) Section
                            render_commits_section(state),
                        ))
                        .style(|s| s.width_pct(100.0)),
                    )
                    .style(|s| s.flex_grow(1.0).width_pct(100.0)),
                ))
                .style(|s| s.flex_grow(1.0).width_pct(100.0))
                .into_any()
            },
        )
        .style(|s| s.flex_grow(1.0).width_pct(100.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .height_pct(100.0)
            .background(theme.get().bg_sidebar)
    })
}

fn trigger_commit(state: AppState, cwd: PathBuf) {
    let msg = state.git_commit_message.get().trim().to_string();
    if msg.is_empty() {
        state.status_message.set("OmniGit: Please enter a commit message".to_string());
        return;
    }

    let status = state.git_status.get();
    // If nothing staged, but there are unstaged changes, stage all first (like VS Code prompt)
    if status.staged.is_empty() && (status.unstaged.len() + status.untracked.len()) > 0 {
        let _ = GitExecutor::stage_all(&cwd);
    }

    match GitExecutor::commit(&cwd, &msg) {
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

/// Staged Changes header and list
fn render_staged_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let git_status = state.git_status;
    let get_cwd = || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    v_stack((
        // Section Header
        h_stack((
            h_stack((
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

            // Unstage All action
            container(render_svg(ICON_MINUS, 12.0))
                .on_click_stop(move |_| {
                    let _ = GitExecutor::unstage_all(&get_cwd());
                    state.refresh_git();
                })
                .style(move |s| {
                    s.size(18.0, 18.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
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

        // Staged Files list
        dyn_container(
            move || git_status.get().staged,
            move |staged_files| {
                let items = staged_files.into_iter().map(move |file| {
                    let file_path = file.path.clone();
                    let unstage_path = file.path.clone();
                    let st = file.status;

                    h_stack((
                        // Status badge (M, A, D, R, etc.)
                        label(move || st.badge().to_string()).style(move |s| {
                            s.font_size(10.0)
                                .font_weight(Weight::BOLD)
                                .color(palette::css::WHITE)
                                .margin_right(6.0)
                        }),
                        // File path
                        label(move || file_path.clone()).style(move |s| {
                            s.font_size(11.0)
                                .color(theme.get().fg_primary)
                                .flex_grow(1.0)
                        }),
                        // Unstage button (-)
                        container(render_svg(ICON_MINUS, 11.0))
                            .on_click_stop(move |_| {
                                let _ = GitExecutor::unstage_file(&get_cwd(), &unstage_path);
                                state.refresh_git();
                            })
                            .style(move |s| {
                                s.size(16.0, 16.0)
                                    .items_center()
                                    .justify_center()
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().hover_bg))
                            }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .height(22.0)
                            .padding_horiz(12.0)
                            .items_center()
                            .hover(move |s| s.background(theme.get().hover_bg))
                    })
                });

                v_stack((items.collect::<Vec<_>>(),)).style(|s| s.width_pct(100.0)).into_any()
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}

/// Changes (Unstaged + Untracked) header and list
fn render_unstaged_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let git_status = state.git_status;
    let get_cwd = || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    v_stack((
        // Section Header
        h_stack((
            h_stack((
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

            // Stage All action (+)
            container(render_svg(ICON_PLUS, 12.0))
                .on_click_stop(move |_| {
                    let _ = GitExecutor::stage_all(&get_cwd());
                    state.refresh_git();
                })
                .style(move |s| {
                    s.size(18.0, 18.0)
                        .items_center()
                        .justify_center()
                        .border_radius(3.0)
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

        // Changes Files list
        dyn_container(
            move || {
                let st = git_status.get();
                let mut all = st.unstaged;
                all.extend(st.untracked);
                all
            },
            move |files| {
                let items = files.into_iter().map(move |file| {
                    let file_path = file.path.clone();
                    let stage_path = file.path.clone();
                    let st = file.status;

                    h_stack((
                        // Status badge (M, D, U, etc.)
                        label(move || st.badge().to_string()).style(move |s| {
                            s.font_size(10.0)
                                .font_weight(Weight::BOLD)
                                .color(palette::css::WHITE)
                                .margin_right(6.0)
                        }),
                        // File path
                        label(move || file_path.clone()).style(move |s| {
                            s.font_size(11.0)
                                .color(theme.get().fg_primary)
                                .flex_grow(1.0)
                        }),
                        // Stage button (+)
                        container(render_svg(ICON_PLUS, 11.0))
                            .on_click_stop(move |_| {
                                let _ = GitExecutor::stage_file(&get_cwd(), &stage_path);
                                state.refresh_git();
                            })
                            .style(move |s| {
                                s.size(16.0, 16.0)
                                    .items_center()
                                    .justify_center()
                                    .cursor(CursorStyle::Pointer)
                                    .hover(move |s| s.background(theme.get().hover_bg))
                            }),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .height(22.0)
                            .padding_horiz(12.0)
                            .items_center()
                            .hover(move |s| s.background(theme.get().hover_bg))
                    })
                });

                v_stack((items.collect::<Vec<_>>(),)).style(|s| s.width_pct(100.0)).into_any()
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}

/// OmniGit Commits / Graph section
fn render_commits_section(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let commits = state.git_commits;

    v_stack((
        // Section Header
        h_stack((
            render_svg(ICON_HISTORY, 12.0),
            label(move || "COMMITS / GRAPH".to_string()).style(move |s| {
                s.font_size(10.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
                    .margin_left(6.0)
            }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(24.0)
                .padding_horiz(10.0)
                .items_center()
                .background(theme.get().bg_activity)
                .margin_top(8.0)
        }),

        // Commits List
        dyn_container(
            move || commits.get(),
            move |commit_list| {
                let items = commit_list.into_iter().map(move |c| {
                    let short_hash = c.short_hash.clone();
                    let subject = c.subject.clone();
                    let author = c.author_name.clone();
                    let time = format_timestamp_relative(c.author_date);

                    v_stack((
                        h_stack((
                            // Graph lane dot
                            container(empty()).style(move |s| {
                                s.size(6.0, 6.0)
                                    .border_radius(3.0)
                                    .background(theme.get().accent_blue)
                                    .margin_right(6.0)
                            }),
                            // Short Hash
                            label(move || short_hash.clone()).style(move |s| {
                                s.font_size(10.0)
                                    .font_weight(Weight::BOLD)
                                    .color(theme.get().accent_blue)
                                    .margin_right(6.0)
                            }),
                            // Subject
                            label(move || subject.clone()).style(move |s| {
                                s.font_size(10.5)
                                    .color(theme.get().fg_primary)
                                    .flex_grow(1.0)
                            }),
                        ))
                        .style(|s| s.width_pct(100.0).items_center()),

                        // Author & Relative Date row
                        h_stack((
                            label(move || format!("{} • {}", author, time)).style(move |s| {
                                s.font_size(9.5)
                                    .color(theme.get().fg_muted)
                                    .margin_left(12.0)
                            }),
                        ))
                        .style(|s| s.margin_top(2.0)),
                    ))
                    .style(move |s| {
                        s.width_pct(100.0)
                            .padding_horiz(10.0)
                            .padding_vert(4.0)
                            .border_bottom(1.0)
                            .border_color(theme.get().border)
                            .hover(move |s| s.background(theme.get().hover_bg))
                    })
                });

                v_stack((items.collect::<Vec<_>>(),)).style(|s| s.width_pct(100.0)).into_any()
            },
        )
        .style(|s| s.width_pct(100.0)),
    ))
    .style(|s| s.width_pct(100.0))
}
