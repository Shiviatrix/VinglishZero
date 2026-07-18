//! TypeScript official-frontend extension point.
//!
//! The TypeScript compiler API integration is intentionally not bundled yet.

use std::path::Path;

use vz_adapters::{
    AdapterCapabilities, AdapterManifest, SemanticAdapter, SourceAdapter, SourceAdapterError,
};
use vz_common::LanguageTag;
use vz_semantic_ir::SemanticGraph;

#[derive(Debug, Default)]
pub struct TypeScriptAdapter;

impl SemanticAdapter for TypeScriptAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new("typescript-compiler-frontend", LanguageTag::TypeScript, "1")
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(false, false, false)
    }
}

impl SourceAdapter for TypeScriptAdapter {
    fn extensions(&self) -> &[&str] {
        &["ts", "tsx"]
    }

    fn language(&self) -> &'static str {
        "TypeScript"
    }

    fn semantic_graph(&self, _source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        Err(SourceAdapterError::FrontendUnavailable {
            language: self.language(),
            detail: "TypeScript compiler API integration is not enabled in this build".to_owned(),
        })
    }
}
