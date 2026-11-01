//! # Educational Doc Tutor Module
//!
//! Provides pedagogical cheat sheets and reference cards for university students.
//! Cards can be dynamically loaded at runtime from `assets/tutor/cards.json`
//! without requiring application recompilation.

use std::path::Path;
use serde::{Deserialize, Serialize};
use codium_core::Language;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocCard {
    pub title: String,
    pub language: Language,
    pub category: String,
    pub content: String,
    pub example: String,
}

pub struct DocTutor {
    cards: Vec<DocCard>,
}

impl Default for DocTutor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocTutor {
    /// Initializes DocTutor, trying to load cards from `./resources/tutor/cards.json`
    /// or falling back to default cards.
    pub fn new() -> Self {
        let candidates = [
            Path::new("resources/tutor/cards.json"),
            Path::new("tutor/cards.json"),
            Path::new("assets/tutor/cards.json"),
            Path::new("cards.json"),
        ];

        for path in &candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(cards) = serde_json::from_str::<Vec<DocCard>>(&content) {
                        return Self { cards };
                    }
                }
            }
        }

        Self::with_defaults()
    }

    /// Load cards from a specific resources or assets folder
    pub fn with_resources(resources_dir: &Path) -> Self {
        let tutor_json = resources_dir.join("tutor").join("cards.json");
        if tutor_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&tutor_json) {
                if let Ok(cards) = serde_json::from_str::<Vec<DocCard>>(&content) {
                    return Self { cards };
                }
            }
        }
        let direct_json = resources_dir.join("cards.json");
        if direct_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&direct_json) {
                if let Ok(cards) = serde_json::from_str::<Vec<DocCard>>(&content) {
                    return Self { cards };
                }
            }
        }
        Self::new()
    }

    /// Backwards compatibility alias for with_resources
    pub fn with_assets(assets_dir: &Path) -> Self {
        Self::with_resources(assets_dir)
    }

    /// Built-in fallback cards if assets/tutor/cards.json is absent
    pub fn with_defaults() -> Self {
        let cards = vec![
            DocCard {
                title: "Standard I/O (printf / scanf)".to_string(),
                language: Language::C,
                category: "I/O".to_string(),
                content: "Be- és kimenet kezelése C-ben. A printf formázott kiíratást, a scanf beolvasást végez mutatókon keresztül (&).".to_string(),
                example: "#include <stdio.h>\n\nint main() {\n    int szam;\n    printf(\"Adj meg egy számot: \");\n    scanf(\"%d\", &szam);\n    printf(\"A beolvasott szám: %d\\n\", szam);\n    return 0;\n}".to_string(),
            },
            DocCard {
                title: "Dinamikus memóriakezelés (malloc / free)".to_string(),
                language: Language::C,
                category: "Memória".to_string(),
                content: "Dinamikus tömb vagy struktúra allokálása a heap memóriában. Minden malloc() híváshoz kötelező egy free() felszabadítás!".to_string(),
                example: "#include <stdlib.h>\n\nint *tomb = (int*)malloc(n * sizeof(int));\nif (tomb == NULL) {\n    // Memóriahiba kezelése\n}\n// Használat...\nfree(tomb);".to_string(),
            },
            DocCard {
                title: "Ownership és Borrowing".to_string(),
                language: Language::Rust,
                category: "Memória".to_string(),
                content: "Rust memóriabiztonsági modellje: Minden értéknek pontosan egy tulajdonosa van. Kölcsönzés történhet immutábilis (&T) vagy egyidejűleg egyetlen mutábilis (&mut T) hivatkozással.".to_string(),
                example: "fn print_len(s: &String) {\n    println!(\"Hossz: {}\", s.len());\n}\n\nlet mut s = String::from(\"Hello\");\nprint_len(&s); // kölcsönzés\ns.push_str(\", World!\");".to_string(),
            },
            DocCard {
                title: "Result és Option hibakezelés".to_string(),
                language: Language::Rust,
                category: "Hibakezelés".to_string(),
                content: "Rustban nincsenek kivételek (exception). A hiányzó értéket Option<T> (Some/None), a hibalehetőséget Result<T, E> (Ok/Err) fejezi ki.".to_string(),
                example: "match std::fs::read_to_string(\"file.txt\") {\n    Ok(content) => println!(\"Tartalom: {content}\"),\n    Err(err) => eprintln!(\"Hiba: {err}\"),\n}".to_string(),
            },
            DocCard {
                title: "Fájlkezelés 'with' blokkal".to_string(),
                language: Language::Python,
                category: "I/O".to_string(),
                content: "A 'with open(...) as f:' kontextuskezelő automatikusan és garantáltan lezárja a fájlt, még kivétel esetén is.".to_string(),
                example: "with open(\"adatok.txt\", \"r\", encoding=\"utf-8\") as f:\n    sorok = [sor.strip() for sor in f.readlines()]\nprint(f\"{len(sorok)} sor beolvasva.\")".to_string(),
            },
        ];

        Self { cards }
    }

    pub fn for_language(&self, lang: &Language) -> Vec<&DocCard> {
        self.cards.iter().filter(|c| &c.language == lang).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&DocCard> {
        let q = query.to_lowercase();
        self.cards
            .iter()
            .filter(|c| c.title.to_lowercase().contains(&q) || c.content.to_lowercase().contains(&q))
            .collect()
    }
}
