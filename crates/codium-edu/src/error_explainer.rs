//! # Educational Compiler Diagnostic & Error Explainer
//!
//! Maps raw compiler stderr (Rust, C, Python, etc.) into pedagogical hints.
//! All explanations and hints are resolved via GNU gettext `.po` catalogs,
//! allowing educators and universities to refine or translate messages
//! without recompiling the application.

use codium_core::Locale;

#[derive(Clone, Debug)]
pub struct DiagnosticExplanation {
    pub error_code: Option<String>,
    pub explanation_key: String,
    pub hint_key: String,
    pub explanation_hu: String,
    pub hint_hu: String,
}

impl DiagnosticExplanation {
    pub fn new(code: Option<String>, explanation_key: impl Into<String>, hint_key: impl Into<String>) -> Self {
        let exp_k = explanation_key.into();
        let hint_k = hint_key.into();
        let exp_hu = Locale::Hu.tr(&exp_k);
        let h_hu = Locale::Hu.tr(&hint_k);
        Self {
            error_code: code,
            explanation_key: exp_k,
            hint_key: hint_k,
            explanation_hu: exp_hu,
            hint_hu: h_hu,
        }
    }

    /// Localized explanation text according to active UI locale
    pub fn explanation(&self, locale: Locale) -> String {
        locale.tr(&self.explanation_key)
    }

    /// Localized hint according to active UI locale
    pub fn hint(&self, locale: Locale) -> String {
        locale.tr(&self.hint_key)
    }
}

pub struct ErrorExplainer;

impl ErrorExplainer {
    pub fn explain(error_output: &str) -> Option<DiagnosticExplanation> {
        let err = error_output.to_lowercase();

        // Rust Borrow Checker
        if err.contains("cannot borrow") && err.contains("as mutable") {
            return Some(DiagnosticExplanation::new(
                Some("E0502 / E0596".to_string()),
                "diag_borrow_checker_explanation",
                "diag_borrow_checker_hint",
            ));
        }

        // C Segmentation Fault
        if err.contains("segmentation fault") || err.contains("sigsegv") {
            return Some(DiagnosticExplanation::new(
                Some("SIGSEGV (139)".to_string()),
                "diag_segfault_explanation",
                "diag_segfault_hint",
            ));
        }

        // C / C++ Undefined reference
        if err.contains("undefined reference to") {
            return Some(DiagnosticExplanation::new(
                Some("Linker Error".to_string()),
                "diag_undefined_ref_explanation",
                "diag_undefined_ref_hint",
            ));
        }

        // Python IndexError / KeyError
        if err.contains("indexerror: list index out of range") {
            return Some(DiagnosticExplanation::new(
                Some("IndexError".to_string()),
                "diag_index_error_explanation",
                "diag_index_error_hint",
            ));
        }

        None
    }
}
