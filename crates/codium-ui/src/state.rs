//! # Application State Management
//!
//! Holds reactive signals for the workspace, active tabs, educational policies,
//! UI panels visibility, locale (GNU gettext .po backed), theme settings,
//! and settings.json configurations.

use codium_core::{CodiumSettings, Keymap, Locale, Workspace, init_i18n};
use codium_edu::EduPolicy;
use floem::reactive::RwSignal;
use crate::theme::CodiumTheme;

/// The target edition of the Codium IDE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodiumEdition {
    /// University edition: AI generative code writing disabled, focuses on learning
    Edu,
    /// Professional AI edition: integrates local LLM, ghost-text completions
    Ai,
}

impl CodiumEdition {
    /// Window title prefix
    pub fn title(&self) -> &'static str {
        match self {
            CodiumEdition::Edu => "EDU_codium",
            CodiumEdition::Ai => "AI_codium",
        }
    }

    /// Edition subtitle localized dynamically
    pub fn subtitle(&self, locale: Locale) -> String {
        match self {
            CodiumEdition::Edu => locale.edu_subtitle(),
            CodiumEdition::Ai => locale.ai_subtitle(),
        }
    }
}

/// Tabs available in the primary sidebar
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarTab {
    Explorer,
    Search,
    DocTutor,
    AiAssistant,
    Settings,
}

/// Tabs available in the bottom docking panel
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BottomTab {
    Terminal,
    Problems,
    DocTutorOutput,
}

/// Top-level dropdown menus in desktop IDE menubar
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveMenu {
    File,
    Edit,
    View,
    Tools,
    Help,
}

/// Reactive state container passed across all UI views
#[derive(Clone, Copy)]
pub struct AppState {
    /// Currently opened dropdown menu in the menubar (if any)
    pub active_menu: RwSignal<Option<ActiveMenu>>,
    /// Active overlay ViewId for open menubar dropdown (if any)
    pub active_menu_overlay: RwSignal<Option<floem::ViewId>>,
    /// Selected IDE edition (EDU vs AI)
    pub edition: CodiumEdition,
    /// Runtime configuration loaded from settings.json
    pub settings: RwSignal<CodiumSettings>,
    /// Active UI language (Locale::En or Locale::Hu)
    pub locale: RwSignal<Locale>,
    /// Theme colors (dark mode palette)
    pub theme: RwSignal<CodiumTheme>,
    /// Active workspace and file tree
    pub workspace: RwSignal<Workspace>,
    /// Selected primary sidebar view
    pub active_sidebar_tab: RwSignal<SidebarTab>,
    /// Sidebar visibility toggle
    pub show_sidebar: RwSignal<bool>,
    /// Bottom panel visibility toggle
    pub show_bottom_panel: RwSignal<bool>,
    /// Active tab in bottom dock
    pub active_bottom_tab: RwSignal<BottomTab>,
    /// University and teacher restrictions/rules
    pub edu_policy: RwSignal<EduPolicy>,
    /// Bottom status bar message
    pub status_message: RwSignal<String>,
    /// Quick search input buffer
    pub search_query: RwSignal<String>,
    /// Terminal output buffer lines
    pub terminal_logs: RwSignal<Vec<String>>,
    /// Secondary right panel toggle (Manuals & Doc Tutor)
    pub show_right_panel: RwSignal<bool>,
    /// Filter for programming language in manuals panel
    pub manuals_lang_query: RwSignal<String>,
    /// Filter for framework / library in manuals panel
    pub manuals_module_query: RwSignal<String>,
    /// Chat input buffer for offline manuals tutor
    pub manuals_chat_query: RwSignal<String>,
    /// Chat message history for offline manuals tutor
    pub manuals_chat_messages: RwSignal<Vec<(String, String)>>,
    /// Left sidebar width in pixels
    pub sidebar_width: RwSignal<f64>,
    /// Bottom panel height in pixels
    pub bottom_panel_height: RwSignal<f64>,
    /// Right secondary panel width in pixels
    pub right_panel_width: RwSignal<f64>,
    /// Toggle JetBrains Search Everywhere pill in titlebar
    pub show_titlebar_search: RwSignal<bool>,
    /// Replace input buffer for search & replace
    pub replace_query: RwSignal<String>,
    /// Whether replace row is unfolded in search sidebar (VS Code style)
    pub show_replace_row: RwSignal<bool>,
    /// Search results across workspace files
    pub search_results: RwSignal<Vec<FileSearchMatch>>,
    /// Currently active/selected search match index
    pub active_search_match_idx: RwSignal<Option<usize>>,
    /// Word wrap toggle for editor (Alt+Z)
    pub word_wrap: RwSignal<bool>,
    /// VSCodium keybinding configuration
    pub keymap: RwSignal<Keymap>,
}

