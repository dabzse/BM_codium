//! # Icon Theme System (VS Code Compatible)
//!
//! Provides vector SVG file and folder icon resolution based on file extensions,
//! exact file names, and folder expansion state. Implements the standard VS Code
//! icon theme format (`iconDefinitions`, `fileExtensions`, `fileNames`, etc.)
//! with `vscode-great-icons` as the default theme.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Definition of a single icon in the theme
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconDefinition {
    #[serde(rename = "iconPath", default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub svg: Option<String>,
}

impl IconDefinition {
    pub fn from_svg(svg: impl Into<String>) -> Self {
        Self {
            icon_path: None,
            svg: Some(svg.into()),
        }
    }
}

/// Complete VS Code-compatible icon theme definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconTheme {
    pub name: String,
    pub file: String,
    pub folder: String,
    #[serde(rename = "folderExpanded", default)]
    pub folder_expanded: String,
    #[serde(rename = "fileExtensions", default)]
    pub file_extensions: HashMap<String, String>,
    #[serde(rename = "fileNames", default)]
    pub file_names: HashMap<String, String>,
    #[serde(rename = "iconDefinitions", default)]
    pub icon_definitions: HashMap<String, IconDefinition>,
}

impl Default for IconTheme {
    fn default() -> Self {
        Self::vscode_great_icons_default()
    }
}

impl IconTheme {
    /// Retrieve SVG markup for a given file name
    pub fn get_file_icon(&self, filename: &str) -> &str {
        let clean = filename.trim();

        // 1. Check exact file name
        if let Some(icon_id) = self.file_names.get(clean) {
            if let Some(svg) = self.lookup_svg(icon_id) {
                return svg;
            }
        }

        // 2. Check lowercased exact file name
        let lower = clean.to_lowercase();
        if let Some(icon_id) = self.file_names.get(&lower) {
            if let Some(svg) = self.lookup_svg(icon_id) {
                return svg;
            }
        }

        // 3. Check multi-part extensions (e.g. .stories.tsx, .d.ts, .test.js)
        if let Some(dot_idx) = clean.find('.') {
            let full_ext = &clean[dot_idx + 1..].to_lowercase();
            if let Some(icon_id) = self.file_extensions.get(full_ext) {
                if let Some(svg) = self.lookup_svg(icon_id) {
                    return svg;
                }
            }
        }

        // 4. Check standard single extension
        if let Some(dot_idx) = clean.rfind('.') {
            let ext = &clean[dot_idx + 1..].to_lowercase();
            if let Some(icon_id) = self.file_extensions.get(ext) {
                if let Some(svg) = self.lookup_svg(icon_id) {
                    return svg;
                }
            }
        }

        // 5. Fallback to default file icon
        self.lookup_svg(&self.file).unwrap_or(DEFAULT_FILE_SVG)
    }

    /// Retrieve SVG markup for a folder
    pub fn get_folder_icon(&self, is_expanded: bool) -> &str {
        let key = if is_expanded {
            &self.folder_expanded
        } else {
            &self.folder
        };
        self.lookup_svg(key).unwrap_or(if is_expanded {
            DEFAULT_FOLDER_OPEN_SVG
        } else {
            DEFAULT_FOLDER_SVG
        })
    }

    fn lookup_svg(&self, icon_id: &str) -> Option<&str> {
        self.icon_definitions.get(icon_id).and_then(|def| def.svg.as_deref())
    }

