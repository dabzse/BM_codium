//! # Codium Syntax Styling Adapter
//!
//! Connects `codium_core::syntax` tokenization and themes to Floem's `Styling` trait
//! for high-performance, real-time syntax coloring inside `TextEditor`.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use codium_core::syntax::{
    compute_line_end_state, tokenize_line, MultilineState, SyntaxLanguage, SyntaxTheme, TokenType,
};
use floem::peniko::Color;
use floem::reactive::{RwSignal, SignalGet};
use floem::text::{Attrs, AttrsList, FamilyOwned, Stretch, Style, Weight};
use floem::views::editor::core::buffer::rope_text::RopeText;
use floem::views::editor::id::EditorId;
use floem::views::editor::layout::TextLayoutLine;
use floem::views::editor::text::{Document, Styling};
use floem::views::editor::EditorStyle;

pub struct CodiumStyling {
    pub syntax_theme: RwSignal<SyntaxTheme>,
    pub language: SyntaxLanguage,
    pub style: Rc<dyn Styling>,
    pub doc: Option<Rc<dyn Document>>,
    pub states: RefCell<Vec<MultilineState>>,
}

impl CodiumStyling {
    pub fn new(
        syntax_theme: RwSignal<SyntaxTheme>,
        language: SyntaxLanguage,
        style: Rc<dyn Styling>,
    ) -> Self {
        Self {
            syntax_theme,
            language,
            style,
            doc: None,
            states: RefCell::new(Vec::new()),
        }
    }

    pub fn set_doc(&mut self, doc: Rc<dyn Document>) {
        self.doc = Some(doc);
    }

    fn get_line_start_state(&self, line: usize) -> MultilineState {
        if line == 0 {
            return MultilineState::Normal;
        }
        let mut cache = self.states.borrow_mut();
        if line < cache.len() {
            return cache[line];
        }

        let mut cur_state = if cache.is_empty() {
            MultilineState::Normal
        } else {
            *cache.last().unwrap()
        };

        if let Some(doc) = &self.doc {
            let rope = doc.rope_text();
            let total = rope.num_lines();
            let mut idx = cache.len();
            while idx < line && idx < total {
                let content = rope.line_content(idx);
                cur_state = compute_line_end_state(self.language, &content, cur_state);
                cache.push(cur_state);
                idx += 1;
            }
        }
        cur_state
    }
}

impl Styling for CodiumStyling {
    fn id(&self) -> u64 {
        self.style.id()
    }

    fn font_size(&self, edid: EditorId, line: usize) -> usize {
        self.style.font_size(edid, line)
    }

    fn line_height(&self, edid: EditorId, line: usize) -> f32 {
        self.style.line_height(edid, line)
    }

    fn font_family(&self, edid: EditorId, line: usize) -> Cow<'_, [FamilyOwned]> {
        self.style.font_family(edid, line)
    }

    fn weight(&self, edid: EditorId, line: usize) -> Weight {
        self.style.weight(edid, line)
    }

    fn italic_style(&self, edid: EditorId, line: usize) -> Style {
        self.style.italic_style(edid, line)
    }

    fn stretch(&self, edid: EditorId, line: usize) -> Stretch {
        self.style.stretch(edid, line)
    }

    fn indent_line(&self, edid: EditorId, line: usize, line_content: &str) -> usize {
        self.style.indent_line(edid, line, line_content)
    }

    fn tab_width(&self, edid: EditorId, line: usize) -> usize {
        self.style.tab_width(edid, line)
    }

    fn atomic_soft_tabs(&self, edid: EditorId, line: usize) -> bool {
        self.style.atomic_soft_tabs(edid, line)
    }

    fn apply_attr_styles(
        &self,
        _edid: EditorId,
        _style: &EditorStyle,
        line: usize,
        default: Attrs,
        attrs: &mut AttrsList,
    ) {
        attrs.clear_spans();
        if let Some(doc) = &self.doc {
            let rope = doc.rope_text();
            if line >= rope.num_lines() {
                return;
            }
            let line_content = rope.line_content(line);
            let start_state = self.get_line_start_state(line);
            let tokens = tokenize_line(self.language, &line_content, start_state);
            let theme = self.syntax_theme.get_untracked();

            for span in tokens.tokens {
                if let Some(color_hex) = theme.color_for_token(span.token_type) {
                    if let Ok(color) = Color::from_str(color_hex) {
                        let mut attr = default.clone().color(color);
                        if span.token_type == TokenType::Comment || span.token_type == TokenType::DocComment {
                            attr = attr.style(Style::Italic);
                        } else if span.token_type == TokenType::Keyword || span.token_type == TokenType::Macro {
                            attr = attr.weight(Weight::BOLD);
                        }
                        let len = line_content.len();
                        let start = span.range.start.min(len);
                        let end = span.range.end.min(len);
                        if start < end {
                            attrs.add_span(start..end, attr);
                        }
                    }
                }
            }
        }
    }

    fn apply_layout_styles(
        &self,
        edid: EditorId,
        style: &EditorStyle,
        line: usize,
        layout_line: &mut TextLayoutLine,
    ) {
        self.style.apply_layout_styles(edid, style, line, layout_line);
    }

    fn paint_caret(&self, edid: EditorId, line: usize) -> bool {
        self.style.paint_caret(edid, line)
    }
}
