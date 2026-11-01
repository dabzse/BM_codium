//! # Configuration & Settings Module
//!
//! Manages `settings.json` loading, path resolution (locales, assets, manuals, plugins,
//! autocomplete assistants), and user preferences without requiring recompilation.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodiumSettings {
    /// Active interface language code ("en", "hu", etc.)
    #[serde(default = "default_locale")]
    pub locale: String,

    /// Relative or absolute path to the locales folder containing *.po files
    #[serde(default = "default_locales_dir")]
    pub locales_dir: String,

    /// Relative or absolute path to the resources folder (tutor cards, icons, desktop files, etc.)
    #[serde(default = "default_resources_dir", alias = "assets_dir")]
    pub resources_dir: String,

    /// Relative or absolute path to the manuals / documentation directory
    #[serde(default = "default_manuals_dir")]
    pub manuals_dir: String,

    /// Allow access to manuals and documentation in educational mode
    #[serde(default = "default_true")]
    pub allow_manuals_access: bool,

    /// Relative or absolute path to the plugins directory
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,

    /// Relative or absolute path to the autocomplete assistant (Kite or custom daemon)
    #[serde(default = "default_autocomplete_path", alias = "kite_path")]
    pub autocomplete_assistant_path: String,

    /// Educational institution name or course title
    #[serde(default = "default_institution")]
    pub institution_name: String,

    /// Offline exam restriction mode
    #[serde(default)]
    pub exam_mode: bool,

    /// Allow generative AI in editor
    #[serde(default)]
    pub allow_ai_generation: bool,

    /// Allow smart autocomplete assistant
    #[serde(default = "default_true", alias = "allow_kite_completion")]
    pub allow_autocomplete_assistant: bool,

    /// Enable pedagogic doc tutor panel
    #[serde(default = "default_true")]
    pub doc_tutor_enabled: bool,

    /// Show or hide JetBrains-style Search Everywhere pill in titlebar
    #[serde(default = "default_true")]
    pub show_titlebar_search: bool,

    /// Word wrap toggle for editor (Alt+Z)
    #[serde(default)]
    pub word_wrap: bool,

    /// Local LLM API endpoint for AI edition
    #[serde(default = "default_ai_endpoint")]
    pub ai_endpoint: String,

    /// Visual theme ("dark" or "light")
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Editor font size
    #[serde(default = "default_font_size")]
    pub font_size: f64,

    /// Left sidebar width in pixels
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f64,

    /// Bottom panel height in pixels
    #[serde(default = "default_bottom_panel_height")]
    pub bottom_panel_height: f64,

    /// Right secondary panel width in pixels
    #[serde(default = "default_right_panel_width")]
    pub right_panel_width: f64,

    /// UI font family
    #[serde(default = "default_ui_font")]
    pub ui_font_family: String,

    /// Editor font family
    #[serde(default = "default_editor_font")]
    pub editor_font_family: String,

    /// Terminal font family (monospace preferred)
    #[serde(default = "default_terminal_font")]
    pub terminal_font_family: String,

    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub window_width: f64,

    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub window_height: f64,

    /// Last opened workspace directory path
    #[serde(default)]
    pub last_workspace_folder: Option<String>,
}

fn default_locale() -> String { "hu".to_string() }
fn default_locales_dir() -> String { "locales".to_string() }
fn default_resources_dir() -> String { "resources".to_string() }
fn default_manuals_dir() -> String { "manuals".to_string() }
fn default_plugins_dir() -> String { "plugins".to_string() }
fn default_autocomplete_path() -> String { "plugins/autocomplete".to_string() }
fn default_institution() -> String { "BM::Codium University & AI Lab".to_string() }
fn default_ai_endpoint() -> String { "http://localhost:11434".to_string() }
fn default_theme() -> String { "dark".to_string() }
fn default_font_size() -> f64 { 13.0 }
fn default_sidebar_width() -> f64 { 260.0 }
fn default_bottom_panel_height() -> f64 { 180.0 }
fn default_right_panel_width() -> f64 { 280.0 }
fn default_ui_font() -> String { "sans-serif".to_string() }
fn default_editor_font() -> String { "monospace".to_string() }
fn default_terminal_font() -> String { "monospace".to_string() }
fn default_window_width() -> f64 { 1280.0 }
fn default_window_height() -> f64 { 800.0 }
fn default_true() -> bool { true }