    /// Available built-in icon themes
    pub fn available_themes() -> &'static [&'static str] {
        &["vscode-great-icons", "minimal"]
    }

    /// Retrieve a built-in icon theme by name
    pub fn built_in_theme(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "vscode-great-icons" | "great-icons" | "great" => Some(Self::vscode_great_icons_default()),
            "minimal" | "monochrome" => Some(Self::minimal_default()),
            _ => None,
        }
    }

    /// Parse icon theme from JSON string (supports VS Code icon theme schema)
    pub fn from_json(json_str: &str) -> Result<Self, anyhow::Error> {
        let mut theme: Self = serde_json::from_str(json_str)?;

        // Ensure missing essential icon definitions have default SVG fallbacks
        let great = Self::vscode_great_icons_default();
        for (k, v) in great.icon_definitions {
            theme.icon_definitions.entry(k).or_insert(v);
        }

        Ok(theme)
    }

    /// Save icon theme as readable JSON
    pub fn save_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Load icon theme profile by name, checking settings/icons/<name>.json,
    /// creating fallback if missing, and falling back to built-in.
    pub fn load_profile(theme_name: &str) -> Self {
        let clean = theme_name.trim();

        let candidate_paths = [
            PathBuf::from(format!("settings/icons/{clean}.json")),
            PathBuf::from(format!("icons/{clean}.json")),
            PathBuf::from(format!("resources/icons/{clean}.json")),
        ];

        for path in &candidate_paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(theme) = Self::from_json(&content) {
                        return theme;
                    }
                }
            }
        }

        // Check if matching a built-in theme
        if let Some(built_in) = Self::built_in_theme(clean) {
            let target_path = PathBuf::from(format!("settings/icons/{clean}.json"));
            let _ = built_in.save_to_file(&target_path);
            return built_in;
        }

        // Default fallback: vscode-great-icons
        let default_theme = Self::vscode_great_icons_default();
        let target_path = PathBuf::from("settings/icons/vscode-great-icons.json");
        let _ = default_theme.save_to_file(&target_path);
        default_theme
    }

    /// Default vscode-great-icons theme with rich, vibrant vector SVG icons
    pub fn vscode_great_icons_default() -> Self {
        let mut defs = HashMap::new();

        defs.insert("_file".to_string(), IconDefinition::from_svg(DEFAULT_FILE_SVG));
        defs.insert("_folder".to_string(), IconDefinition::from_svg(DEFAULT_FOLDER_SVG));
        defs.insert("_folder_open".to_string(), IconDefinition::from_svg(DEFAULT_FOLDER_OPEN_SVG));

        defs.insert("_f_rust".to_string(), IconDefinition::from_svg(RUST_SVG));
        defs.insert("_f_python".to_string(), IconDefinition::from_svg(PYTHON_SVG));
        defs.insert("_f_js".to_string(), IconDefinition::from_svg(JAVASCRIPT_SVG));
        defs.insert("_f_ts".to_string(), IconDefinition::from_svg(TYPESCRIPT_SVG));
        defs.insert("_f_react".to_string(), IconDefinition::from_svg(REACT_SVG));
        defs.insert("_f_html".to_string(), IconDefinition::from_svg(HTML_SVG));
        defs.insert("_f_css".to_string(), IconDefinition::from_svg(CSS_SVG));
        defs.insert("_f_sass".to_string(), IconDefinition::from_svg(SASS_SVG));
        defs.insert("_f_json".to_string(), IconDefinition::from_svg(JSON_SVG));
        defs.insert("_f_toml".to_string(), IconDefinition::from_svg(TOML_SVG));
        defs.insert("_f_yaml".to_string(), IconDefinition::from_svg(YAML_SVG));
        defs.insert("_f_markdown".to_string(), IconDefinition::from_svg(MARKDOWN_SVG));
        defs.insert("_f_git".to_string(), IconDefinition::from_svg(GIT_SVG));
        defs.insert("_f_docker".to_string(), IconDefinition::from_svg(DOCKER_SVG));
        defs.insert("_f_c".to_string(), IconDefinition::from_svg(C_SVG));
        defs.insert("_f_cpp".to_string(), IconDefinition::from_svg(CPP_SVG));
        defs.insert("_f_csharp".to_string(), IconDefinition::from_svg(CSHARP_SVG));
        defs.insert("_f_go".to_string(), IconDefinition::from_svg(GO_SVG));
        defs.insert("_f_java".to_string(), IconDefinition::from_svg(JAVA_SVG));
        defs.insert("_f_kotlin".to_string(), IconDefinition::from_svg(KOTLIN_SVG));
        defs.insert("_f_php".to_string(), IconDefinition::from_svg(PHP_SVG));
        defs.insert("_f_ruby".to_string(), IconDefinition::from_svg(RUBY_SVG));
        defs.insert("_f_swift".to_string(), IconDefinition::from_svg(SWIFT_SVG));
        defs.insert("_f_shell".to_string(), IconDefinition::from_svg(SHELL_SVG));
        defs.insert("_f_database".to_string(), IconDefinition::from_svg(DATABASE_SVG));
        defs.insert("_f_image".to_string(), IconDefinition::from_svg(IMAGE_SVG));
        defs.insert("_f_archive".to_string(), IconDefinition::from_svg(ARCHIVE_SVG));
        defs.insert("_f_settings".to_string(), IconDefinition::from_svg(SETTINGS_SVG));
        defs.insert("_f_lock".to_string(), IconDefinition::from_svg(LOCK_SVG));

        let mut exts = HashMap::new();
        exts.insert("rs".to_string(), "_f_rust".to_string());
        exts.insert("py".to_string(), "_f_python".to_string());
        exts.insert("pyw".to_string(), "_f_python".to_string());
        exts.insert("js".to_string(), "_f_js".to_string());
        exts.insert("mjs".to_string(), "_f_js".to_string());
        exts.insert("cjs".to_string(), "_f_js".to_string());
        exts.insert("ts".to_string(), "_f_ts".to_string());
        exts.insert("mts".to_string(), "_f_ts".to_string());
        exts.insert("cts".to_string(), "_f_ts".to_string());
        exts.insert("jsx".to_string(), "_f_react".to_string());
        exts.insert("tsx".to_string(), "_f_react".to_string());
        exts.insert("html".to_string(), "_f_html".to_string());
        exts.insert("htm".to_string(), "_f_html".to_string());
        exts.insert("css".to_string(), "_f_css".to_string());
        exts.insert("scss".to_string(), "_f_sass".to_string());
        exts.insert("sass".to_string(), "_f_sass".to_string());
        exts.insert("less".to_string(), "_f_sass".to_string());
        exts.insert("json".to_string(), "_f_json".to_string());
        exts.insert("jsonc".to_string(), "_f_json".to_string());
        exts.insert("toml".to_string(), "_f_toml".to_string());
        exts.insert("yaml".to_string(), "_f_yaml".to_string());
        exts.insert("yml".to_string(), "_f_yaml".to_string());
        exts.insert("md".to_string(), "_f_markdown".to_string());
        exts.insert("markdown".to_string(), "_f_markdown".to_string());
        exts.insert("c".to_string(), "_f_c".to_string());
        exts.insert("h".to_string(), "_f_c".to_string());
        exts.insert("cpp".to_string(), "_f_cpp".to_string());
        exts.insert("hpp".to_string(), "_f_cpp".to_string());
        exts.insert("cc".to_string(), "_f_cpp".to_string());
        exts.insert("cxx".to_string(), "_f_cpp".to_string());
        exts.insert("cs".to_string(), "_f_csharp".to_string());
        exts.insert("go".to_string(), "_f_go".to_string());
        exts.insert("java".to_string(), "_f_java".to_string());
        exts.insert("jar".to_string(), "_f_java".to_string());
        exts.insert("kt".to_string(), "_f_kotlin".to_string());
        exts.insert("kts".to_string(), "_f_kotlin".to_string());
        exts.insert("php".to_string(), "_f_php".to_string());
        exts.insert("rb".to_string(), "_f_ruby".to_string());
        exts.insert("rake".to_string(), "_f_ruby".to_string());
        exts.insert("swift".to_string(), "_f_swift".to_string());
        exts.insert("sh".to_string(), "_f_shell".to_string());
        exts.insert("bash".to_string(), "_f_shell".to_string());
        exts.insert("zsh".to_string(), "_f_shell".to_string());
        exts.insert("ps1".to_string(), "_f_shell".to_string());
        exts.insert("sql".to_string(), "_f_database".to_string());
        exts.insert("png".to_string(), "_f_image".to_string());
        exts.insert("jpg".to_string(), "_f_image".to_string());
        exts.insert("jpeg".to_string(), "_f_image".to_string());
        exts.insert("gif".to_string(), "_f_image".to_string());
        exts.insert("svg".to_string(), "_f_image".to_string());
        exts.insert("webp".to_string(), "_f_image".to_string());
        exts.insert("ico".to_string(), "_f_image".to_string());
        exts.insert("zip".to_string(), "_f_archive".to_string());
        exts.insert("tar".to_string(), "_f_archive".to_string());
        exts.insert("gz".to_string(), "_f_archive".to_string());
        exts.insert("7z".to_string(), "_f_archive".to_string());
        exts.insert("rar".to_string(), "_f_archive".to_string());
        exts.insert("lock".to_string(), "_f_lock".to_string());

        let mut names = HashMap::new();
        names.insert("Cargo.toml".to_string(), "_f_rust".to_string());
        names.insert("Cargo.lock".to_string(), "_f_lock".to_string());
        names.insert("package.json".to_string(), "_f_json".to_string());
        names.insert("package-lock.json".to_string(), "_f_lock".to_string());
        names.insert("tsconfig.json".to_string(), "_f_ts".to_string());
        names.insert("Dockerfile".to_string(), "_f_docker".to_string());
        names.insert(".gitignore".to_string(), "_f_git".to_string());
        names.insert(".gitattributes".to_string(), "_f_git".to_string());
        names.insert(".gitmodules".to_string(), "_f_git".to_string());
        names.insert("README.md".to_string(), "_f_markdown".to_string());
        names.insert("settings.json".to_string(), "_f_settings".to_string());
        names.insert(".env".to_string(), "_f_settings".to_string());

        Self {
            name: "vscode-great-icons".to_string(),
            file: "_file".to_string(),
            folder: "_folder".to_string(),
            folder_expanded: "_folder_open".to_string(),
            file_extensions: exts,
            file_names: names,
            icon_definitions: defs,
        }
    }

    /// Minimal monochrome icon theme
    pub fn minimal_default() -> Self {
        let mut defs = HashMap::new();
        defs.insert("_file".to_string(), IconDefinition::from_svg(DEFAULT_FILE_SVG));
        defs.insert("_folder".to_string(), IconDefinition::from_svg(DEFAULT_FOLDER_SVG));
        defs.insert("_folder_open".to_string(), IconDefinition::from_svg(DEFAULT_FOLDER_OPEN_SVG));

        Self {
            name: "minimal".to_string(),
            file: "_file".to_string(),
            folder: "_folder".to_string(),
            folder_expanded: "_folder_open".to_string(),
            file_extensions: HashMap::new(),
            file_names: HashMap::new(),
            icon_definitions: defs,
        }
    }
}

