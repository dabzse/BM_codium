//! # Editor Area View
//!
//! Renders the tab bar of open documents and the active text editor canvas,
//! or a rich welcome screen with quick-action shortcuts when no files are open.

use floem::event::{Event, EventListener, EventPropagation};
use floem::prelude::*;
use floem::reactive::create_memo;
use floem::style::CursorStyle;
use floem::text::Weight;
use crate::state::AppState;

/// Main editor canvas container
pub fn editor_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let ws = state.workspace;

    // Memoize active document key so editor view is ONLY rebuilt when active tab/file changes,
    // and NEVER on every keystroke!
    let active_doc_key = create_memo(move |_| {
        let w = ws.get();
        (
            w.active_document_idx,
            w.active_document().and_then(|d| d.path().map(|p| p.to_path_buf())),
        )
    });

    v_stack((
        // Tab Bar of open files
        tab_bar_view(state),

        // Editor Canvas or Welcome Screen
        dyn_container(
            move || active_doc_key.get(),
            move |key| {
                if key.0.is_some() {
                    active_editor_view(state).into_any()
                } else {
                    welcome_view(state).into_any()
                }
            },
        )
        .style(move |s| {
            s.flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .width_pct(100.0)
                .background(theme.get().bg_editor)
        }),
    ))
    .style(|s| s.flex_col().flex_grow(1.0).height_pct(100.0))
}

/// Horizontal tab bar rendering tabs for each open document
fn tab_bar_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let ws = state.workspace;
    let word_wrap = state.word_wrap;
    let locale = state.locale;

    let tab_meta = create_memo(move |_| {
        let w = ws.get();
        let tabs: Vec<(String, bool)> = w
            .open_documents
            .iter()
            .map(|d| (d.file_name(), d.is_dirty()))
            .collect();
        (tabs, w.active_document_idx)
    });

    h_stack((
        scroll(
            dyn_container(
                move || tab_meta.get(),
                move |(tabs_info, active_idx)| {
                    let mut tabs = Vec::new();
                    for (idx, (title, is_dirty)) in tabs_info.into_iter().enumerate() {
                        let is_active = active_idx == Some(idx);
                        let ws_signal = ws;

                        let tab = h_stack((
                            label(move || title.clone()).style(move |s| {
                                s.font_size(12.0)
                                    .color(if is_active {
                                        theme.get().fg_primary
                                    } else {
                                        theme.get().fg_muted
                                    })
                            }),
                            if is_dirty {
                                label(move || " ●").style(move |s| {
                                    s.font_size(10.0).color(theme.get().accent_blue)
                                }).into_any()
                            } else {
                                empty().into_any()
                            },
                            container(
                                label(move || " ×").style(move |s| {
                                    s.font_size(13.0).color(theme.get().fg_muted)
                                }),
                            )
                            .on_event_stop(EventListener::PointerDown, move |_| {})
                            .on_click_stop(move |_| {
                                ws_signal.update(|w| w.close_tab(idx));
                            })
                            .style(move |s| {
                                s.margin_left(8.0)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(|s| s.color(theme.get().fg_primary))
                            }),
                        ))
                        .on_event_stop(EventListener::PointerDown, move |event| {
                            if let Event::PointerDown(pe) = event {
                                if pe.button.is_auxiliary() {
                                    ws_signal.update(|w| w.close_tab(idx));
                                } else if pe.button.is_primary() {
                                    ws_signal.update(|w| w.active_document_idx = Some(idx));
                                }
                            }
                        })
                        .style(move |s| {
                            s.height(35.0)
                                .items_center()
                                .padding_horiz(12.0)
                                .background(if is_active {
                                    theme.get().bg_tab_active
                                } else {
                                    theme.get().bg_tab_inactive
                                })
                                .border_right(1.0)
                                .border_color(theme.get().border)
                                .apply_if(is_active, |s| {
                                    s.border_top(2.0).border_color(theme.get().accent_blue)
                                })
                                .cursor(CursorStyle::Pointer)
                                .hover(|s| {
                                    if !is_active {
                                        s.background(theme.get().hover_bg)
                                    } else {
                                        s
                                    }
                                })
                        });

                        tabs.push(tab);
                    }

                    h_stack_from_iter(tabs).style(|s| s.flex_row().items_center())
                },
            )
            .style(|s| s.flex_row().height(35.0)),
        )
        .style(move |s| {
            s.flex_grow(1.0)
                .flex_basis(0.0)
                .min_width(0.0)
                .height(35.0)
        }),

        // Right side tools: Word wrap toggle button
        dyn_container(
            move || (word_wrap.get(), locale.get()),
            move |(is_wrap, loc)| {
                let wrap_label = if is_wrap {
                    loc.tr("↩ Word Wrap: ON")
                } else {
                    loc.tr("↩ Word Wrap (Alt+Z)")
                };

                container(
                    label(move || wrap_label.clone()).style(move |s| {
                        s.font_size(11.0)
                            .color(if is_wrap { theme.get().accent_blue } else { theme.get().fg_muted })
                    })
                )
                .on_click_stop(move |_| {
                    word_wrap.update(|w| *w = !*w);
                })
                .style(move |s| {
                    s.height(24.0)
                        .items_center()
                        .padding_horiz(8.0)
                        .margin_right(8.0)
                        .border(1.0)
                        .border_radius(4.0)
                        .border_color(if is_wrap { theme.get().accent_blue } else { theme.get().border })
                        .background(if is_wrap { theme.get().hover_bg } else { palette::css::TRANSPARENT })
                        .cursor(CursorStyle::Pointer)
                        .hover(move |s| s.background(theme.get().hover_bg).border_color(theme.get().accent_blue))
                })
                .into_any()
            }
        )
        .style(|s| s.items_center().height(35.0)),
    ))
    .style(move |s| {
        s.width_pct(100.0)
            .height(35.0)
            .background(theme.get().bg_tab_inactive)
            .border_bottom(1.0)
            .border_color(theme.get().border)
            .items_center()
    })
}

