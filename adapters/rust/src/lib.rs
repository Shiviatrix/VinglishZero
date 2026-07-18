//! Rust official-frontend extension point.
//!
//! A stable Rust frontend integration is intentionally not bundled yet. This
//! adapter registers `.rs` deterministically and reports that absence instead
//! of fabricating a parser.

use std::path::Path;

use vz_adapters::{
    AdapterCapabilities, AdapterManifest, SemanticAdapter, SourceAdapter, SourceAdapterError,
};
use vz_common::LanguageTag;
use vz_semantic_ir::SemanticGraph;

#[derive(Debug, Default)]
pub struct RustAdapter;

impl SemanticAdapter for RustAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new("rust-official-frontend", LanguageTag::Rust, "1")
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(false, false, false)
    }
}

impl SourceAdapter for RustAdapter {
    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn language(&self) -> &'static str {
        "Rust"
    }

    fn semantic_graph(&self, _source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        Err(SourceAdapterError::FrontendUnavailable {
            language: self.language(),
            detail:
                "rust-analyzer or a stable Rust frontend integration is not enabled in this build"
                    .to_owned(),
        })
    }
}
