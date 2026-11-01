//! # Keymap Event Dispatcher (VSCodium Compatible)
//!
//! Maps Floem KeyEvent to VSCodium Commands and executes them across AppState.
//! Handles modifier normalization (including AltGr and Ctrl+Alt on Linux/Windows)
//! and physical key layout tolerance (e.g. Hungarian QWERTZ swap for Z and Y).

use codium_core::keymap::{Command, KeyChord, Keymap};
use floem::event::Event;
use floem::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use floem::prelude::*;
use crate::state::{AppState, BottomTab, SidebarTab};

/// Checks whether an incoming Floem KeyEvent matches a KeyChord
pub fn matches_chord(event: &floem::keyboard::KeyEvent, chord: &KeyChord) -> bool {
    let mods = event.modifiers;

    // Alt modifier handling (includes AltGr and Windows/Linux Ctrl+Alt)
    let alt_active = mods.alt() || mods.altgr() || (mods.control() && mods.alt());
    if chord.alt != alt_active {
        return false;
    }

    // Ctrl modifier handling
    // When AltGr is pressed, some systems report both Ctrl and Alt.
    // If the chord requires Alt but not Ctrl, we ignore Ctrl when alt_active is true.
    let ctrl_active = if chord.alt && !chord.ctrl {
        false
    } else {
        mods.control()
    };
    if chord.ctrl != ctrl_active {
        return false;
    }

    if chord.shift != mods.shift() {
        return false;
    }

    if chord.meta != mods.meta() {
        return false;
    }

    let chord_key = chord.key.to_lowercase();

    // 1. Match logical key
    let logical_match = match &event.key.logical_key {
        Key::Character(c) => c.eq_ignore_ascii_case(&chord_key),
        Key::Named(NamedKey::F1) => chord_key == "f1",
        Key::Named(NamedKey::F4) => chord_key == "f4",
        Key::Named(NamedKey::F5) => chord_key == "f5",
        Key::Named(NamedKey::F12) => chord_key == "f12",
        _ => false,
    };

    if logical_match {
        return true;
    }

    // 2. Match physical key code (solves layout swaps like QWERTY vs QWERTZ for Z and Y)
    match chord_key.as_str() {
        "z" => matches!(
            event.key.physical_key,
            PhysicalKey::Code(KeyCode::KeyZ) | PhysicalKey::Code(KeyCode::KeyY)
        ),
        "y" => matches!(
            event.key.physical_key,
            PhysicalKey::Code(KeyCode::KeyY) | PhysicalKey::Code(KeyCode::KeyZ)
        ),
        "s" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyS)),
        "n" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyN)),
        "o" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyO)),
        "w" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyW)),
        "b" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyB)),
        "j" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyJ)),
        "f" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::KeyF)),
        "f1" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::F1)),
        "f4" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::F4)),
        "f5" => matches!(event.key.physical_key, PhysicalKey::Code(KeyCode::F5)),
        _ => false,
    }
}

/// Matches a Floem Event against the active Keymap
pub fn match_keymap(event: &Event, keymap: &Keymap) -> Option<Command> {
    if let Event::KeyDown(ke) = event {
        for binding in &keymap.bindings {
            if let Some(chord) = KeyChord::parse(&binding.key) {
                if matches_chord(ke, &chord) {
                    return Some(binding.command);
                }
            }
        }
    }
    None
}

/// Dispatches a recognized IDE command to AppState
pub fn execute_command(cmd: Command, state: AppState) {
    let ws = state.workspace;
    let locale = state.locale;
    let status_message = state.status_message;

    match cmd {
        Command::ToggleWordWrap => {
            state.word_wrap.update(|w| *w = !*w);
            let active = state.word_wrap.get();
            let msg = if active {
                locale.get().tr("Word Wrap enabled (Alt+Z)")
            } else {
                locale.get().tr("Word Wrap disabled (Alt+Z)")
            };
            status_message.set(msg);
        }
        Command::SaveFile => {
            ws.update(|w| {
                let _ = w.save_active_document();
            });
            status_message.set(locale.get().tr("File saved."));
        }
        Command::NewFile => {
            ws.update(|w| {
                let buf = codium_core::buffer::TextBuffer::from_str("", None);
                w.open_documents.push(buf);
                w.active_document_idx = Some(w.open_documents.len() - 1);
            });
        }
        Command::OpenFile => {
            crate::views::menubar::trigger_open_file(state);
        }
        Command::OpenFolder => {
            crate::views::sidebar::trigger_open_folder(state);
        }
        Command::CloseActiveEditor => {
            ws.update(|w| {
                if let Some(i) = w.active_document_idx {
                    w.close_tab(i);
                }
            });
        }
        Command::CloseAllEditors => {
            ws.update(|w| {
                w.open_documents.clear();
                w.active_document_idx = None;
            });
        }
        Command::ToggleSidebar => {
            state.show_sidebar.update(|s| *s = !*s);
        }
        Command::ToggleTerminal => {
            state.show_bottom_panel.update(|b| *b = !*b);
        }
        Command::ToggleAuxiliaryBar => {
            state.show_right_panel.update(|r| *r = !*r);
        }
        Command::FindInFiles => {
            state.show_sidebar.set(true);
            state.active_sidebar_tab.set(SidebarTab::Search);
        }
        Command::RunProject => {
            state.show_bottom_panel.set(true);
            state.active_bottom_tab.set(BottomTab::Terminal);
            state.terminal_logs.update(|l| l.push("▶ Project run started...".to_string()));
        }
        Command::SearchEverywhere => {
            state.show_sidebar.set(true);
            state.active_sidebar_tab.set(SidebarTab::Search);
        }
        Command::OpenDocumentation => {
            state.show_right_panel.set(true);
        }
        Command::Quit => {
            std::process::exit(0);
        }
    }
}