/// Represents a match in a file for workspace Search & Replace
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSearchMatch {
    pub file_path: std::path::PathBuf,
    pub line_number: usize,
    pub line_content: String,
}

impl AppState {
    /// Initialize application state with appropriate defaults
    pub fn new(edition: CodiumEdition) -> Self {
        let mut ws = Workspace::new();
        // Automatically open the current directory if available
        if let Ok(cwd) = std::env::current_dir() {
            let _ = ws.open_folder(cwd);
        }

        // 1. Load settings.json dynamically from current/exe directory
        let settings = CodiumSettings::load();

        // 2. Initialize i18n engine with locales_dir from settings.json
        init_i18n(&settings.resolve_locales_dir());

        // 3. Determine initial locale from settings
        let initial_locale = Locale::from_code(&settings.locale);

        // 4. Configure educational policy according to edition and settings.json
        let mut policy = EduPolicy::from_settings(&settings);
        match edition {
            CodiumEdition::Edu => {
                policy.allow_ai_generation = false;
            }
            CodiumEdition::Ai => {
                policy.allow_ai_generation = true;
                policy.institution_name = "AI Professional Workspace".to_string();
            }
        };

        let initial_logs = vec![
            format!("=== {} {} ===", edition.title(), initial_locale.tr("Native Terminal Ready")),
            initial_locale.tr("Type commands below or click ▶ Run to execute code."),
        ];

        let initial_manuals_messages = vec![
            ("Tutor".to_string(), initial_locale.tr("Ask about downloaded manuals or select a programming language and module!")),
        ];

        Self {
            active_menu: RwSignal::new(None),
            active_menu_overlay: RwSignal::new(None),
            edition,
            sidebar_width: RwSignal::new(settings.sidebar_width),
            bottom_panel_height: RwSignal::new(settings.bottom_panel_height),
            right_panel_width: RwSignal::new(settings.right_panel_width),
            show_titlebar_search: RwSignal::new(settings.show_titlebar_search),
            word_wrap: RwSignal::new(settings.word_wrap),
            replace_query: RwSignal::new(String::new()),
            show_replace_row: RwSignal::new(false),
            search_results: RwSignal::new(Vec::new()),
            active_search_match_idx: RwSignal::new(None),
            settings: RwSignal::new(settings),
            locale: RwSignal::new(initial_locale),
            theme: RwSignal::new(CodiumTheme::default()),
            workspace: RwSignal::new(ws),
            active_sidebar_tab: RwSignal::new(SidebarTab::Explorer),
            show_sidebar: RwSignal::new(true),
            show_bottom_panel: RwSignal::new(true),
            active_bottom_tab: RwSignal::new(BottomTab::Terminal),
            edu_policy: RwSignal::new(policy),
            status_message: RwSignal::new("Ready".to_string()),
            search_query: RwSignal::new(String::new()),
            terminal_logs: RwSignal::new(initial_logs),
            show_right_panel: RwSignal::new(true),
            manuals_lang_query: RwSignal::new(String::new()),
            manuals_module_query: RwSignal::new(String::new()),
            manuals_chat_query: RwSignal::new(String::new()),
            manuals_chat_messages: RwSignal::new(initial_manuals_messages),
            keymap: RwSignal::new(Keymap::load()),
        }
    }
}