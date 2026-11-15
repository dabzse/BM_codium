use std::str::FromStr;
use floem::peniko::Color;

#[derive(Clone, Debug)]
pub struct CodiumTheme {
    pub bg_editor: Color,
    pub bg_sidebar: Color,
    pub bg_activity: Color,
    pub bg_titlebar: Color,
    pub bg_tab_active: Color,
    pub bg_tab_inactive: Color,
    pub bg_panel: Color,
    pub border: Color,
    pub fg_primary: Color,
    pub fg_muted: Color,
    pub accent_blue: Color,
    pub accent_edu: Color,
    pub accent_ai: Color,
    pub hover_bg: Color,
    pub bg_button: Color,
    pub fg_button: Color,
    pub hover_button: Color,
}

impl Default for CodiumTheme {
    fn default() -> Self {
        Self {
            bg_editor: Color::from_str("#1e1e1e").unwrap(),
            bg_sidebar: Color::from_str("#252526").unwrap(),
            bg_activity: Color::from_str("#181818").unwrap(),
            bg_titlebar: Color::from_str("#252526").unwrap(),
            bg_tab_active: Color::from_str("#1e1e1e").unwrap(),
            bg_tab_inactive: Color::from_str("#2d2d2d").unwrap(),
            bg_panel: Color::from_str("#1e1e1e").unwrap(),
            border: Color::from_str("#383838").unwrap(),
            fg_primary: Color::from_str("#cccccc").unwrap(),
            fg_muted: Color::from_str("#858585").unwrap(),
            accent_blue: Color::from_str("#007acc").unwrap(),
            accent_edu: Color::from_str("#1abc9c").unwrap(),
            accent_ai: Color::from_str("#8e44ad").unwrap(),
            hover_bg: Color::from_str("#37373d").unwrap(),
            bg_button: Color::from_str("#0e639c").unwrap(),
            fg_button: Color::from_str("#ffffff").unwrap(),
            hover_button: Color::from_str("#1177bb").unwrap(),
        }
    }
}

impl CodiumTheme {
    /// Classic Monokai Dark theme palette
    pub fn monokai() -> Self {
        Self {
            bg_editor: Color::from_str("#272822").unwrap(),
            bg_sidebar: Color::from_str("#1e1f1c").unwrap(),
            bg_activity: Color::from_str("#181916").unwrap(),
            bg_titlebar: Color::from_str("#1e1f1c").unwrap(),
            bg_tab_active: Color::from_str("#272822").unwrap(),
            bg_tab_inactive: Color::from_str("#1e1f1c").unwrap(),
            bg_panel: Color::from_str("#272822").unwrap(),
            border: Color::from_str("#3e3d32").unwrap(),
            fg_primary: Color::from_str("#f8f8f2").unwrap(),
            fg_muted: Color::from_str("#75715e").unwrap(),
            accent_blue: Color::from_str("#66d9ef").unwrap(),
            accent_edu: Color::from_str("#a6e22e").unwrap(),
            accent_ai: Color::from_str("#ae81ff").unwrap(),
            hover_bg: Color::from_str("#3e3d32").unwrap(),
            bg_button: Color::from_str("#0e639c").unwrap(),
            fg_button: Color::from_str("#ffffff").unwrap(),
            hover_button: Color::from_str("#1177bb").unwrap(),
        }
    }

    /// One Dark theme palette
    pub fn one_dark() -> Self {
        Self {
            bg_editor: Color::from_str("#282c34").unwrap(),
            bg_sidebar: Color::from_str("#21252b").unwrap(),
            bg_activity: Color::from_str("#1e2227").unwrap(),
            bg_titlebar: Color::from_str("#21252b").unwrap(),
            bg_tab_active: Color::from_str("#282c34").unwrap(),
            bg_tab_inactive: Color::from_str("#21252b").unwrap(),
            bg_panel: Color::from_str("#282c34").unwrap(),
            border: Color::from_str("#181a1f").unwrap(),
            fg_primary: Color::from_str("#abb2bf").unwrap(),
            fg_muted: Color::from_str("#5c6370").unwrap(),
            accent_blue: Color::from_str("#61afef").unwrap(),
            accent_edu: Color::from_str("#98c379").unwrap(),
            accent_ai: Color::from_str("#c678dd").unwrap(),
            hover_bg: Color::from_str("#2c313a").unwrap(),
            bg_button: Color::from_str("#4d78cc").unwrap(),
            fg_button: Color::from_str("#ffffff").unwrap(),
            hover_button: Color::from_str("#5c8ae6").unwrap(),
        }
    }

    /// Solarized Dark theme palette
    pub fn solarized_dark() -> Self {
        Self {
            bg_editor: Color::from_str("#002b36").unwrap(),
            bg_sidebar: Color::from_str("#073642").unwrap(),
            bg_activity: Color::from_str("#00212b").unwrap(),
            bg_titlebar: Color::from_str("#073642").unwrap(),
            bg_tab_active: Color::from_str("#002b36").unwrap(),
            bg_tab_inactive: Color::from_str("#073642").unwrap(),
            bg_panel: Color::from_str("#002b36").unwrap(),
            border: Color::from_str("#073642").unwrap(),
            fg_primary: Color::from_str("#839496").unwrap(),
            fg_muted: Color::from_str("#586e75").unwrap(),
            accent_blue: Color::from_str("#268bd2").unwrap(),
            accent_edu: Color::from_str("#2aa198").unwrap(),
            accent_ai: Color::from_str("#6c71c4").unwrap(),
            hover_bg: Color::from_str("#073642").unwrap(),
            bg_button: Color::from_str("#268bd2").unwrap(),
            fg_button: Color::from_str("#ffffff").unwrap(),
            hover_button: Color::from_str("#2aa198").unwrap(),
        }
    }

    /// Select or construct UI theme matching the syntax theme
    pub fn for_theme(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "monokai" => Self::monokai(),
            "one_dark" | "onedark" => Self::one_dark(),
            "solarized_dark" | "solarized" => Self::solarized_dark(),
            _ => Self::default(),
        }
    }
}
