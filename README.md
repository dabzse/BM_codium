# BM::Codium [AI | EDU]

[![Language](assets/rust-lang.svg)](https://www.rust-lang.org/)
[![License: AGPL-3.0-or-later](assets/license_AGPLv3.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Dual-License: AGPLv3+ / Commercial](assets/license_AGPLv3+_commercial.svg)](LICENSE)

---

## Hungarian Documentation

A magyar nyelvű leírásért lásd a [README_hu.md](README_hu.md) fájlt!

---

## Beginning and the Basic Idea

Yesterday my **BSc final exam** was successful, so today I've decided to build something truly useful for universities and students...  
This one is called: **EDU::Codium**. It is an educational text editor not just for learning programming languages, but also using it for school work, university tasks, etc. At last, but not least: for doing the exam tasks.  
**AI-free**: code generation is strictly forbidden, and code completion is disabled by default so that students truly learn to code. Furthermore, it contains features for teachers to manage assignments and courses.  
This idea was inspired by programming environments such as **Zed**, **Lapce**, **Lite-XL** and mostly **VSCodium** (as the name is also similar).  

Alongside it lives **AI::Codium**, the counterpart edition tailored for advanced software engineering with integrated offline local LLM capabilities.

---

## Two Dedicated Editions

| Feature             | 🎓 EDU::Codium (`edu-codium`)                                    | ✦ AI::Codium (`ai-codium`)                            |
| :------------------ | :--------------------------------------------------------------- | :---------------------------------------------------- |
| **Target Audience** | Universities, High Schools, Students, Teachers, Exams            | Advanced Software Engineers, Researchers              |
| **Generative AI**   | **Strictly Forbidden** (no copy-pasting code, real learning)     | **Enabled** (Ghost-text completions, LLM chat)        |
| **Code Completion** | **Teacher-Configurable** (disabled by default, Kite mode)        | Full AI & LSP autocompletion                          |
| **Doc Tutor**       | **Doc Tutor & Error Explainer** (provides hints, not final code) | AI explanations & refactoring suggestions             |
| **Teacher Policy**  | Via `.codium-edu.toml` file (exam mode, lock downs)              | Standard user settings                                |
| **Binary Size**     | **standalone native binary**                                     | **standalone native binary**                          |
| **Startup Time**    | **planned to be near-instant** (Rust, small and fast)            | **planned to be near-instant** (Rust, small and fast) |

---

## Layout and User Interface (Ergonomics)

BM::Codium combines the best of modern IDEs into a unified, zero-bloat, and lightning-fast software:

1. **Familiarity from VS Code**:
   - **Activity Bar (Left icon strip)**: Vertical vector buttons for Explorer, Search, Doc Tutor, Git, Database, etc., and Settings with an active indicator accent line.
   - **Sidebar**: Collapsible file explorer with folder trees and language-specific icons.
   - **Tabbed Editor**: Multi-tab document management with active/inactive contrast and dirty state dot (`●`).
   - **Bottom Dock**: Multi-functional bottom panel with Terminal, Problems/Diagnostics view, and Doc Tutor hints.
   - **Status Bar**: Clean bottom bar with cursor coordinates (`Ln, Col`), Encoding (`UTF-8`), Spaces, and Language detection.

2. **JetBrains Productivity Features**:
   - **Search Everywhere (`Shift + Shift` / `Ctrl + P`)**: Central search pill in the titlebar searching files, symbols, and commands all at once.
   - **Run Button (▶)**: Direct compilation and execution from the titlebar or right next to code entry points (`main()`).
   - **Quick Documentation (`F1` / Doc Tutor)**: Instant access to standard library cheat sheets and syntax cards.

3. **100% Native Rust**:
   - **Zero Electron / Zero Web Bloat**: Direct native machine code with GPU hardware acceleration.
   - **???**: something, which handles files of very big size.
   - **Vector SVG Icons**: Embedded vector Codicons instead of text emojis. (?)

4. **Bilingual Localization (English / Hungarian)**:
   - **Default Language**: English (`EN`), complying with university and international standards.
   - **Parallel Hungarian (`HU`) Language**: Instant runtime switching via the `🌐 EN / HU` toggle in the titlebar and statusbar. (?)

---

## Project Architecture

The project follows a clean, modular Cargo Workspace layout:

```text
native-codium/
├── Cargo.toml                # Workspace definition & optimized profiles
├── .cargo/config.toml        # Offline vendored crate resolution (vendor-offline)
├── crates/
│   ├── codium-core/          # Text editor engine, Undo/Redo, File Trees, i18n locale engine
│   ├── codium-edu/           # University policies (.codium-edu.toml), DocTutor, error explainer
│   ├── codium-ai/            # Local LLM traits, inline ghost-text completion provider
│   └── codium-ui/            # GUI layout: Titlebar, Activity Bar, Sidebar, Tabs, Bottom Panel
└── apps/
    └── codium-app/           # Binary runners:
        ├── src/bin/ai_codium.rs   -> ai-codium executable
        └── src/bin/edu_codium.rs  -> edu-codium executable
```

---

## Building and Running

### Prerequisites

- Rust compiler
- Standard Linux graphics libraries (Wayland / X11)

### Build (Offline)

```bash
cd native-codium
cargo build --offline --bin edu-codium --bin ai-codium
```

The resulting binaries will be placed in `native-codium/target/debug/` (or `target/release/` for production):

- `target/debug/edu-codium`
- `target/debug/ai-codium`

### Launch

```bash
# Launch University Educational Edition
./target/debug/edu-codium

# Open a specific file or folder
./target/debug/edu-codium /path/to/project

# Launch AI Edition
./target/debug/ai-codium
```

---

## University Policy Configuration (`.codium-edu.toml`)

Teachers can place a `.codium-edu.toml` file into their assignment repositories:

```toml
# Institution / Course name
institution_name = "Computer Science Basics - BSc"

# Strictly disable generative AI code-writing
allow_ai_generation = false

# Teacher-controlled token autocompletion
allow_kite_completion = false

# Enable pedagogic documentation and hint cards
doc_tutor_enabled = true

# Offline exam mode (blocks network calls)
exam_mode = true
```
