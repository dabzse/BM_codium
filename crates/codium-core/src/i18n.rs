//! # GNU gettext `.po` Internationalization (i18n) Engine
//!
//! Provides dynamic runtime loading of `.po` catalogs from `locales/`,
//! with embedded fallbacks. Allows translators and educators to edit translations
//! or add new languages (e.g. `de.po`, `fr.po`) without recompiling the application.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// Embedded fallback catalogs guarantee that standalone binaries never fail or show empty strings
static FALLBACK_EN: &str = include_str!("../../../locales/en.po");
static FALLBACK_HU: &str = include_str!("../../../locales/hu.po");

/// A single parsed GNU gettext PO catalog
#[derive(Clone, Debug, Default)]
pub struct PoCatalog {
    pub language: String,
    pub messages: HashMap<String, String>,
}

impl PoCatalog {
    /// Parses GNU gettext .po format text into message mappings
    pub fn parse(content: &str, fallback_lang: &str) -> Self {
        let mut messages = HashMap::new();
        let mut language = fallback_lang.to_string();

        let mut current_msgid: Option<String> = None;
        let mut current_msgstr: Option<String> = None;
        let mut in_msgid = false;
        let mut in_msgstr = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("msgid ") {
                // Save previous entry
                if let (Some(id), Some(str_val)) = (current_msgid.take(), current_msgstr.take()) {
                    if id.is_empty() {
                        // Header block: detect "Language: xx"
                        for h in str_val.lines() {
                            if let Some(rest) = h.strip_prefix("Language:") {
                                language = rest.trim().to_lowercase();
                            }
                        }
                    } else {
                        messages.insert(id, str_val);
                    }
                }

                let raw = trimmed.trim_start_matches("msgid ").trim();
                current_msgid = Some(unescape_po_str(raw));
                in_msgid = true;
                in_msgstr = false;
            } else if trimmed.starts_with("msgstr ") {
                let raw = trimmed.trim_start_matches("msgstr ").trim();
                current_msgstr = Some(unescape_po_str(raw));
                in_msgid = false;
                in_msgstr = true;
            } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
                let unescaped = unescape_po_str(trimmed);
                if in_msgid {
                    if let Some(ref mut id) = current_msgid {
                        id.push_str(&unescaped);
                    }
                } else if in_msgstr {
                    if let Some(ref mut str_val) = current_msgstr {
                        str_val.push_str(&unescaped);
                    }
                }
            }
        }

        // Save last entry
        if let (Some(id), Some(str_val)) = (current_msgid, current_msgstr) {
            if id.is_empty() {
                for h in str_val.lines() {
                    if let Some(rest) = h.strip_prefix("Language:") {
                        language = rest.trim().to_lowercase();
                    }
                }
            } else {
                messages.insert(id, str_val);
            }
        }

        Self { language, messages }
    }

    pub fn get(&self, msgid: &str) -> Option<&str> {
        self.messages.get(msgid).map(|s| s.as_str())
    }
}

fn unescape_po_str(s: &str) -> String {
    let inner = s.trim_matches('"');
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Global I18n Manager maintaining loaded .po catalogs
pub struct I18n {
    catalogs: HashMap<String, PoCatalog>,
    locales_dir: Option<PathBuf>,
}

impl I18n {
    pub fn new() -> Self {
        let mut catalogs = HashMap::new();
        // Register embedded defaults
        let en_cat = PoCatalog::parse(FALLBACK_EN, "en");
        let hu_cat = PoCatalog::parse(FALLBACK_HU, "hu");
        catalogs.insert("en".to_string(), en_cat);
        catalogs.insert("hu".to_string(), hu_cat);

        Self {
            catalogs,
            locales_dir: None,
        }
    }

    /// Loads or reloads all *.po catalogs from the given directory on disk.
    /// If the directory exists, disk files overwrite embedded defaults.
    pub fn load_from_dir(&mut self, dir: &Path) {
        self.locales_dir = Some(dir.to_path_buf());
        if !dir.exists() || !dir.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "po") {
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_lowercase();

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let cat = PoCatalog::parse(&content, &stem);
                        let lang = if !cat.language.is_empty() {
                            cat.language.clone()
                        } else {
                            stem
                        };
                        self.catalogs.insert(lang, cat);
                    }
                }
            }
        }
    }

    /// Translates a `msgid` for a specified language code.
    /// Falls back to English catalog, then to `msgid` itself.
    pub fn tr(&self, lang: &str, msgid: &str) -> String {
        let lang_lower = lang.to_lowercase();
        if let Some(cat) = self.catalogs.get(&lang_lower) {
            if let Some(trans) = cat.get(msgid) {
                if !trans.is_empty() {
                    return trans.to_string();
                }
            }
        }

        // Fallback to English catalog
        if lang_lower != "en" {
            if let Some(en_cat) = self.catalogs.get("en") {
                if let Some(trans) = en_cat.get(msgid) {
                    if !trans.is_empty() {
                        return trans.to_string();
                    }
                }
            }
        }

        // Final fallback: return the original msgid
        msgid.to_string()
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

// Global shared instance
static GLOBAL_I18N: RwLock<Option<Arc<I18n>>> = RwLock::new(None);

pub fn get_i18n() -> Arc<I18n> {
    {
        let read = GLOBAL_I18N.read();
        if let Some(ref instance) = *read {
            return Arc::clone(instance);
        }
    }

    let mut write = GLOBAL_I18N.write();
    if let Some(ref instance) = *write {
        return Arc::clone(instance);
    }

    let mut i18n = I18n::new();
    // Default load attempt from ./locales
    i18n.load_from_dir(Path::new("locales"));
    let arc = Arc::new(i18n);
    *write = Some(Arc::clone(&arc));
    arc
}

/// Initializes the global i18n manager from a custom locales directory
pub fn init_i18n(locales_dir: &Path) {
    let mut write = GLOBAL_I18N.write();
    let mut i18n = I18n::new();
    i18n.load_from_dir(locales_dir);
    *write = Some(Arc::new(i18n));
}

/// Supported interface languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    Hu,
}