/// Active text buffer view powered by Floem's native TextEditor engine
use floem::views::text_editor::text_editor;
use floem::views::editor::text::WrapMethod;

fn active_editor_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let ws = state.workspace;
    let word_wrap = state.word_wrap;
    let settings = state.settings;

    let active_idx = ws.get_untracked().active_document_idx;
    let initial_text = if let Some(idx) = active_idx {
        ws.get_untracked()
            .open_documents
            .get(idx)
            .map(|d| d.text())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let ed = text_editor(initial_text);
    let doc_ref = ed.doc();
    let ws_signal = ws;

    // Sync edited text back into the workspace TextBuffer
    let ed = ed.update(move |_| {
        let current_text = doc_ref.text().to_string();
        if let Some(idx) = ws_signal.get_untracked().active_document_idx {
            ws_signal.update(|w| {
                if let Some(d) = w.open_documents.get_mut(idx) {
                    d.set_text(&current_text);
                }
            });
        }
    });

    let ed = ed.editor_style(move |s| {
        let t = theme.get();
        let wrap = if word_wrap.get() {
            WrapMethod::EditorWidth
        } else {
            WrapMethod::None
        };
        s.cursor_color(t.fg_primary)
            .current_line_color(t.hover_bg)
            .gutter_accent_color(match state.edition {
                crate::state::CodiumEdition::Edu => t.accent_edu,
                crate::state::CodiumEdition::Ai => t.accent_ai,
            })
            .gutter_dim_color(t.fg_muted)
            .wrap_method(wrap)
            .scroll_beyond_last_line(true)
            .indent_guide(true)
            .indent_guide_color(t.border)
    });

    ed.on_event(EventListener::KeyDown, move |event| {
        let keymap = state.keymap.get();
        if let Some(cmd) = crate::keymap::match_keymap(event, &keymap) {
            crate::keymap::execute_command(cmd, state);
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    })
    .style(move |s| {
        s.size_pct(100.0, 100.0)
            .flex_grow(1.0)
            .min_height(0.0)
            .min_width(0.0)
            .font_family(settings.get().editor_font_family)
            .font_size(13.5)
            .color(theme.get().fg_primary)
            .background(theme.get().bg_editor)
    })
}

/// Welcome screen displayed when no file tabs are currently open
fn welcome_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let edition = state.edition;
    let locale = state.locale;

    v_stack((
        label(move || edition.title()).style(move |s| {
            s.font_size(32.0)
                .font_weight(Weight::BOLD)
                .color(match edition {
                    crate::state::CodiumEdition::Edu => theme.get().accent_edu,
                    crate::state::CodiumEdition::Ai => theme.get().accent_ai,
                })
        }),
        label(move || edition.subtitle(locale.get())).style(move |s| {
            s.font_size(14.0)
                .color(theme.get().fg_muted)
                .margin_top(8.0)
                .margin_bottom(24.0)
        }),
        v_stack((
            welcome_shortcut("Shift + Shift", move || locale.get().shortcut_search_desc(), state),
            welcome_shortcut("Ctrl + B", move || locale.get().shortcut_sidebar_desc(), state),
            welcome_shortcut("Ctrl + J", move || locale.get().shortcut_terminal_desc(), state),
            welcome_shortcut("▶ Run", move || locale.get().shortcut_run_desc(), state),
            welcome_shortcut("F1", move || locale.get().shortcut_help_desc(), state),
        ))
        .style(|s| s.flex_col().items_start()),
    ))
    .style(move |s| {
        s.size_pct(100.0, 100.0)
            .items_center()
            .justify_center()
            .background(theme.get().bg_editor)
    })
}

fn welcome_shortcut(keys: &'static str, desc: impl Fn() -> String + 'static, state: AppState) -> impl IntoView {
    let theme = state.theme;
    h_stack((
        container(
            label(move || keys).style(move |s| {
                s.font_size(11.5)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
            }),
        )
        .style(move |s| {
            s.background(theme.get().hover_bg)
                .padding_horiz(8.0)
                .padding_vert(3.0)
                .border_radius(4.0)
                .width(110.0)
                .items_center()
                .justify_center()
        }),
        label(move || desc()).style(move |s| {
            s.font_size(12.5)
                .color(theme.get().fg_muted)
                .margin_left(12.0)
        }),
    ))
    .style(|s| s.items_center().margin_bottom(10.0))
}
