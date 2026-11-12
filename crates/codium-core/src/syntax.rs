//! # Syntax Highlighting & Theme System
//!
//! Provides tokenization, syntax definitions across languages (Rust, Python, JS/TS,
//! C/C++, JSON, TOML, Markdown, etc.), multiline comment/string tracking, and
//! VS Code-compatible color theme parsing with default Monokai palette.

use std::collections::HashMap;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// High-level syntactic token category
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Keyword,
    Storage,
    Type,
    Function,
    Macro,
    String,
    Comment,
    DocComment,
    Number,
    Boolean,
    Constant,
    Operator,
    Punctuation,
    Variable,
    Parameter,
    Attribute,
    Tag,
    Plain,
}

/// A token span with byte range in the line and token type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSpan {
    pub range: Range<usize>,
    pub token_type: TokenType,
}

/// State maintained across lines for multiline constructs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultilineState {
    Normal,
    BlockComment,
    DocComment,
    TripleDoubleQuote,
    TripleSingleQuote,
}

/// Supported programming / markup language
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyntaxLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Go,
    Html,
    Css,
    Json,
    Toml,
    Yaml,
    Markdown,
    Shell,
    Sql,
    PlainText,
}

impl SyntaxLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        let clean = ext.trim().trim_start_matches('.').to_lowercase();
        match clean.as_str() {
            "rs" => Self::Rust,
            "py" | "pyw" => Self::Python,
            "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => Self::TypeScript,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Self::Cpp,
            "go" => Self::Go,
            "html" | "htm" | "xml" | "svg" => Self::Html,
            "css" | "scss" | "less" => Self::Css,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "md" | "markdown" => Self::Markdown,
            "sh" | "bash" | "zsh" => Self::Shell,
            "sql" => Self::Sql,
            _ => Self::PlainText,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Go => "Go",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
            Self::Markdown => "Markdown",
            Self::Shell => "Shell",
            Self::Sql => "SQL",
            Self::PlainText => "Plain Text",
        }
    }
}

/// Token styling parameters
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenStyle {
    pub foreground: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
}

impl TokenStyle {
    pub fn new(fg: impl Into<String>) -> Self {
        Self {
            foreground: fg.into(),
            font_style: None,
        }
    }

    pub fn with_style(fg: impl Into<String>, style: impl Into<String>) -> Self {
        Self {
            foreground: fg.into(),
            font_style: Some(style.into()),
        }
    }
}

/// UI & Editor core colors in a theme
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(default = "default_editor_bg")]
    pub background: String,
    #[serde(default = "default_editor_fg")]
    pub foreground: String,
    #[serde(default = "default_line_highlight")]
    pub line_highlight: String,
    #[serde(default = "default_selection")]
    pub selection: String,
    #[serde(default = "default_cursor")]
    pub cursor: String,
    #[serde(default = "default_gutter_dim")]
    pub gutter_dim: String,
    #[serde(default = "default_gutter_active")]
    pub gutter_active: String,
    #[serde(default = "default_border")]
    pub border: String,
}

fn default_editor_bg() -> String { "#272822".to_string() }
fn default_editor_fg() -> String { "#F8F8F2".to_string() }
fn default_line_highlight() -> String { "#3E3D32".to_string() }
fn default_selection() -> String { "#49483E".to_string() }
fn default_cursor() -> String { "#F8F8F0".to_string() }
fn default_gutter_dim() -> String { "#75715E".to_string() }
fn default_gutter_active() -> String { "#F8F8F2".to_string() }
fn default_border() -> String { "#3E3D32".to_string() }

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: default_editor_bg(),
            foreground: default_editor_fg(),
            line_highlight: default_line_highlight(),
            selection: default_selection(),
            cursor: default_cursor(),
            gutter_dim: default_gutter_dim(),
            gutter_active: default_gutter_active(),
            border: default_border(),
        }
    }
}

/// Complete syntax color theme
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxTheme {
    pub name: String,
    #[serde(default)]
    pub colors: ThemeColors,
    #[serde(default)]
    pub token_colors: HashMap<TokenType, TokenStyle>,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::monokai_default()
    }
}

impl SyntaxTheme {
    /// Get the hex color string for a token type if present
    pub fn color_for_token(&self, token_type: TokenType) -> Option<&str> {
        self.token_colors.get(&token_type).map(|s| s.foreground.as_str())
    }

    /// Get the full style for a token type
    pub fn style_for_token(&self, token_type: TokenType) -> Option<&TokenStyle> {
        self.token_colors.get(&token_type)
    }