impl Default for CodiumSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            locales_dir: default_locales_dir(),
            resources_dir: default_resources_dir(),
            manuals_dir: default_manuals_dir(),
            allow_manuals_access: true,
            plugins_dir: default_plugins_dir(),
            autocomplete_assistant_path: default_autocomplete_path(),
            institution_name: default_institution(),
            exam_mode: false,
            allow_ai_generation: false,
            allow_autocomplete_assistant: true,
            doc_tutor_enabled: true,
            show_titlebar_search: true,
            ai_endpoint: default_ai_endpoint(),
            theme: default_theme(),
            font_size: default_font_size(),
            sidebar_width: default_sidebar_width(),
            bottom_panel_height: default_bottom_panel_height(),
            right_panel_width: default_right_panel_width(),
            word_wrap: false,
            ui_font_family: default_ui_font(),
            editor_font_family: default_editor_font(),
            terminal_font_family: default_terminal_font(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            last_workspace_folder: None,
        }
    }
}

impl CodiumSettings {
    /// Discovers and loads `settings.json` from `settings/` directory, current directory or application root.
    /// If no file is found, creates a default `settings/settings.json`.
    pub fn load() -> Self {
        let candidates = [
            PathBuf::from("settings/settings.json"),
            PathBuf::from("settings.json"),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("settings").join("settings.json")))
                .unwrap_or_else(|| PathBuf::from("settings/settings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("settings.json")))
                .unwrap_or_else(|| PathBuf::from("settings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().and_then(|d| d.parent().map(|g| g.join("settings").join("settings.json"))))
                .unwrap_or_else(|| PathBuf::from("settings/settings.json")),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().and_then(|d| d.parent().map(|g| g.join("settings.json"))))
                .unwrap_or_else(|| PathBuf::from("settings.json")),
        ];

        for path in &candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(settings) = serde_json::from_str::<Self>(&content) {
                        return settings;
                    }
                }
            }
        }

        // Save default settings/settings.json if none exists
        let default_settings = Self::default();
        let _ = default_settings.save();
        default_settings
    }

    /// Saves the current configuration to `settings/settings.json` (or `settings.json` if already present in root)
    pub fn save(&self) -> Result<(), std::io::Error> {
        let target_path = if Path::new("settings.json").exists() && !Path::new("settings/settings.json").exists() {
            PathBuf::from("settings.json")
        } else {
            PathBuf::from("settings/settings.json")
        };
        self.save_to(&target_path)
    }

    /// Saves to a specific file path, automatically creating parent directories if needed
    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Resolves `locales_dir` against current working directory or binary location
    pub fn resolve_locales_dir(&self) -> PathBuf {
        Self::resolve_path(&self.locales_dir)
    }

    /// Resolves `resources_dir` against current working directory or binary location
    pub fn resolve_resources_dir(&self) -> PathBuf {
        Self::resolve_path(&self.resources_dir)
    }

    /// Backwards compatibility alias for `assets_dir`
    pub fn resolve_assets_dir(&self) -> PathBuf {
        self.resolve_resources_dir()
    }

    /// Resolves `manuals_dir` against current working directory or binary location
    pub fn resolve_manuals_dir(&self) -> PathBuf {
        Self::resolve_path(&self.manuals_dir)
    }

    /// Resolves `plugins_dir` against current working directory or binary location
    pub fn resolve_plugins_dir(&self) -> PathBuf {
        Self::resolve_path(&self.plugins_dir)
    }

    /// Resolves `autocomplete_assistant_path` against current working directory or binary location
    pub fn resolve_autocomplete_assistant_path(&self) -> PathBuf {
        Self::resolve_path(&self.autocomplete_assistant_path)
    }

    /// Backwards compatibility alias for kite_path
    pub fn kite_path(&self) -> &str {
        &self.autocomplete_assistant_path
    }

    fn resolve_path(relative_or_absolute: &str) -> PathBuf {
        let p = Path::new(relative_or_absolute);
        if p.is_absolute() {
            return p.to_path_buf();
        }

        // Check CWD
        let cwd_candidate = PathBuf::from(relative_or_absolute);
        if cwd_candidate.exists() {
            return cwd_candidate;
        }

        // Check relative to executable parent
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let candidate = parent.join(relative_or_absolute);
                if candidate.exists() {
                    return candidate;
                }
                if let Some(grandparent) = parent.parent() {
                    let candidate2 = grandparent.join(relative_or_absolute);
                    if candidate2.exists() {
                        return candidate2;
                    }
                }
            }
        }

        cwd_candidate
    }
}
