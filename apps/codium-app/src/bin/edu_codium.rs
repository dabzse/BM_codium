use std::path::PathBuf;
use clap::Parser;
use floem::prelude::*;
use floem::window::WindowConfig;
use floem::Application;
use floem::peniko::kurbo::Size;
use codium_ui::{AppState, CodiumEdition, root_app_view};

#[derive(Parser, Debug)]
#[command(name = "edu-codium", author, version, about = "EDU::Codium - University Native Rust IDE for Students (AI Disabled, Doc Tutor Enabled)")]
struct Cli {
    /// Path to file or folder to open
    path: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();

    let state = AppState::new(CodiumEdition::Edu);

    let settings = state.settings.get();
    let win_w = settings.window_width;
    let win_h = settings.window_height;

    if let Some(path) = args.path {
        state.workspace.update(|ws| {
            if path.is_dir() {
                let _ = ws.open_folder(path);
            } else {
                let _ = ws.open_file(path);
            }
        });
    } else if let Some(last_folder) = &settings.last_workspace_folder {
        let path = PathBuf::from(last_folder);
        if path.is_dir() {
            state.workspace.update(|ws| {
                let _ = ws.open_folder(path);
            });
        }
    }

    let mut window_config = WindowConfig::default()
        .size(Size::new(win_w, win_h))
        .title("EDU_codium - University Edition");

    if let Some(icon) = codium_ui::codium_window_icon(true) {
        window_config = window_config.window_icon(icon);
    }

    Application::new()
        .window(
            move |_| root_app_view(state),
            Some(window_config),
        )
        .run();
}