// Crisp Vector SVG Icon Markup

pub const DEFAULT_FILE_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M3 1.5h6.5L13 5v9.5H3V1.5z" fill="#90A4AE"/><path d="M9.5 1.5V5H13" fill="#B0BEC5"/></svg>"##;
pub const DEFAULT_FOLDER_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M1.5 2.5l1.5-1h3.5l1.5 1h6.5l.5.5v9.5l-.5.5h-13l-.5-.5v-10z" fill="#E5A93C"/></svg>"##;
pub const DEFAULT_FOLDER_OPEN_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M1.5 2.5l1.5-1h3.5l1.5 1h6.5l.5.5v2.5h-11.5l-1.5 7v-10zm1.2 4h12.5l-1.8 6.5h-11.7l1-6.5z" fill="#F5BA42"/></svg>"##;

pub const RUST_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><circle cx="8" cy="8" r="7" fill="#DEA584"/><circle cx="8" cy="8" r="4.5" fill="#202020"/><text x="5" y="11" font-size="8.5" font-weight="bold" fill="#DEA584">R</text></svg>"##;
pub const PYTHON_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M7.9 1C5.2 1 5.4 2.2 5.4 2.2v1.3h2.6v.4H4.2S2 3.6 2 6.4s1.9 2.6 1.9 2.6h1.1V7.7s-.1-1.3 1.3-1.3h2.6s1.2 0 1.2-1.2V2.3S10.3 1 7.9 1zm-1.4.9a.5.5 0 1 1 0 1 .5.5 0 0 1 0-1z" fill="#3572A5"/><path d="M8.1 15c2.7 0 2.5-1.2 2.5-1.2v-1.3H8v-.4h3.8s2.2.3 2.2-2.5-1.9-2.6-1.9-2.6H11v1.3s.1 1.3-1.3 1.3H7.1s-1.2 0-1.2 1.2v2.9s-.2 1.3 2.2 1.3zm1.4-.9a.5.5 0 1 1 0-1 .5.5 0 0 1 0 1z" fill="#FFD43B"/></svg>"##;
pub const JAVASCRIPT_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#F7DF1E"/><text x="4" y="11" font-size="8" font-weight="bold" fill="#000">JS</text></svg>"##;
pub const TYPESCRIPT_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#3178C6"/><text x="4" y="11" font-size="8" font-weight="bold" fill="#FFF">TS</text></svg>"##;
pub const REACT_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><ellipse cx="8" cy="8" rx="6.5" ry="2.5" fill="none" stroke="#61DAFB" stroke-width="1.1" transform="rotate(30 8 8)"/><ellipse cx="8" cy="8" rx="6.5" ry="2.5" fill="none" stroke="#61DAFB" stroke-width="1.1" transform="rotate(-30 8 8)"/><ellipse cx="8" cy="8" rx="6.5" ry="2.5" fill="none" stroke="#61DAFB" stroke-width="1.1" transform="rotate(90 8 8)"/><circle cx="8" cy="8" r="1.3" fill="#61DAFB"/></svg>"##;
pub const HTML_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M2.5 1.5l1 12 4.5 1.5 4.5-1.5 1-12h-11z" fill="#E34F26"/><path d="M8 2.5v10.8l3.6-1.2.8-9.6H8z" fill="#EF652A"/><text x="3.5" y="9.5" font-size="5" font-weight="bold" fill="#FFF">&lt;&gt;</text></svg>"##;
pub const CSS_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M2.5 1.5l1 12 4.5 1.5 4.5-1.5 1-12h-11z" fill="#1572B6"/><path d="M8 2.5v10.8l3.6-1.2.8-9.6H8z" fill="#33A9DC"/><text x="4" y="9.5" font-size="5" font-weight="bold" fill="#FFF">#</text></svg>"##;
pub const SASS_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="3" x="1" y="1" fill="#CF649A"/><text x="2.5" y="10.5" font-size="6" font-weight="bold" fill="#FFF">sass</text></svg>"##;
pub const JSON_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#FBC02D"/><text x="3.5" y="10.5" font-size="7" font-weight="bold" fill="#212121">{}</text></svg>"##;
pub const TOML_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#9C4121"/><text x="2" y="10.5" font-size="5.5" font-weight="bold" fill="#FFF">toml</text></svg>"##;
pub const YAML_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#CB171E"/><text x="2" y="10.5" font-size="5.5" font-weight="bold" fill="#FFF">yml</text></svg>"##;
pub const MARKDOWN_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="15" height="10" rx="1.5" x="0.5" y="3" fill="#083FA1"/><path d="M2.5 5.5v5h1.5l1.5-2 1.5 2h1.5v-5h-1.5v2.8l-1.5-2-1.5 2V5.5H2.5zm8 0v3h-1.2l1.7 2 1.7-2h-1.2v-3h-1z" fill="#FFF"/></svg>"##;
pub const GIT_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M15.2 7.2L8.8.8c-.4-.4-1-.4-1.4 0L6 2.2l1.8 1.8c.4-.1.8 0 1.1.3.4.4.4 1 0 1.4-.4.4-1 .4-1.4 0-.3-.3-.4-.7-.3-1.1L5.4 2.8 1 7.2c-.4.4-.4 1 0 1.4l6.4 6.4c.4.4 1 .4 1.4 0l6.4-6.4c.4-.4.4-1 0-1.4z" fill="#F05032"/></svg>"##;
pub const DOCKER_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M15.5 6.5c-.3-.2-.8-.2-1.1-.1-.3-.6-.8-1.1-1.5-1.4-.2 0-.3.1-.3.2 0 .5.2 1.1.4 1.5-.4.2-1 .3-1.8.3H1.2C.5 7 0 7.6 0 8.3c0 2.2 1.8 4.2 4.2 4.2 3.1 0 5.7-1.7 6.8-4.2 1.2 0 2.3-.5 3.1-1.3.5-.2 1.1-.3 1.4-.5zm-8.8-1h1.2v1.2H6.7V5.5zm1.5 0h1.2v1.2H8.2V5.5zm-3 0h1.2v1.2H5.2V5.5zm4.5 0h1.2v1.2H9.7V5.5z" fill="#2496ED"/></svg>"##;
pub const C_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#659AD2"/><text x="4.5" y="11" font-size="9" font-weight="bold" fill="#FFF">C</text></svg>"##;
pub const CPP_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#00599C"/><text x="2" y="10.5" font-size="7" font-weight="bold" fill="#FFF">C++</text></svg>"##;
pub const CSHARP_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#68217A"/><text x="3" y="10.5" font-size="7.5" font-weight="bold" fill="#FFF">C#</text></svg>"##;
pub const GO_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#00ADD8"/><text x="2.5" y="10.5" font-size="7" font-weight="bold" fill="#FFF">GO</text></svg>"##;
pub const JAVA_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M4 11s.5 1 4 1 4-1 4-1H4zm6.5-6.5c-1 0-1.8.8-1.8 1.8h1.2c0-.3.3-.6.6-.6s.6.3.6.6c0 .8-1.5 1-1.5 2.2h1.2c0-.7 1.5-.9 1.5-2.2 0-1-.8-1.8-1.8-1.8z" fill="#EA2D2E"/><path d="M3 13h10v1H3z" fill="#5382A1"/></svg>"##;
pub const KOTLIN_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="2" x="1" y="1" fill="#7F52FF"/><text x="3.5" y="11" font-size="8.5" font-weight="bold" fill="#FFF">K</text></svg>"##;
pub const PHP_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="10" rx="3" x="1" y="3" fill="#777BB4"/><text x="2.5" y="10" font-size="5.5" font-weight="bold" fill="#FFF">php</text></svg>"##;
pub const RUBY_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path d="M3.5 3.5l-2 3.5 6.5 7 6.5-7-2-3.5h-9z" fill="#CC342D"/></svg>"##;
pub const SWIFT_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="14" rx="3" x="1" y="1" fill="#FA7343"/><circle cx="8" cy="8" r="4.5" fill="#FFF"/></svg>"##;
pub const SHELL_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="14" height="12" rx="2" x="1" y="2" fill="#2E3440"/><text x="2.5" y="10.5" font-size="7" font-weight="bold" fill="#89E051">&gt;_</text></svg>"##;
pub const DATABASE_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><ellipse cx="8" cy="3.5" rx="6" ry="2" fill="#336791"/><path d="M2 3.5v4c0 1.1 2.7 2 6 2s6-.9 6-2v-4" fill="none" stroke="#336791" stroke-width="1.2"/><path d="M2 7.5v4c0 1.1 2.7 2 6 2s6-.9 6-2v-4" fill="none" stroke="#336791" stroke-width="1.2"/></svg>"##;
pub const IMAGE_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="13" height="11" rx="1.5" x="1.5" y="2.5" fill="#A074C4"/><circle cx="5" cy="5.5" r="1.2" fill="#FFF"/><path d="M3 11.5l3.5-4 3 3.5 2-2 2.5 2.5H3z" fill="#FFF"/></svg>"##;
pub const ARCHIVE_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="12" height="13" rx="1.5" x="2" y="1.5" fill="#8D6E63"/><rect width="3" height="13" x="6.5" y="1.5" fill="#5D4037"/><rect width="2" height="3" x="7" y="6" fill="#FFD54F"/></svg>"##;
pub const SETTINGS_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><circle cx="8" cy="8" r="6" fill="#546E7A"/><circle cx="8" cy="8" r="2.5" fill="#202020"/></svg>"##;
pub const LOCK_SVG: &str = r##"<svg width="15" height="15" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><rect width="10" height="8" rx="1.5" x="3" y="6.5" fill="#FBC02D"/><path d="M5.5 6.5V4.5a2.5 2.5 0 015 0v2" fill="none" stroke="#F57F17" stroke-width="1.3"/><circle cx="8" cy="10" r="1" fill="#795548"/></svg>"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vscode_great_icons_default() {
        let theme = IconTheme::vscode_great_icons_default();
        assert_eq!(theme.name, "vscode-great-icons");
        assert_eq!(theme.get_file_icon("main.rs"), RUST_SVG);
        assert_eq!(theme.get_file_icon("Cargo.toml"), RUST_SVG);
        assert_eq!(theme.get_file_icon("Cargo.lock"), LOCK_SVG);
        assert_eq!(theme.get_file_icon("script.py"), PYTHON_SVG);
        assert_eq!(theme.get_file_icon("app.tsx"), REACT_SVG);
        assert_eq!(theme.get_file_icon("index.html"), HTML_SVG);
        assert_eq!(theme.get_file_icon("style.css"), CSS_SVG);
        assert_eq!(theme.get_file_icon("data.json"), JSON_SVG);
        assert_eq!(theme.get_file_icon("README.md"), MARKDOWN_SVG);
        assert_eq!(theme.get_file_icon(".gitignore"), GIT_SVG);
        assert_eq!(theme.get_file_icon("Dockerfile"), DOCKER_SVG);
        assert_eq!(theme.get_file_icon("random.unknown"), DEFAULT_FILE_SVG);
    }

    #[test]
    fn test_folder_icons() {
        let theme = IconTheme::vscode_great_icons_default();
        assert_eq!(theme.get_folder_icon(false), DEFAULT_FOLDER_SVG);
        assert_eq!(theme.get_folder_icon(true), DEFAULT_FOLDER_OPEN_SVG);
    }

    #[test]
    fn test_load_profile_and_fallback() {
        let theme = IconTheme::load_profile("vscode-great-icons");
        assert_eq!(theme.name, "vscode-great-icons");

        let min = IconTheme::load_profile("minimal");
        assert_eq!(min.name, "minimal");
        let _ = fs::remove_dir_all("settings");
    }
}
