use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EduPolicy {
    /// Generative AI (LLM auto-writing code) is strictly disabled in EDU mode
    pub allow_ai_generation: bool,
    /// Autocomplete assistant daemon can be enabled if the teacher allows it
    pub allow_autocomplete_assistant: bool,
    /// Access to offline manuals and documentation
    pub allow_manuals_access: bool,
    /// Built-in language documentation and pedagogic tutor panel
    pub doc_tutor_enabled: bool,
    /// Offline exam mode blocks all network access
    pub exam_mode: bool,
    /// Name of the university / course
    pub institution_name: String,
    /// Backwards compatibility alias
    pub allow_kite_completion: bool,
}

impl Default for EduPolicy {
    fn default() -> Self {
        Self {
            allow_ai_generation: false,
            allow_autocomplete_assistant: true,
            allow_manuals_access: true,
            doc_tutor_enabled: true,
            exam_mode: false,
            institution_name: "University / Educational Mode".to_string(),
            allow_kite_completion: true,
        }
    }
}

impl EduPolicy {
    pub fn from_settings(settings: &codium_core::CodiumSettings) -> Self {
        Self {
            allow_ai_generation: settings.allow_ai_generation,
            allow_autocomplete_assistant: settings.allow_autocomplete_assistant,
            allow_manuals_access: settings.allow_manuals_access,
            doc_tutor_enabled: settings.doc_tutor_enabled,
            exam_mode: settings.exam_mode,
            institution_name: settings.institution_name.clone(),
            allow_kite_completion: settings.allow_autocomplete_assistant,
        }
    }

    pub fn load_from_workspace(workspace_dir: &Path) -> Self {
        let policy_path = workspace_dir.join(".codium-edu.toml");
        if policy_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&policy_path) {
                let mut policy = Self::default();
                for line in content.lines() {
                    let line = line.trim();
                    if (line.starts_with("allow_autocomplete_assistant") || line.starts_with("allow_kite_completion")) && line.contains("true") {
                        policy.allow_autocomplete_assistant = true;
                        policy.allow_kite_completion = true;
                    }
                    if line.starts_with("allow_manuals_access") && line.contains("false") {
                        policy.allow_manuals_access = false;
                    }
                    if line.starts_with("allow_ai_generation") && line.contains("true") {
                        policy.allow_ai_generation = false;
                    }
                    if line.starts_with("institution_name") {
                        if let Some(val) = line.split('=').nth(1) {
                            policy.institution_name = val.trim().trim_matches('"').to_string();
                        }
                    }
                }
                return policy;
            }
        }
        Self::default()
    }
}
