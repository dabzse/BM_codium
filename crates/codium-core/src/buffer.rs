use std::path::{Path, PathBuf};
use lapce_xi_rope::{Rope, Interval};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    // 17 Primary Programming Languages (matching canonical README order):
    Python,
    Php,
    Rust,
    C,
    Cpp,
    CSharp,
    Java,
    Kotlin,
    Erlang,
    Scala,
    Elixir,
    Go,
    TypeScript,
    JavaScript,
    Ruby,
    Zig,
    Swift,
    ObjectiveC,

    // Web, Markup, Shell & Data:
    Html,
    Css,
    Markdown,
    Shell,
    Toml,
    Json,
    Sql,
    Plaintext,
}

impl Language {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
            "py" | "pyw" => Language::Python,
            "php" | "phtml" => Language::Php,
            "rs" => Language::Rust,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" => Language::Cpp,
            "cs" => Language::CSharp,
            "java" => Language::Java,
            "kt" | "kts" => Language::Kotlin,
            "erl" | "hrl" => Language::Erlang,
            "scala" | "sc" => Language::Scala,
            "ex" | "exs" => Language::Elixir,
            "go" => Language::Go,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "rb" | "rake" => Language::Ruby,
            "zig" => Language::Zig,
            "swift" => Language::Swift,
            "m" | "mm" => Language::ObjectiveC,
            "html" | "htm" | "xml" | "svg" => Language::Html,
            "css" | "scss" | "sass" | "less" => Language::Css,
            "md" | "rst" | "asc" | "tex" => Language::Markdown,
            "sh" | "bash" | "bat" | "cmd" | "ps1" => Language::Shell,
            "toml" => Language::Toml,
            "json" => Language::Json,
            "sql" => Language::Sql,
            _ => Language::Plaintext,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::Python => "Python",
            Language::Php => "PHP",
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Erlang => "Erlang",
            Language::Scala => "Scala",
            Language::Elixir => "Elixir",
            Language::Go => "Go",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Ruby => "Ruby",
            Language::Zig => "Zig",
            Language::Swift => "Swift",
            Language::ObjectiveC => "Objective-C",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Markdown => "Markdown",
            Language::Shell => "Shell",
            Language::Toml => "TOML",
            Language::Json => "JSON",
            Language::Sql => "SQL",
            Language::Plaintext => "Plain Text",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextBuffer {
    rope: Rope,
    path: Option<PathBuf>,
    language: Language,
    cursor: usize,
    #[allow(dead_code)]
    selection_start: Option<usize>,
    is_dirty: bool,
    undo_stack: Vec<Rope>,
    redo_stack: Vec<Rope>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::from(""),
            path: None,
            language: Language::Plaintext,
            cursor: 0,
            selection_start: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_str(text: &str, path: Option<PathBuf>) -> Self {
        let language = path.as_deref().map(Language::from_path).unwrap_or(Language::Plaintext);
        Self {
            rope: Rope::from(text),
            path,
            language,
            cursor: 0,
            selection_start: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn len(&self) -> usize {
        self.rope.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.language = Language::from_path(&path);
        self.path = Some(path);
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, offset: usize) {
        self.cursor = offset.min(self.len());
    }

    pub fn line_of_offset(&self, offset: usize) -> usize {
        self.rope.line_of_offset(offset.min(self.len()))
    }

    pub fn offset_of_line(&self, line: usize) -> usize {
        self.rope.offset_of_line(line)
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_of_offset(offset);
        let line_offset = self.offset_of_line(line);
        let col = offset.saturating_sub(line_offset);
        (line + 1, col + 1)
    }

    pub fn line_count(&self) -> usize {
        if self.rope.is_empty() {
            1
        } else {
            self.rope.line_of_offset(self.rope.len()) + 1
        }
    }

    pub fn set_rope(&mut self, rope: Rope) {
        self.rope = rope;
        self.cursor = self.cursor.min(self.rope.len());
        self.is_dirty = true;
    }

    pub fn set_text(&mut self, text: &str) {
        self.undo_stack.push(self.rope.clone());
        self.redo_stack.clear();
        self.rope = Rope::from(text);
        self.cursor = self.cursor.min(self.rope.len());
        self.is_dirty = true;
    }

    pub fn insert_text(&mut self, text: &str) {
        self.undo_stack.push(self.rope.clone());
        self.redo_stack.clear();

        let offset = self.cursor;
        let mut builder = lapce_xi_rope::DeltaBuilder::new(self.rope.len());
        builder.replace(Interval::new(offset, offset), Rope::from(text));
        let delta = builder.build();
        self.rope = delta.apply(&self.rope);
        self.cursor += text.len();
        self.is_dirty = true;
    }

    pub fn delete_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.undo_stack.push(self.rope.clone());
        self.redo_stack.clear();

        let start = self.cursor - 1;
        let mut builder = lapce_xi_rope::DeltaBuilder::new(self.rope.len());
        builder.delete(Interval::new(start, self.cursor));
        let delta = builder.build();
        self.rope = delta.apply(&self.rope);
        self.cursor = start;
        self.is_dirty = true;
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.rope.clone());
            self.rope = prev;
            self.cursor = self.cursor.min(self.rope.len());
            self.is_dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.rope.clone());
            self.rope = next;
            self.cursor = self.cursor.min(self.rope.len());
            self.is_dirty = true;
        }
    }
}