    /// Available built-in themes
    pub fn available_themes() -> &'static [&'static str] {
        &["monokai", "vscode_dark", "one_dark", "solarized_dark"]
    }

    /// Retrieve a built-in theme by name
    pub fn built_in_theme(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "monokai" => Some(Self::monokai_default()),
            "vscode_dark" | "vscode" | "dark" => Some(Self::vscode_dark_default()),
            "one_dark" | "onedark" => Some(Self::one_dark_default()),
            "solarized_dark" | "solarized" => Some(Self::solarized_dark_default()),
            _ => None,
        }
    }

    /// Standard Monokai theme (Default)
    pub fn monokai_default() -> Self {
        let mut tokens = HashMap::new();
        // Keywords & Storage
        tokens.insert(TokenType::Keyword, TokenStyle::new("#F92672"));
        tokens.insert(TokenType::Storage, TokenStyle::with_style("#66D9EF", "italic"));
        tokens.insert(TokenType::Operator, TokenStyle::new("#F92672"));
        tokens.insert(TokenType::Punctuation, TokenStyle::new("#F8F8F2"));

        // Strings & Chars
        tokens.insert(TokenType::String, TokenStyle::new("#E6DB74"));
        tokens.insert(TokenType::Tag, TokenStyle::new("#F92672"));
        tokens.insert(TokenType::Attribute, TokenStyle::new("#A6E22E"));

        // Numbers & Constants
        tokens.insert(TokenType::Number, TokenStyle::new("#AE81FF"));
        tokens.insert(TokenType::Boolean, TokenStyle::new("#AE81FF"));
        tokens.insert(TokenType::Constant, TokenStyle::new("#AE81FF"));

        // Functions & Macros
        tokens.insert(TokenType::Function, TokenStyle::new("#A6E22E"));
        tokens.insert(TokenType::Macro, TokenStyle::new("#66D9EF"));

        // Types & Structs
        tokens.insert(TokenType::Type, TokenStyle::new("#66D9EF"));

        // Variables & Parameters
        tokens.insert(TokenType::Variable, TokenStyle::new("#F8F8F2"));
        tokens.insert(TokenType::Parameter, TokenStyle::with_style("#FD971F", "italic"));

        // Comments
        tokens.insert(TokenType::Comment, TokenStyle::with_style("#75715E", "italic"));
        tokens.insert(TokenType::DocComment, TokenStyle::with_style("#8F8B7B", "italic"));
        tokens.insert(TokenType::Plain, TokenStyle::new("#F8F8F2"));

        Self {
            name: "Monokai".to_string(),
            colors: ThemeColors {
                background: "#272822".to_string(),
                foreground: "#F8F8F2".to_string(),
                line_highlight: "#3E3D32".to_string(),
                selection: "#49483E".to_string(),
                cursor: "#F8F8F0".to_string(),
                gutter_dim: "#75715E".to_string(),
                gutter_active: "#F8F8F2".to_string(),
                border: "#3E3D32".to_string(),
            },
            token_colors: tokens,
        }
    }

    /// VS Code Dark+ default theme
    pub fn vscode_dark_default() -> Self {
        let mut tokens = HashMap::new();
        tokens.insert(TokenType::Keyword, TokenStyle::new("#C586C0"));
        tokens.insert(TokenType::Storage, TokenStyle::new("#569CD6"));
        tokens.insert(TokenType::Operator, TokenStyle::new("#D4D4D4"));
        tokens.insert(TokenType::Punctuation, TokenStyle::new("#D4D4D4"));
        tokens.insert(TokenType::String, TokenStyle::new("#CE9178"));
        tokens.insert(TokenType::Tag, TokenStyle::new("#569CD6"));
        tokens.insert(TokenType::Attribute, TokenStyle::new("#9CDCFE"));
        tokens.insert(TokenType::Number, TokenStyle::new("#B5CEA8"));
        tokens.insert(TokenType::Boolean, TokenStyle::new("#569CD6"));
        tokens.insert(TokenType::Constant, TokenStyle::new("#4FC1FF"));
        tokens.insert(TokenType::Function, TokenStyle::new("#DCDCAA"));
        tokens.insert(TokenType::Macro, TokenStyle::new("#DCDCAA"));
        tokens.insert(TokenType::Type, TokenStyle::new("#4EC9B0"));
        tokens.insert(TokenType::Variable, TokenStyle::new("#9CDCFE"));
        tokens.insert(TokenType::Parameter, TokenStyle::new("#9CDCFE"));
        tokens.insert(TokenType::Comment, TokenStyle::with_style("#6A9955", "italic"));
        tokens.insert(TokenType::DocComment, TokenStyle::with_style("#608B4E", "italic"));
        tokens.insert(TokenType::Plain, TokenStyle::new("#D4D4D4"));

        Self {
            name: "VS Code Dark+".to_string(),
            colors: ThemeColors {
                background: "#1E1E1E".to_string(),
                foreground: "#D4D4D4".to_string(),
                line_highlight: "#282828".to_string(),
                selection: "#264F78".to_string(),
                cursor: "#AEAFAD".to_string(),
                gutter_dim: "#858585".to_string(),
                gutter_active: "#C6C6C6".to_string(),
                border: "#383838".to_string(),
            },
            token_colors: tokens,
        }
    }

    /// One Dark theme
    pub fn one_dark_default() -> Self {
        let mut tokens = HashMap::new();
        tokens.insert(TokenType::Keyword, TokenStyle::new("#C678DD"));
        tokens.insert(TokenType::Storage, TokenStyle::new("#C678DD"));
        tokens.insert(TokenType::Operator, TokenStyle::new("#56B6C2"));
        tokens.insert(TokenType::Punctuation, TokenStyle::new("#ABB2BF"));
        tokens.insert(TokenType::String, TokenStyle::new("#98C379"));
        tokens.insert(TokenType::Tag, TokenStyle::new("#E06C75"));
        tokens.insert(TokenType::Attribute, TokenStyle::new("#D19A66"));
        tokens.insert(TokenType::Number, TokenStyle::new("#D19A66"));
        tokens.insert(TokenType::Boolean, TokenStyle::new("#D19A66"));
        tokens.insert(TokenType::Constant, TokenStyle::new("#E5C07B"));
        tokens.insert(TokenType::Function, TokenStyle::new("#61AFEF"));
        tokens.insert(TokenType::Macro, TokenStyle::new("#61AFEF"));
        tokens.insert(TokenType::Type, TokenStyle::new("#E5C07B"));
        tokens.insert(TokenType::Variable, TokenStyle::new("#E06C75"));
        tokens.insert(TokenType::Parameter, TokenStyle::new("#ABB2BF"));
        tokens.insert(TokenType::Comment, TokenStyle::with_style("#5C6370", "italic"));
        tokens.insert(TokenType::DocComment, TokenStyle::with_style("#6B727F", "italic"));
        tokens.insert(TokenType::Plain, TokenStyle::new("#ABB2BF"));

        Self {
            name: "One Dark".to_string(),
            colors: ThemeColors {
                background: "#282C34".to_string(),
                foreground: "#ABB2BF".to_string(),
                line_highlight: "#2C313A".to_string(),
                selection: "#3E4451".to_string(),
                cursor: "#528BFF".to_string(),
                gutter_dim: "#5C6370".to_string(),
                gutter_active: "#ABB2BF".to_string(),
                border: "#181A1F".to_string(),
            },
            token_colors: tokens,
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark_default() -> Self {
        let mut tokens = HashMap::new();
        tokens.insert(TokenType::Keyword, TokenStyle::new("#859900"));
        tokens.insert(TokenType::Storage, TokenStyle::new("#859900"));
        tokens.insert(TokenType::Operator, TokenStyle::new("#859900"));
        tokens.insert(TokenType::Punctuation, TokenStyle::new("#839496"));
        tokens.insert(TokenType::String, TokenStyle::new("#2AA198"));
        tokens.insert(TokenType::Tag, TokenStyle::new("#268BD2"));
        tokens.insert(TokenType::Attribute, TokenStyle::new("#B58900"));
        tokens.insert(TokenType::Number, TokenStyle::new("#D33682"));
        tokens.insert(TokenType::Boolean, TokenStyle::new("#B58900"));
        tokens.insert(TokenType::Constant, TokenStyle::new("#CB4B16"));
        tokens.insert(TokenType::Function, TokenStyle::new("#268BD2"));
        tokens.insert(TokenType::Macro, TokenStyle::new("#268BD2"));
        tokens.insert(TokenType::Type, TokenStyle::new("#B58900"));
        tokens.insert(TokenType::Variable, TokenStyle::new("#839496"));
        tokens.insert(TokenType::Parameter, TokenStyle::new("#268BD2"));
        tokens.insert(TokenType::Comment, TokenStyle::with_style("#586E75", "italic"));
        tokens.insert(TokenType::DocComment, TokenStyle::with_style("#657B83", "italic"));
        tokens.insert(TokenType::Plain, TokenStyle::new("#839496"));

        Self {
            name: "Solarized Dark".to_string(),
            colors: ThemeColors {
                background: "#002B36".to_string(),
                foreground: "#839496".to_string(),
                line_highlight: "#073642".to_string(),
                selection: "#073642".to_string(),
                cursor: "#839496".to_string(),
                gutter_dim: "#586E75".to_string(),
                gutter_active: "#93A1A1".to_string(),
                border: "#073642".to_string(),
            },
            token_colors: tokens,
        }
    }

    /// Parse theme from JSON (supports both native key-value and VS Code tokenColors schema)
    pub fn from_json(json_str: &str) -> Result<Self, anyhow::Error> {
        let val: serde_json::Value = serde_json::from_str(json_str)?;

        let name = val.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Custom Theme")
            .to_string();

        let mut colors = ThemeColors::default();
        if let Some(c_obj) = val.get("colors").and_then(|v| v.as_object()) {
            if let Some(bg) = c_obj.get("editor.background").or_else(|| c_obj.get("background")).and_then(|v| v.as_str()) {
                colors.background = bg.to_string();
            }
            if let Some(fg) = c_obj.get("editor.foreground").or_else(|| c_obj.get("foreground")).and_then(|v| v.as_str()) {
                colors.foreground = fg.to_string();
            }
            if let Some(lh) = c_obj.get("editor.lineHighlightBackground").or_else(|| c_obj.get("line_highlight")).and_then(|v| v.as_str()) {
                colors.line_highlight = lh.to_string();
            }
            if let Some(sel) = c_obj.get("editor.selectionBackground").or_else(|| c_obj.get("selection")).and_then(|v| v.as_str()) {
                colors.selection = sel.to_string();
            }
            if let Some(cur) = c_obj.get("editorCursor.foreground").or_else(|| c_obj.get("cursor")).and_then(|v| v.as_str()) {
                colors.cursor = cur.to_string();
            }
            if let Some(gd) = c_obj.get("editorLineNumber.foreground").or_else(|| c_obj.get("gutter_dim")).and_then(|v| v.as_str()) {
                colors.gutter_dim = gd.to_string();
            }
            if let Some(ga) = c_obj.get("editorLineNumber.activeForeground").or_else(|| c_obj.get("gutter_active")).and_then(|v| v.as_str()) {
                colors.gutter_active = ga.to_string();
            }
            if let Some(b) = c_obj.get("editorIndentGuide.background").or_else(|| c_obj.get("border")).and_then(|v| v.as_str()) {
                colors.border = b.to_string();
            }
        }

        let mut token_colors = HashMap::new();

        if let Some(tc_arr) = val.get("tokenColors").and_then(|v| v.as_array()) {
            // VS Code standard array format: [ { scope: [...], settings: { foreground: "#...", fontStyle: "..." } } ]
            for entry in tc_arr {
                let settings = match entry.get("settings").and_then(|s| s.as_object()) {
                    Some(s) => s,
                    None => continue,
                };
                let fg = match settings.get("foreground").and_then(|f| f.as_str()) {
                    Some(f) => f,
                    None => continue,
                };
                let font_style = settings.get("fontStyle").and_then(|s| s.as_str()).map(|s| s.to_string());
                let style = TokenStyle {
                    foreground: fg.to_string(),
                    font_style,
                };

                let scopes: Vec<String> = if let Some(s_str) = entry.get("scope").and_then(|s| s.as_str()) {
                    s_str.split(',').map(|s| s.trim().to_string()).collect()
                } else if let Some(s_arr) = entry.get("scope").and_then(|s| s.as_array()) {
                    s_arr.iter().filter_map(|s| s.as_str().map(|str_val| str_val.trim().to_string())).collect()
                } else {
                    Vec::new()
                };

                for scope in scopes {
                    for tt in map_vscode_scope_to_token_types(&scope) {
                        token_colors.insert(tt, style.clone());
                    }
                }
            }
        } else if let Some(tc_obj) = val.get("token_colors").and_then(|v| v.as_object()) {
            // Native format: { "keyword": { "foreground": "#..." }, ... }
            for (k, v) in tc_obj {
                if let Ok(tt) = serde_json::from_value::<TokenType>(serde_json::Value::String(k.clone())) {
                    if let Ok(ts) = serde_json::from_value::<TokenStyle>(v.clone()) {
                        token_colors.insert(tt, ts);
                    }
                }
            }
        }

        // Fill any missing essential tokens from Monokai default
        let monokai = Self::monokai_default();
        for (tt, default_style) in monokai.token_colors {
            token_colors.entry(tt).or_insert(default_style);
        }

        Ok(Self {
            name,
            colors,
            token_colors,
        })
    }

    /// Save the theme to disk as readable JSON
    pub fn save_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Load theme profile by name, checking settings/themes/<name>.json,
    /// creating fallback if missing, and falling back to built-in.
    pub fn load_profile(theme_name: &str) -> Self {
        let clean = theme_name.trim();

        let candidate_paths = [
            PathBuf::from(format!("settings/themes/{clean}.json")),
            PathBuf::from(format!("themes/{clean}.json")),
            PathBuf::from(format!("resources/themes/{clean}.json")),
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
            let target_path = PathBuf::from(format!("settings/themes/{clean}.json"));
            let _ = built_in.save_to_file(&target_path);
            return built_in;
        }

        // Default fallback: Monokai
        let default_theme = Self::monokai_default();
        let target_path = PathBuf::from("settings/themes/monokai.json");
        let _ = default_theme.save_to_file(&target_path);
        default_theme
    }
}

fn map_vscode_scope_to_token_types(scope: &str) -> Vec<TokenType> {
    let s = scope.to_lowercase();
    let mut out = Vec::new();
    if s.contains("comment") {
        out.push(TokenType::Comment);
        if s.contains("documentation") || s.contains("doc") {
            out.push(TokenType::DocComment);
        }
    } else if s.contains("string") {
        out.push(TokenType::String);
    } else if s.contains("numeric") || s.contains("number") {
        out.push(TokenType::Number);
    } else if s.contains("boolean") {
        out.push(TokenType::Boolean);
    } else if s.contains("constant") {
        out.push(TokenType::Constant);
    } else if s.contains("keyword.control") || s.contains("keyword") {
        out.push(TokenType::Keyword);
    } else if s.contains("storage.type") || s.contains("storage") {
        out.push(TokenType::Storage);
    } else if s.contains("function") || s.contains("method") {
        out.push(TokenType::Function);
    } else if s.contains("type") || s.contains("class") || s.contains("struct") || s.contains("interface") {
        out.push(TokenType::Type);
    } else if s.contains("variable.parameter") || s.contains("parameter") {
        out.push(TokenType::Parameter);
    } else if s.contains("variable") {
        out.push(TokenType::Variable);
    } else if s.contains("operator") {
        out.push(TokenType::Operator);
    } else if s.contains("punctuation") {
        out.push(TokenType::Punctuation);
    } else if s.contains("tag") {
        out.push(TokenType::Tag);
    } else if s.contains("attribute") {
        out.push(TokenType::Attribute);
    }
    out
}

/// Tokenizer output for a single line
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenizeResult {
    pub tokens: Vec<TokenSpan>,
    pub end_state: MultilineState,
}

/// Fast, robust lexical analyzer for a single line of code
pub fn tokenize_line(
    language: SyntaxLanguage,
    line: &str,
    start_state: MultilineState,
) -> TokenizeResult {
    let mut tokens = Vec::new();
    let mut state = start_state;
    let mut idx = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();

    // 1. Handle multiline continuation from previous line
    if state == MultilineState::BlockComment || state == MultilineState::DocComment {
        let is_doc = state == MultilineState::DocComment;
        let token_type = if is_doc { TokenType::DocComment } else { TokenType::Comment };
        let end_marker = match language {
            SyntaxLanguage::Html | SyntaxLanguage::Markdown => "-->",
            _ => "*/",
        };

        if let Some(pos) = line.find(end_marker) {
            let end_idx = pos + end_marker.len();
            tokens.push(TokenSpan {
                range: 0..end_idx,
                token_type,
            });
            idx = end_idx;
            state = MultilineState::Normal;
        } else {
            tokens.push(TokenSpan {
                range: 0..len,
                token_type,
            });
            return TokenizeResult {
                tokens,
                end_state: state,
            };
        }
    } else if state == MultilineState::TripleDoubleQuote || state == MultilineState::TripleSingleQuote {
        let marker = if state == MultilineState::TripleDoubleQuote { "\"\"\"" } else { "'''" };
        if let Some(pos) = line.find(marker) {
            let end_idx = pos + 3;
            tokens.push(TokenSpan {
                range: 0..end_idx,
                token_type: TokenType::String,
            });
            idx = end_idx;
            state = MultilineState::Normal;
        } else {
            tokens.push(TokenSpan {
                range: 0..len,
                token_type: TokenType::String,
            });
            return TokenizeResult {
                tokens,
                end_state: state,
            };
        }
    }

    // 2. Tokenize remainder of line
    while idx < len {
        let b = bytes[idx];

        // Whitespace
        if b.is_ascii_whitespace() {
            idx += 1;
            continue;
        }

        // Line and block comments
        if idx + 1 < len && b == b'/' && bytes[idx + 1] == b'/' {
            let is_doc = idx + 2 < len && (bytes[idx + 2] == b'/' || bytes[idx + 2] == b'!');
            let token_type = if is_doc { TokenType::DocComment } else { TokenType::Comment };
            tokens.push(TokenSpan {
                range: idx..len,
                token_type,
            });
            break;
        }

        if idx + 1 < len && b == b'/' && bytes[idx + 1] == b'*' {
            let is_doc = idx + 2 < len && (bytes[idx + 2] == b'*' || bytes[idx + 2] == b'!');
            let token_type = if is_doc { TokenType::DocComment } else { TokenType::Comment };
            if let Some(end_rel) = line[idx + 2..].find("*/") {
                let end_idx = idx + 2 + end_rel + 2;
                tokens.push(TokenSpan {
                    range: idx..end_idx,
                    token_type,
                });
                idx = end_idx;
                continue;
            } else {
                tokens.push(TokenSpan {
                    range: idx..len,
                    token_type,
                });
                state = if is_doc { MultilineState::DocComment } else { MultilineState::BlockComment };
                break;
            }
        }

        // Scripting line comments: # in Python, Shell, TOML, YAML
        if b == b'#' && (language == SyntaxLanguage::Python
            || language == SyntaxLanguage::Shell
            || language == SyntaxLanguage::Toml
            || language == SyntaxLanguage::Yaml)
        {
            tokens.push(TokenSpan {
                range: idx..len,
                token_type: TokenType::Comment,
            });
            break;
        }

        // HTML/XML Comments: <!-- ... -->
        if (language == SyntaxLanguage::Html || language == SyntaxLanguage::Markdown)
            && line[idx..].starts_with("<!--")
        {
            if let Some(end_rel) = line[idx + 4..].find("-->") {
                let end_idx = idx + 4 + end_rel + 3;
                tokens.push(TokenSpan {
                    range: idx..end_idx,
                    token_type: TokenType::Comment,
                });
                idx = end_idx;
                continue;
            } else {
                tokens.push(TokenSpan {
                    range: idx..len,
                    token_type: TokenType::Comment,
                });
                state = MultilineState::BlockComment;
                break;
            }
        }

        // Rust Attributes: #[derive(...)] or #![no_std]
        if language == SyntaxLanguage::Rust && b == b'#' && idx + 1 < len && (bytes[idx + 1] == b'[' || (idx + 2 < len && bytes[idx + 1] == b'!' && bytes[idx + 2] == b'[')) {
            let start = idx;
            idx += 2;
            let mut depth = 1;
            while idx < len && depth > 0 {
                if bytes[idx] == b'[' {
                    depth += 1;
                } else if bytes[idx] == b']' {
                    depth -= 1;
                }
                idx += 1;
            }
            tokens.push(TokenSpan {
                range: start..idx,
                token_type: TokenType::Attribute,
            });
            continue;
        }

        // Python decorators / TS decorators: @identifier
        if (language == SyntaxLanguage::Python || language == SyntaxLanguage::TypeScript) && b == b'@' {
            let start = idx;
            idx += 1;
            while idx < len && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'_' || bytes[idx] == b'.') {
                idx += 1;
            }
            tokens.push(TokenSpan {
                range: start..idx,
                token_type: TokenType::Attribute,
            });
            continue;
        }

        // Strings: Triple Quotes in Python
        if language == SyntaxLanguage::Python && (line[idx..].starts_with("\"\"\"") || line[idx..].starts_with("'''")) {
            let marker = &line[idx..idx + 3];
            let start = idx;
            idx += 3;
            if let Some(pos) = line[idx..].find(marker) {
                let end_idx = idx + pos + 3;
                tokens.push(TokenSpan {
                    range: start..end_idx,
                    token_type: TokenType::String,
                });
                idx = end_idx;
                continue;
            } else {
                tokens.push(TokenSpan {
                    range: start..len,
                    token_type: TokenType::String,
                });
                state = if marker == "\"\"\"" {
                    MultilineState::TripleDoubleQuote
                } else {
                    MultilineState::TripleSingleQuote
                };
                break;
            }
        }

        // Standard Strings: "...", '...', `...`, r#"..."#
        if b == b'"' || b == b'\'' || b == b'`' {
            let quote = b;
            let start = idx;
            idx += 1;
            let mut escaped = false;
            let mut closed = false;
            while idx < len {
                let cb = bytes[idx];
                if escaped {
                    escaped = false;
                } else if cb == b'\\' {
                    escaped = true;
                } else if cb == quote {
                    idx += 1;
                    closed = true;
                    break;
                }
                idx += 1;
            }
            tokens.push(TokenSpan {
                range: start..idx,
                token_type: TokenType::String,
            });
            if !closed && quote == b'`' {
                // Template strings may be multiline
            }
            continue;
        }

        // Rust Raw Strings: r#"..."#
        if language == SyntaxLanguage::Rust && b == b'r' && idx + 1 < len && (bytes[idx + 1] == b'"' || bytes[idx + 1] == b'#') {
            let start = idx;
            idx += 1;
            let mut hashes = 0;
            while idx < len && bytes[idx] == b'#' {
                hashes += 1;
                idx += 1;
            }
            if idx < len && bytes[idx] == b'"' {
                idx += 1;
                let closing = format!("\"{}", "#".repeat(hashes));
                if let Some(pos) = line[idx..].find(&closing) {
                    idx += pos + closing.len();
                } else {
                    idx = len;
                }
                tokens.push(TokenSpan {
                    range: start..idx,
                    token_type: TokenType::String,
                });
                continue;
            }
            idx = start; // Fallback to normal identifier
        }

        // Numbers: hex, binary, floats, decimals, suffixes
        if b.is_ascii_digit() || (b == b'.' && idx + 1 < len && bytes[idx + 1].is_ascii_digit()) {
            let start = idx;
            idx += 1;
            if idx < len && (bytes[idx] == b'x' || bytes[idx] == b'X' || bytes[idx] == b'b' || bytes[idx] == b'B' || bytes[idx] == b'o' || bytes[idx] == b'O') {
                idx += 1;
                while idx < len && (bytes[idx].is_ascii_hexdigit() || bytes[idx] == b'_') {
                    idx += 1;
                }
            } else {
                while idx < len && (bytes[idx].is_ascii_digit() || bytes[idx] == b'.' || bytes[idx] == b'_' || bytes[idx] == b'e' || bytes[idx] == b'E' || bytes[idx] == b'+' || bytes[idx] == b'-') {
                    if (bytes[idx] == b'+' || bytes[idx] == b'-') && !(bytes[idx - 1] == b'e' || bytes[idx - 1] == b'E') {
                        break;
                    }
                    idx += 1;
                }
            }
            // Optional type suffix (e.g. u32, f64, px, rem)
            while idx < len && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'_' || bytes[idx] == b'%') {
                idx += 1;
            }
            tokens.push(TokenSpan {
                range: start..idx,
                token_type: TokenType::Number,
            });
            continue;
        }

        // Identifiers and Keywords
        if b.is_ascii_alphabetic() || b == b'_' || (b > 127) {
            let start = idx;
            idx += 1;
            while idx < len && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'_' || bytes[idx] > 127) {
                idx += 1;
            }
            let word = &line[start..idx];

            let token_type = classify_word(language, word, line, idx);
            tokens.push(TokenSpan {
                range: start..idx,
                token_type,
            });
            continue;
        }

        // Operators & Punctuation
        let start = idx;
        idx += 1;

        // Two-character operators (==, !=, <=, >=, &&, ||, ->, =>, ::, +=, -=, *=, /=)
        if idx < len {
            let pair = &line[start..start + 2];
            match pair {
                "==" | "!=" | "<=" | ">=" | "&&" | "||" | "->" | "=>" | "::" | "+=" | "-=" | "*=" | "/=" | "<<" | ">>" | ".." => {
                    idx += 1;
                    tokens.push(TokenSpan {
                        range: start..idx,
                        token_type: TokenType::Operator,
                    });
                    continue;
                }
                _ => {}
            }
        }

        let is_punct = matches!(b, b'(' | b')' | b'{' | b'}' | b'[' | b']' | b';' | b',');
        let token_type = if is_punct {
            TokenType::Punctuation
        } else if matches!(b, b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>' | b'&' | b'|' | b'^' | b'!' | b'~' | b'?' | b':' | b'.') {
            TokenType::Operator
        } else {
            TokenType::Plain
        };

        tokens.push(TokenSpan {
            range: start..idx,
            token_type,
        });
    }

    TokenizeResult {
        tokens,
        end_state: state,
    }
}