impl Locale {
    /// Language code string ("EN" or "HU")
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "EN",
            Locale::Hu => "HU",
        }
    }

    /// Lowercase ISO language code ("en" or "hu")
    pub fn lang_code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Hu => "hu",
        }
    }

    /// Construct Locale from code ("en", "hu", "EN", "HU")
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "hu" => Locale::Hu,
            _ => Locale::En,
        }
    }

    /// Toggle to the other supported locale
    pub fn toggle(&self) -> Self {
        match self {
            Locale::En => Locale::Hu,
            Locale::Hu => Locale::En,
        }
    }

    /// Dynamic gettext translation lookup
    pub fn tr(&self, msgid: &str) -> String {
        get_i18n().tr(self.lang_code(), msgid)
    }

    pub fn search_everywhere(&self) -> String {
        self.tr("Search Files & Symbols (Shift+Shift / Ctrl+P)")
    }

    pub fn run_btn(&self) -> String {
        self.tr("Run")
    }

    pub fn toggle_sidebar(&self) -> String {
        self.tr("Sidebar")
    }

    pub fn toggle_panel(&self) -> String {
        self.tr("Panel")
    }

    pub fn explorer_title(&self) -> String {
        self.tr("EXPLORER")
    }

    pub fn search_title(&self) -> String {
        self.tr("SEARCH IN FILES")
    }

    pub fn doc_tutor_title(&self) -> String {
        self.tr("EDU DOC TUTOR")
    }

    pub fn doc_tutor_subtitle(&self) -> String {
        self.tr("Quick reference and help cards for students")
    }

    pub fn ai_assistant_title(&self) -> String {
        self.tr("AI ASSISTANT")
    }

    pub fn settings_title(&self) -> String {
        self.tr("SETTINGS & POLICIES")
    }

    pub fn terminal_tab(&self) -> String {
        self.tr("TERMINAL")
    }

    pub fn problems_tab(&self) -> String {
        self.tr("PROBLEMS")
    }

    pub fn hints_tab(&self) -> String {
        self.tr("DOC TUTOR HINTS")
    }

    pub fn ready_status(&self) -> String {
        self.tr("Ready")
    }

    pub fn edu_mode_pill(&self) -> String {
        self.tr("EDU Mode (AI Disabled)")
    }

    pub fn ai_mode_pill(&self) -> String {
        self.tr("AI Ready (Local LLM)")
    }

    pub fn edu_subtitle(&self) -> String {
        self.tr("University Edition - No AI Cheating, Just Pure Learning")
    }

    pub fn ai_subtitle(&self) -> String {
        self.tr("AI Edition - Integrated Local LLM & Smart Assistance")
    }

    pub fn shortcut_search_desc(&self) -> String {
        self.tr("Search Everywhere (Files, Symbols, Commands)")
    }

    pub fn shortcut_sidebar_desc(&self) -> String {
        self.tr("Toggle Primary Side Bar")
    }

    pub fn shortcut_terminal_desc(&self) -> String {
        self.tr("Toggle Terminal / Bottom Panel")
    }

    pub fn shortcut_run_desc(&self) -> String {
        self.tr("Execute active file / project")
    }

    pub fn shortcut_help_desc(&self) -> String {
        self.tr("Open University Doc Tutor / Language Help")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_po_parser() {
        let po_data = r#"
# Test PO
msgid ""
msgstr ""
"Language: hu\n"

msgid "Run"
msgstr "Futtatás"

msgid "Multiline test"
msgstr ""
"Több soros "
"szöveg teszt"
"#;

        let cat = PoCatalog::parse(po_data, "hu");
        assert_eq!(cat.language, "hu");
        assert_eq!(cat.get("Run"), Some("Futtatás"));
        assert_eq!(cat.get("Multiline test"), Some("Több soros szöveg teszt"));
    }

    #[test]
    fn test_locale_translations() {
        let en = Locale::En;
        let hu = Locale::Hu;

        assert_eq!(en.run_btn(), "Run");
        assert_eq!(hu.run_btn(), "Futtatás");
        assert_eq!(hu.explorer_title(), "INTÉZŐ");
        assert_eq!(en.explorer_title(), "EXPLORER");
    }
}
