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
        }
    }
}