/// Classify an identifier / keyword token
fn classify_word(language: SyntaxLanguage, word: &str, line: &str, next_idx: usize) -> TokenType {
    // 1. Booleans
    if matches!(word, "true" | "false" | "True" | "False") {
        return TokenType::Boolean;
    }

    // 2. Constants & Nil/Null
    if matches!(word, "null" | "None" | "nil" | "undefined" | "NaN" | "Infinity") {
        return TokenType::Constant;
    }

    // 3. Rust macros (e.g. println!, vec!)
    if language == SyntaxLanguage::Rust && next_idx < line.len() && line.as_bytes()[next_idx] == b'!' {
        return TokenType::Macro;
    }

    // 4. Function call (followed by '(')
    let rest = line[next_idx..].trim_start();
    if rest.starts_with('(') {
        return TokenType::Function;
    }

    // 5. Language keywords
    if is_keyword(language, word) {
        return TokenType::Keyword;
    }

    // 6. Storage type keywords
    if is_storage(language, word) {
        return TokenType::Storage;
    }

    // 7. Primitive and common types
    if is_builtin_type(language, word) {
        return TokenType::Type;
    }

    // 8. PascalCase identifier (likely a Struct, Enum, Class, or Type)
    let first = word.chars().next().unwrap_or('a');
    if first.is_ascii_uppercase() && word.chars().any(|c| c.is_ascii_lowercase()) {
        return TokenType::Type;
    }

    // 9. ALL_CAPS identifier (likely a Constant)
    if word.len() > 1 && word.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()) {
        return TokenType::Constant;
    }

    TokenType::Variable
}

