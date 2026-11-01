//! # Keymap & Command Engine (VSCodium Compatible)
//!
//! Provides a structured, configurable command and keybinding system.
//! Supports standard VS Code / VSCodium keybinding definitions, chord parsing,
//! and serialization to/from `keybindings.json`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Standard IDE commands mappable to keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    #[serde(rename = "workbench.action.toggleWordWrap")]
    ToggleWordWrap,
    #[serde(rename = "workbench.action.files.save")]
    SaveFile,
    #[serde(rename = "workbench.action.files.newUntitledFile")]
    NewFile,
    #[serde(rename = "workbench.action.files.openFile")]
    OpenFile,
    #[serde(rename = "workbench.action.files.openFolder")]
    OpenFolder,
    #[serde(rename = "workbench.action.closeActiveEditor")]
    CloseActiveEditor,
    #[serde(rename = "workbench.action.closeAllEditors")]
    CloseAllEditors,
    #[serde(rename = "workbench.action.toggleSidebarVisibility")]
    ToggleSidebar,
    #[serde(rename = "workbench.action.terminal.toggleTerminal")]
    ToggleTerminal,
    #[serde(rename = "workbench.action.toggleAuxiliaryBar")]
    ToggleAuxiliaryBar,
    #[serde(rename = "workbench.action.findInFiles")]
    FindInFiles,
    #[serde(rename = "workbench.action.debug.run")]
    RunProject,
    #[serde(rename = "workbench.action.quickOpen")]
    SearchEverywhere,
    #[serde(rename = "workbench.action.openDocumentation")]
    OpenDocumentation,
    #[serde(rename = "workbench.action.quit")]
    Quit,
}

impl Command {
    /// Canonical command ID (VSCodium action identifier)
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::ToggleWordWrap => "workbench.action.toggleWordWrap",
            Command::SaveFile => "workbench.action.files.save",
            Command::NewFile => "workbench.action.files.newUntitledFile",
            Command::OpenFile => "workbench.action.files.openFile",
            Command::OpenFolder => "workbench.action.files.openFolder",
            Command::CloseActiveEditor => "workbench.action.closeActiveEditor",
            Command::CloseAllEditors => "workbench.action.closeAllEditors",
            Command::ToggleSidebar => "workbench.action.toggleSidebarVisibility",
            Command::ToggleTerminal => "workbench.action.terminal.toggleTerminal",
            Command::ToggleAuxiliaryBar => "workbench.action.toggleAuxiliaryBar",
            Command::FindInFiles => "workbench.action.findInFiles",
            Command::RunProject => "workbench.action.debug.run",
            Command::SearchEverywhere => "workbench.action.quickOpen",
            Command::OpenDocumentation => "workbench.action.openDocumentation",
            Command::Quit => "workbench.action.quit",
        }
    }
}

/// Normalized single key combination (modifiers + key)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: String,
}

impl KeyChord {
    /// Parse shortcut strings like `"Ctrl+Shift+F"`, `"alt+z"`, `"Ctrl+S"`
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        let mut key = String::new();

        for part in parts {
            let lower = part.to_lowercase();
            match lower.as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" | "altgr" => alt = true,
                "shift" => shift = true,
                "meta" | "cmd" | "win" | "super" => meta = true,
                other => {
                    key = other.to_string();
                }
            }
        }

        if key.is_empty() {
            return None;
        }

        Some(KeyChord {
            ctrl,
            alt,
            shift,
            meta,
            key,
        })
    }

    /// Formats as standard human-readable display string
    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Meta");
        }
        let k = match self.key.as_str() {
            "f1" => "F1".to_string(),
            "f4" => "F4".to_string(),
            "f5" => "F5".to_string(),
            "f12" => "F12".to_string(),
            other => other.to_uppercase(),
        };
        parts.push(&k);
        parts.join("+")
    }
}

/// Single keybinding mapping a shortcut string to a Command with optional condition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub command: Command,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

/// Collection of keybindings loaded from defaults or `keybindings.json`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keymap {
    pub bindings: Vec<KeyBinding>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::vscodium_default()
    }
}

impl Keymap {
    /// Built-in VSCodium / VS Code standard keybindings
    pub fn vscodium_default() -> Self {
        Self {
            bindings: vec![
                KeyBinding {
                    key: "Alt+Z".to_string(),
                    command: Command::ToggleWordWrap,
                    when: Some("editorTextFocus".to_string()),
                },
                KeyBinding {
                    key: "Ctrl+S".to_string(),
                    command: Command::SaveFile,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+N".to_string(),
                    command: Command::NewFile,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+O".to_string(),
                    command: Command::OpenFile,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+Shift+O".to_string(),
                    command: Command::OpenFolder,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+W".to_string(),
                    command: Command::CloseActiveEditor,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+B".to_string(),
                    command: Command::ToggleSidebar,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+J".to_string(),
                    command: Command::ToggleTerminal,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+Shift+B".to_string(),
                    command: Command::ToggleAuxiliaryBar,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+Shift+F".to_string(),
                    command: Command::FindInFiles,
                    when: None,
                },
                KeyBinding {
                    key: "Ctrl+F5".to_string(),
                    command: Command::RunProject,
                    when: None,
                },
                KeyBinding {
                    key: "F1".to_string(),
                    command: Command::OpenDocumentation,
                    when: None,
                },
                KeyBinding {
                    key: "Alt+F4".to_string(),
                    command: Command::Quit,
                    when: None,
                },
            ],
        }
    }

    /// Retrieve the shortcut display string for a given command
    pub fn shortcut_for(&self, cmd: Command) -> Option<String> {
        self.bindings
            .iter()
            .find(|b| b.command == cmd)
            .map(|b| b.key.clone())
    }

    /// Discovers and loads keybindings from `settings/keybindings.json` or fallback paths,
    /// creating a default file if missing.
    pub fn load() -> Self {
        let candidates = [
            PathBuf::from("settings/keybindings.json"),
            PathBuf::from("keybindings.json"),
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    p.parent()
                        .map(|d| d.join("settings").join("keybindings.json"))
                })
                .unwrap_or_else(|| PathBuf::from("settings/keybindings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("keybindings.json")))
                .unwrap_or_else(|| PathBuf::from("keybindings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    p.parent().and_then(|d| {
                        d.parent()
                            .map(|g| g.join("settings").join("keybindings.json"))
                    })
                })
                .unwrap_or_else(|| PathBuf::from("settings/keybindings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    p.parent()
                        .and_then(|d| d.parent().map(|g| g.join("keybindings.json")))
                })
                .unwrap_or_else(|| PathBuf::from("keybindings.json")),
        ];

        for path in &candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(bindings) = serde_json::from_str::<Vec<KeyBinding>>(&content) {
                        return Self { bindings };
                    }
                }
            }
        }

        let default_km = Self::vscodium_default();
        let _ = default_km.save(Path::new("settings/keybindings.json"));
        default_km
    }

    /// Load keybindings from file, creating default file if missing
    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(bindings) = serde_json::from_str::<Vec<KeyBinding>>(&content) {
                    return Self { bindings };
                }
            }
        }
        let default_km = Self::vscodium_default();
        let _ = default_km.save(path);
        default_km
    }

    /// Save current keymap as JSON array to file, ensuring parent directory exists
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let json =
            serde_json::to_string_pretty(&self.bindings).unwrap_or_else(|_| "[]".to_string());
        std::fs::write(path, json)
    }
}
