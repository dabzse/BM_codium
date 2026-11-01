pub mod icons;
pub mod keymap;
pub mod state;
pub mod theme;
pub mod views;

pub use icons::{codium_window_icon, render_svg, render_svg_colored};
pub use keymap::{execute_command, match_keymap, matches_chord};
pub use state::{ActiveMenu, AppState, BottomTab, CodiumEdition, SidebarTab};
pub use theme::CodiumTheme;
pub use views::app_view::root_app_view;