fn is_keyword(lang: SyntaxLanguage, w: &str) -> bool {
    match lang {
        SyntaxLanguage::Rust => matches!(
            w,
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
                | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
                | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
                | "trait" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
                | "dyn"
        ),
        SyntaxLanguage::Python => matches!(
            w,
            "and" | "as" | "assert" | "async" | "await" | "break" | "class" | "continue"
                | "def" | "del" | "elif" | "else" | "except" | "finally" | "for" | "from"
                | "global" | "if" | "import" | "in" | "is" | "lambda" | "nonlocal" | "not"
                | "or" | "pass" | "raise" | "return" | "try" | "while" | "with" | "yield"
        ),
        SyntaxLanguage::JavaScript | SyntaxLanguage::TypeScript => matches!(
            w,
            "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger"
                | "default" | "delete" | "do" | "else" | "export" | "extends" | "finally"
                | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new"
                | "return" | "super" | "switch" | "this" | "throw" | "try" | "typeof"
                | "var" | "void" | "while" | "with" | "yield" | "let" | "async" | "await"
                | "interface" | "type" | "enum" | "implements" | "public" | "private"
                | "protected" | "readonly"
        ),
        SyntaxLanguage::C | SyntaxLanguage::Cpp => matches!(
            w,
            "auto" | "break" | "case" | "const" | "continue" | "default" | "do" | "else"
                | "enum" | "extern" | "for" | "goto" | "if" | "inline" | "register"
                | "restrict" | "return" | "sizeof" | "static" | "struct" | "switch"
                | "typedef" | "union" | "volatile" | "while" | "class" | "namespace"
                | "template" | "typename" | "public" | "private" | "protected" | "new"
                | "delete" | "using" | "virtual" | "override"
        ),
        SyntaxLanguage::Go => matches!(
            w,
            "break" | "default" | "func" | "interface" | "select" | "case" | "defer"
                | "go" | "map" | "struct" | "chan" | "else" | "goto" | "package" | "switch"
                | "const" | "fallthrough" | "if" | "range" | "type" | "continue" | "for"
                | "import" | "return" | "var"
        ),
        SyntaxLanguage::Sql => matches!(
            w,
            "SELECT" | "select" | "FROM" | "from" | "WHERE" | "where" | "INSERT" | "insert"
                | "UPDATE" | "update" | "DELETE" | "delete" | "JOIN" | "join" | "LEFT" | "left"
                | "RIGHT" | "right" | "INNER" | "inner" | "OUTER" | "outer" | "ON" | "on"
                | "GROUP" | "group" | "BY" | "by" | "ORDER" | "order" | "HAVING" | "having"
                | "LIMIT" | "limit" | "OFFSET" | "offset" | "CREATE" | "create" | "TABLE" | "table"
                | "DROP" | "drop" | "ALTER" | "alter" | "INDEX" | "index" | "AS" | "as"
        ),
        SyntaxLanguage::Shell => matches!(
            w,
            "if" | "then" | "else" | "elif" | "fi" | "case" | "esac" | "for" | "while"
                | "until" | "do" | "done" | "in" | "function" | "return" | "exit" | "export"
        ),
        _ => false,
    }
}

