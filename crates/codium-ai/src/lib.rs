use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InlineSuggestion {
    pub text: String,
    pub replace_range: Option<(usize, usize)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub trait AiEngine: Send + Sync {
    fn request_inline_completion(&self, prefix: &str, suffix: &str) -> Option<InlineSuggestion>;
    fn send_chat_message(&self, history: &[ChatMessage], message: &str) -> String;
}

pub struct LocalAiEngine {
    pub endpoint_url: String,
    pub model_name: String,
    pub is_connected: bool,
}

impl Default for LocalAiEngine {
    fn default() -> Self {
        Self {
            endpoint_url: "http://127.0.0.1:11434".to_string(),
            model_name: "codium-local-coder".to_string(),
            is_connected: true,
        }
    }
}

impl AiEngine for LocalAiEngine {
    fn request_inline_completion(&self, prefix: &str, _suffix: &str) -> Option<InlineSuggestion> {
        let trimmed = prefix.trim_end();
        if trimmed.ends_with("fn ") {
            Some(InlineSuggestion {
                text: "main() {\n    println!(\"Hello Native Codium!\");\n}".to_string(),
                replace_range: None,
            })
        } else if trimmed.ends_with("for ") {
            Some(InlineSuggestion {
                text: "i in 0..10 {\n    // do work\n}".to_string(),
                replace_range: None,
            })
        } else {
            None
        }
    }

    fn send_chat_message(&self, _history: &[ChatMessage], message: &str) -> String {
        format!("✦ [AI Codium Local Model]: Válasz a kérdésedre: '{}'. A kód és a modell offline fut.", message)
    }
}
