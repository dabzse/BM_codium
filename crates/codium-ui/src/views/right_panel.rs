//! # Secondary Right Panel (Manuals & Doc Tutor Assistant)
//!
//! Provides the right-side documentation assistant:
//! - Offline manuals directory browser (PHP, Python, Django, MySQL, C, Rust)
//! - Language and Framework / Module filtering
//! - Local offline tutor chat
//! - University policy enforcement (can be restricted or disabled by instructors during exams)

use std::path::Path;
use floem::prelude::*;
use floem::peniko::Brush;
use floem::style::CursorStyle;
use floem::text::Weight;
use floem::taffy::style::Display;
use codium_core::i18n::Locale;
use crate::state::AppState;

pub fn right_panel_view(state: AppState) -> impl IntoView {
    let theme = state.theme;
    let show_panel = state.show_right_panel;
    let locale = state.locale;
    let policy = state.edu_policy;
    let settings = state.settings;
    let lang_query = state.manuals_lang_query;
    let mod_query = state.manuals_module_query;
    let chat_query = state.manuals_chat_query;
    let chat_msgs = state.manuals_chat_messages;

    v_stack((
        // Panel Header
        h_stack((
            label(move || locale.get().tr("MANUALS & DOC TUTOR")).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().accent_edu)
            }),
            container(
                label(move || "×").style(move |s| {
                    s.font_size(14.0).color(theme.get().fg_muted)
                }),
            )
            .on_click_stop(move |_| {
                show_panel.set(false);
            })
            .style(move |s| {
                s.cursor(CursorStyle::Pointer)
                    .hover(|s| s.color(theme.get().fg_primary))
            }),
        ))
        .style(move |s| {
            s.width_pct(100.0)
                .height(32.0)
                .padding_horiz(12.0)
                .justify_between()
                .items_center()
                .border_bottom(1.0)
                .border_color(theme.get().border)
        }),

        // Subtitle description
        label(move || locale.get().tr("Offline language & framework documentation assistant")).style(move |s| {
            s.font_size(10.5)
                .color(theme.get().fg_muted)
                .margin_horiz(12.0)
                .margin_top(8.0)
                .margin_bottom(6.0)
        }),

        // Exam Restriction Notice (if restricted by teacher policy)
        dyn_container(
            move || (policy.get().exam_mode, policy.get().allow_manuals_access),
            move |(exam_mode, allow_manuals)| {
                if exam_mode && !allow_manuals {
                    container(
                        label(move || locale.get().tr("Manuals access is restricted in exam mode")).style(move |s| {
                            s.font_size(11.0)
                                .color(palette::css::WHITE)
                                .font_weight(Weight::BOLD)
                        }),
                    )
                    .style(move |s| {
                        s.background(palette::css::CRIMSON)
                            .padding(8.0)
                            .margin(10.0)
                            .border_radius(4.0)
                    })
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),

        // Language & Framework filters
        v_stack((
            label(move || locale.get().tr("Language (e.g. python, php, c):")).style(move |s| {
                s.font_size(10.5).color(theme.get().fg_muted).margin_bottom(2.0)
            }),
            text_input(lang_query).style(move |s| {
                s.width_pct(100.0)
                    .height(26.0)
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .padding_horiz(8.0)
                    .color(theme.get().fg_primary)
                    .cursor_color(Brush::Solid(palette::css::WHITE))
                    .margin_bottom(8.0)
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

            label(move || locale.get().tr("Framework / Module (e.g. django):")).style(move |s| {
                s.font_size(10.5).color(theme.get().fg_muted).margin_bottom(2.0)
            }),
            text_input(mod_query).style(move |s| {
                s.width_pct(100.0)
                    .height(26.0)
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .padding_horiz(8.0)
                    .color(theme.get().fg_primary)
                    .cursor_color(Brush::Solid(palette::css::WHITE))
                    .margin_bottom(10.0)
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
        ))
        .style(|s| s.flex_col().padding_horiz(12.0)),

        // Scanned Manuals Section
        v_stack((
            label(move || locale.get().tr("Detected Manuals:")).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().fg_primary)
                    .margin_bottom(4.0)
            }),
            scroll(
                dyn_container(
                    move || (lang_query.get(), mod_query.get(), settings.get().manuals_dir.clone(), locale.get()),
                    move |(lang, module, _dir, loc)| {
                        let manuals_root = settings.get().resolve_manuals_dir();
                        let items = scan_manuals(&manuals_root, &lang, &module, loc);

                        v_stack_from_iter(
                            items.into_iter().map(|item| {
                                let name = item.clone();
                                h_stack((
                                    label(move || format!("📚 {}", name)).style(move |s| {
                                        s.font_size(11.0)
                                            .color(theme.get().fg_primary)
                                    }),
                                ))
                                .style(move |s| {
                                    s.padding_vert(3.0)
                                        .padding_horiz(6.0)
                                        .cursor(CursorStyle::Pointer)
                                        .hover(|s| s.background(theme.get().hover_bg))
                                        .border_radius(3.0)
                                })
                            })
                        )
                        .style(|s| s.flex_col())
                    },
                )
            )
            .style(move |s| {
                s.height(100.0)
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .padding(6.0)
            }),
        ))
        .style(|s| s.padding_horiz(12.0).margin_bottom(12.0)),

        // Local Manual Tutor Chat
        v_stack((
            label(move || format!("💬 {}", locale.get().tr("Local Manual Tutor"))).style(move |s| {
                s.font_size(11.0)
                    .font_weight(Weight::BOLD)
                    .color(theme.get().accent_edu)
                    .margin_bottom(4.0)
            }),
            // Messages container
            scroll(
                dyn_container(
                    move || chat_msgs.get(),
                    move |messages| {
                        v_stack_from_iter(
                            messages.into_iter().map(|(sender, text)| {
                                let is_tutor = sender == "Tutor";
                                v_stack((
                                    label(move || sender.clone()).style(move |s| {
                                        s.font_size(10.0)
                                            .font_weight(Weight::BOLD)
                                            .color(if is_tutor { theme.get().accent_edu } else { theme.get().accent_blue })
                                    }),
                                    label(move || text.clone()).style(move |s| {
                                        s.font_size(11.0)
                                            .color(theme.get().fg_primary)
                                            .margin_top(2.0)
                                    }),
                                ))
                                .style(move |s| {
                                    s.padding(6.0)
                                        .margin_bottom(6.0)
                                        .background(theme.get().hover_bg)
                                        .border_radius(4.0)
                                })
                            })
                        )
                        .style(|s| s.flex_col())
                    },
                )
            )
            .style(move |s| {
                s.flex_grow(1.0)
                    .background(theme.get().bg_editor)
                    .border(1.0)
                    .border_color(theme.get().border)
                    .border_radius(4.0)
                    .padding(6.0)
            }),

            // Multi-line Textarea Style Chat Input + Send Button
            v_stack((
                text_input(chat_query)
                    .on_enter(move || {
                        let q = chat_query.get().trim().to_string();
                        if !q.is_empty() {
                            let answer = generate_offline_manual_answer(&q, &lang_query.get(), &mod_query.get(), locale.get());
                            chat_msgs.update(|msgs| {
                                msgs.push(("Student".to_string(), q));
                                msgs.push(("Tutor".to_string(), answer));
                            });
                            chat_query.set(String::new());
                        }
                    })
                    .style(move |s| {
                        let len = chat_query.get().len();
                        let height = if len > 60 {
                            80.0
                        } else if len > 30 {
                            64.0
                        } else {
                            52.0
                        };
                        s.width_pct(100.0)
                            .height(height)
                            .background(theme.get().bg_editor)
                            .border(1.0)
                            .border_color(theme.get().border)
                            .border_radius(4.0)
                            .padding(8.0)
                            .color(theme.get().fg_primary)
                            .cursor_color(Brush::Solid(palette::css::WHITE))
                            .font_size(12.0)
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
                h_stack((
                    label(move || locale.get().tr("Press Enter to send")).style(move |s| {
                        s.font_size(10.0)
                            .color(theme.get().fg_muted)
                    }),
                    empty().style(|s| s.flex_grow(1.0)),
                    container(
                        label(move || locale.get().tr("Send")).style(move |s| {
                            s.font_size(11.0)
                                .font_weight(Weight::BOLD)
                                .color(palette::css::WHITE)
                        }),
                    )
                    .on_click_stop(move |_| {
                        let q = chat_query.get().trim().to_string();
                        if !q.is_empty() {
                            let answer = generate_offline_manual_answer(&q, &lang_query.get(), &mod_query.get(), locale.get());
                            chat_msgs.update(|msgs| {
                                msgs.push(("Student".to_string(), q));
                                msgs.push(("Tutor".to_string(), answer));
                            });
                            chat_query.set(String::new());
                        }
                    })
                    .style(move |s| {
                        s.background(theme.get().accent_edu)
                            .padding_horiz(12.0)
                            .height(26.0)
                            .items_center()
                            .border_radius(4.0)
                            .cursor(CursorStyle::Pointer)
                            .hover(move |s| s.background(theme.get().accent_edu.with_alpha(0.85)))
                    }),
                ))
                .style(|s| s.items_center().width_pct(100.0).margin_top(6.0)),
            ))
            .style(|s| s.margin_top(8.0)),
        ))
        .style(|s| s.flex_col().flex_grow(1.0).padding_horiz(12.0).margin_bottom(12.0)),
    ))
    .style(move |s| {
        s.width(state.right_panel_width.get())
            .height_pct(100.0)
            .background(theme.get().bg_sidebar)
            .display(if show_panel.get() { Display::Flex } else { Display::None })
    })
}

/// Recursively scans manuals_dir and returns matched relative paths
fn scan_manuals(root: &Path, lang_filter: &str, mod_filter: &str, locale: Locale) -> Vec<String> {
    let mut results = Vec::new();
    if !root.exists() {
        return vec![locale.tr("(Manuals directory empty or not found)")];
    }

    let lang_l = lang_filter.trim().to_lowercase();
    let mod_l = mod_filter.trim().to_lowercase();

    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name.starts_with('.') || name == "README.md" {
                continue;
            }

            if !lang_l.is_empty() && !name.to_lowercase().contains(&lang_l) {
                continue;
            }

            if path.is_dir() {
                // Check submodules
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    let mut found_sub = false;
                    for sub in sub_entries.flatten() {
                        let sub_p = sub.path();
                        let sub_name = sub_p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if !mod_l.is_empty() && !sub_name.to_lowercase().contains(&mod_l) {
                            continue;
                        }
                        results.push(format!("{}/{}", name, sub_name));
                        found_sub = true;
                    }
                    if !found_sub && mod_l.is_empty() {
                        results.push(format!("{}/", name));
                    }
                } else {
                    results.push(format!("{}/", name));
                }
            } else {
                results.push(name);
            }
        }
    }

    if results.is_empty() {
        results.push(locale.tr("No matching manuals found."));
    }

    results
}