fn is_storage(lang: SyntaxLanguage, w: &str) -> bool {
    match lang {
        SyntaxLanguage::Rust => matches!(w, "fn" | "struct" | "enum" | "trait" | "impl" | "type"),
        SyntaxLanguage::Python => matches!(w, "def" | "class"),
        SyntaxLanguage::JavaScript | SyntaxLanguage::TypeScript => matches!(w, "function" | "class"),
        SyntaxLanguage::C | SyntaxLanguage::Cpp => matches!(w, "struct" | "class" | "enum" | "union" | "typedef"),
        SyntaxLanguage::Go => matches!(w, "func" | "type" | "struct" | "interface"),
        _ => false,
    }
}

fn is_builtin_type(lang: SyntaxLanguage, w: &str) -> bool {
    match lang {
        SyntaxLanguage::Rust => matches!(
            w,
            "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "f32" | "f64" | "char" | "str" | "String" | "Option"
                | "Result" | "Vec" | "Box" | "Rc" | "Arc" | "RefCell"
        ),
        SyntaxLanguage::Python => matches!(
            w,
            "int" | "float" | "str" | "bool" | "list" | "dict" | "set" | "tuple" | "bytes" | "object" | "Any" | "Optional"
        ),
        SyntaxLanguage::JavaScript | SyntaxLanguage::TypeScript => matches!(
            w,
            "number" | "string" | "boolean" | "any" | "void" | "never" | "unknown" | "object" | "Symbol" | "BigInt" | "Array" | "Map" | "Set" | "Promise"
        ),
        SyntaxLanguage::C | SyntaxLanguage::Cpp => matches!(
            w,
            "int" | "char" | "float" | "double" | "void" | "short" | "long" | "unsigned" | "signed" | "bool" | "size_t" | "int32_t" | "uint32_t" | "int64_t" | "uint64_t" | "string" | "vector"
        ),
        SyntaxLanguage::Go => matches!(
            w,
            "int" | "int8" | "int16" | "int32" | "int64" | "uint" | "uint8" | "uint16" | "uint32" | "uint64" | "float32" | "float64" | "string" | "bool" | "byte" | "rune" | "error"
        ),
        _ => false,
    }
}

