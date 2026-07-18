//! JavaScript official-frontend extension point.
//!
//! The TypeScript compiler API integration is intentionally not bundled yet.

use std::path::Path;

use vz_adapters::{
    AdapterCapabilities, AdapterManifest, SemanticAdapter, SourceAdapter, SourceAdapterError,
};
use vz_common::LanguageTag;
use vz_semantic_ir::SemanticGraph;

#[derive(Debug, Default)]
pub struct JavaScriptAdapter;

impl SemanticAdapter for JavaScriptAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new(
            "javascript-typescript-frontend",
            LanguageTag::JavaScript,
            "1",
        )
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(false, false, false)
    }
}

impl SourceAdapter for JavaScriptAdapter {
    fn extensions(&self) -> &[&str] {
        &["js", "mjs", "cjs"]
    }

    fn language(&self) -> &'static str {
        "JavaScript"
    }

    fn semantic_graph(&self, _source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        Err(SourceAdapterError::FrontendUnavailable {
            language: self.language(),
            detail: "TypeScript compiler API integration is not enabled in this build".to_owned(),
        })
    }
}
