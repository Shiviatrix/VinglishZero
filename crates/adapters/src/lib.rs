//! Adapter contracts for Vinglish Zero.
//!
//! This crate defines the common shape of a language adapter without embedding
//! any language-specific parsing or lowering logic.

use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

use vz_common::LanguageTag;
use vz_semantic_ir::SemanticGraph;

/// Describes adapter capabilities and provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterManifest {
    pub name: String,
    pub language: LanguageTag,
    pub version: String,
    pub notes: Vec<String>,
}

impl AdapterManifest {
    pub fn new(name: impl Into<String>, language: LanguageTag, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            language,
            version: version.into(),
            notes: Vec::new(),
        }
    }
}

/// Declares what an adapter can provide to the semantic engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub parses_source: bool,
    pub consumes_hir: bool,
    pub emits_semantic_ir: bool,
}

impl AdapterCapabilities {
    pub const fn new(parses_source: bool, consumes_hir: bool, emits_semantic_ir: bool) -> Self {
        Self {
            parses_source,
            consumes_hir,
            emits_semantic_ir,
        }
    }
}

/// The shared contract every adapter is expected to satisfy.
pub trait SemanticAdapter {
    fn manifest(&self) -> AdapterManifest;
    fn capabilities(&self) -> AdapterCapabilities;
}

/// A frontend that accepts a source file and produces language-agnostic IR.
///
/// Implementations own all language-specific frontend interaction. The CLI and
/// every semantic engine consumer operate only on the resulting graph.
pub trait SourceAdapter: SemanticAdapter {
    /// File extensions accepted by this frontend, without a leading dot.
    fn extensions(&self) -> &[&str];

    /// Human-readable language name for deterministic diagnostics.
    fn language(&self) -> &'static str;

    /// Produces a graph from source or an adapter-owned interchange input.
    fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, SourceAdapterError>;
}

/// Errors reported at the frontend boundary before Semantic IR reaches engines.
#[derive(Debug)]
pub enum SourceAdapterError {
    ReadSource {
        path: PathBuf,
        source: std::io::Error,
    },
    CompilerNotFound {
        language: &'static str,
        executable: String,
    },
    CompilerFailed {
        language: &'static str,
        exit_code: Option<i32>,
        stderr: String,
    },
    InvalidSource {
        language: &'static str,
        message: String,
    },
    FrontendUnavailable {
        language: &'static str,
        detail: String,
    },
    AdapterFailure {
        language: &'static str,
        message: String,
    },
}

impl fmt::Display for SourceAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadSource { path, source } => {
                write!(formatter, "cannot read '{}': {source}", path.display())
            }
            Self::CompilerNotFound {
                language,
                executable,
            } => write!(
                formatter,
                "{language} frontend executable '{executable}' was not found"
            ),
            Self::CompilerFailed {
                language,
                exit_code,
                stderr,
            } => {
                write!(
                    formatter,
                    "{language} frontend failed{}",
                    exit_code
                        .map(|code| format!(" with exit code {code}"))
                        .unwrap_or_default()
                )?;
                if !stderr.is_empty() {
                    write!(formatter, ": {stderr}")?;
                }
                Ok(())
            }
            Self::InvalidSource { language, message } => {
                write!(formatter, "invalid {language} source: {message}")
            }
            Self::FrontendUnavailable { language, detail } => {
                write!(formatter, "{language} frontend is unavailable: {detail}")
            }
            Self::AdapterFailure { language, message } => {
                write!(formatter, "{language} adapter failed: {message}")
            }
        }
    }
}

impl std::error::Error for SourceAdapterError {}

/// Extension-indexed set of independently registered source adapters.
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn SourceAdapter>>,
    extensions: BTreeMap<String, usize>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers every extension declared by `adapter`.
    ///
    /// A duplicate is rejected so adapter selection never depends on
    /// registration order.
    pub fn register(&mut self, adapter: impl SourceAdapter + 'static) -> Result<(), RegistryError> {
        let extensions = adapter.extensions();
        if extensions.is_empty() {
            return Err(RegistryError::NoExtensions {
                language: adapter.language(),
            });
        }
        for extension in extensions {
            let extension = normalize_extension(extension);
            if self.extensions.contains_key(&extension) {
                return Err(RegistryError::DuplicateExtension { extension });
            }
        }
        let adapter_index = self.adapters.len();
        self.adapters.push(Box::new(adapter));
        for extension in self.adapters[adapter_index].extensions() {
            self.extensions
                .insert(normalize_extension(extension), adapter_index);
        }
        Ok(())
    }

    pub fn adapter_for(&self, source: &Path) -> Result<&dyn SourceAdapter, RegistryError> {
        let extension = source
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| RegistryError::MissingExtension {
                path: source.to_path_buf(),
            })?;
        self.extensions
            .get(&normalize_extension(extension))
            .and_then(|index| self.adapters.get(*index))
            .map(Box::as_ref)
            .ok_or_else(|| RegistryError::UnsupportedExtension {
                path: source.to_path_buf(),
                extension: extension.to_owned(),
            })
    }

    pub fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, RegistryLookupError> {
        self.adapter_for(source)
            .map_err(RegistryLookupError::Registry)?
            .semantic_graph(source)
            .map_err(RegistryLookupError::Adapter)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    NoExtensions { language: &'static str },
    DuplicateExtension { extension: String },
    MissingExtension { path: PathBuf },
    UnsupportedExtension { path: PathBuf, extension: String },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoExtensions { language } => {
                write!(formatter, "{language} adapter declares no file extensions")
            }
            Self::DuplicateExtension { extension } => {
                write!(formatter, "multiple adapters registered for '.{extension}'")
            }
            Self::MissingExtension { path } => {
                write!(
                    formatter,
                    "cannot select an adapter for '{}': missing file extension",
                    path.display()
                )
            }
            Self::UnsupportedExtension { path, extension } => write!(
                formatter,
                "unsupported source '{}': no adapter is registered for '.{extension}'",
                path.display()
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug)]
pub enum RegistryLookupError {
    Registry(RegistryError),
    Adapter(SourceAdapterError),
}

impl fmt::Display for RegistryLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => error.fmt(formatter),
            Self::Adapter(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RegistryLookupError {}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}

/// Registered extension point for a language whose official frontend is not
/// linked into this build yet. It makes language detection deterministic while
/// refusing to fabricate a partial parser.
pub struct UnavailableSourceAdapter {
    language: &'static str,
    extensions: &'static [&'static str],
    detail: String,
}

impl UnavailableSourceAdapter {
    pub fn new(
        language: &'static str,
        extensions: &'static [&'static str],
        detail: impl Into<String>,
    ) -> Self {
        Self {
            language,
            extensions,
            detail: detail.into(),
        }
    }
}

impl SemanticAdapter for UnavailableSourceAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new(
            format!("{}-official-frontend", self.language.to_ascii_lowercase()),
            LanguageTag::Custom(self.language.to_ascii_lowercase()),
            "1",
        )
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(false, false, false)
    }
}

impl SourceAdapter for UnavailableSourceAdapter {
    fn extensions(&self) -> &[&str] {
        self.extensions
    }

    fn language(&self) -> &'static str {
        self.language
    }

    fn semantic_graph(&self, _source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        Err(SourceAdapterError::FrontendUnavailable {
            language: self.language,
            detail: self.detail.clone(),
        })
    }
}