/// Offline pedagogic helper answering student manual questions without external API
fn generate_offline_manual_answer(question: &str, lang: &str, module: &str, locale: Locale) -> String {
    let q = question.to_lowercase();
    let target = if !module.is_empty() {
        format!("{}/{}", lang, module)
    } else if !lang.is_empty() {
        lang.to_string()
    } else {
        locale.tr("Manual")
    };

    if q.contains("django") || q.contains("orm") || q.contains("filter") {
        return locale.tr("Django ORM: Use Model.objects.filter(...) queries. Details in manuals/python/django/ directory.");
    }
    if q.contains("printf") || q.contains("scanf") || q.contains("io") {
        return locale.tr("C I/O: printf formats (%d int, %f float, %s string). Scanf needs pointers: scanf(\"%d\", &val). Details in manuals/c/.");
    }
    if q.contains("borrow") || q.contains("ownership") {
        return locale.tr("Rust memory: Either any number of immutable references (&T) or exactly one mutable reference (&mut T) can exist at a time. See manuals/rust/.");
    }
    if q.contains("php") || q.contains("array") || q.contains("pdo") {
        return locale.tr("PHP manual: PDO database connection: new PDO(...). Arrays: associative and indexed. See manuals/php/.");
    }

    let template = locale.tr("Information for [{target}] topic: Search downloaded files in manuals/ directory or refine your query keyword!");
    template.replace("{target}", &target)
}