/// Compute line end multiline state quickly without full span retention
pub fn compute_line_end_state(language: SyntaxLanguage, line: &str, start_state: MultilineState) -> MultilineState {
    let res = tokenize_line(language, line, start_state);
    res.end_state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monokai_default() {
        let theme = SyntaxTheme::monokai_default();
        assert_eq!(theme.name, "Monokai");
        assert_eq!(theme.colors.background, "#272822");
        assert_eq!(theme.color_for_token(TokenType::Keyword), Some("#F92672"));
        assert_eq!(theme.color_for_token(TokenType::String), Some("#E6DB74"));
        assert_eq!(theme.color_for_token(TokenType::Comment), Some("#75715E"));
        assert_eq!(theme.color_for_token(TokenType::Function), Some("#A6E22E"));
        assert_eq!(theme.color_for_token(TokenType::Type), Some("#66D9EF"));
    }

    #[test]
    fn test_all_builtin_themes_loadable() {
        for name in SyntaxTheme::available_themes() {
            let theme = SyntaxTheme::built_in_theme(name);
            assert!(theme.is_some(), "Theme {} must exist", name);
            let theme = theme.unwrap();
            assert!(!theme.token_colors.is_empty());
        }
    }

    #[test]
    fn test_rust_tokenization() {
        let code = "fn main() { let count: i32 = 42; println!(\"Hi\"); }";
        let res = tokenize_line(SyntaxLanguage::Rust, code, MultilineState::Normal);
        assert_eq!(res.end_state, MultilineState::Normal);

        let types: Vec<TokenType> = res.tokens.into_iter().map(|t| t.token_type).collect();
        assert!(types.contains(&TokenType::Storage)); // fn
        assert!(types.contains(&TokenType::Function)); // main
        assert!(types.contains(&TokenType::Keyword)); // let
        assert!(types.contains(&TokenType::Type)); // i32
        assert!(types.contains(&TokenType::Number)); // 42
        assert!(types.contains(&TokenType::Macro)); // println!
        assert!(types.contains(&TokenType::String)); // "Hi"
    }

    #[test]
    fn test_multiline_block_comments() {
        let line1 = "/* start comment";
        let res1 = tokenize_line(SyntaxLanguage::Rust, line1, MultilineState::Normal);
        assert_eq!(res1.end_state, MultilineState::BlockComment);

        let line2 = "   middle of comment";
        let res2 = tokenize_line(SyntaxLanguage::Rust, line2, res1.end_state);
        assert_eq!(res2.end_state, MultilineState::BlockComment);

        let line3 = "   end comment */ let x = 1;";
        let res3 = tokenize_line(SyntaxLanguage::Rust, line3, res2.end_state);
        assert_eq!(res3.end_state, MultilineState::Normal);
        assert!(res3.tokens.iter().any(|t| t.token_type == TokenType::Keyword));
    }

    #[test]
    fn test_python_triple_quotes() {
        let line1 = "\"\"\"docstring start";
        let res1 = tokenize_line(SyntaxLanguage::Python, line1, MultilineState::Normal);
        assert_eq!(res1.end_state, MultilineState::TripleDoubleQuote);

        let line2 = "end of docstring\"\"\"";
        let res2 = tokenize_line(SyntaxLanguage::Python, line2, res1.end_state);
        assert_eq!(res2.end_state, MultilineState::Normal);
    }

    #[test]
    fn test_vscode_json_import() {
        let vscode_json = r##"{
            "name": "Custom Monokai",
            "colors": {
                "editor.background": "#272822",
                "editor.foreground": "#f8f8f2"
            },
            "tokenColors": [
                {
                    "scope": ["keyword", "storage"],
                    "settings": {
                        "foreground": "#F92672"
                    }
                },
                {
                    "scope": "string",
                    "settings": {
                        "foreground": "#E6DB74"
                    }
                }
            ]
        }"##;

        let theme = SyntaxTheme::from_json(vscode_json).expect("VS Code JSON must parse");
        assert_eq!(theme.name, "Custom Monokai");
        assert_eq!(theme.colors.background, "#272822");
        assert_eq!(theme.color_for_token(TokenType::Keyword), Some("#F92672"));
        assert_eq!(theme.color_for_token(TokenType::String), Some("#E6DB74"));
    }
}
