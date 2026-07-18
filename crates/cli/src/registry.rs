//! CLI composition of independently implemented source adapters.
//!
//! This module performs registration only. Extension matching and all frontend
//! behavior live in `vz-adapters` and the individual adapter crates.

use vz_adapter_c::CAdapter;
use vz_adapter_java::JavaAdapter;
use vz_adapter_python::PythonAdapter;
use vz_adapter_vinglish::VinglishAdapter;
use vz_adapters::{AdapterRegistry, RegistryError, UnavailableSourceAdapter};

pub fn default_registry() -> Result<AdapterRegistry, RegistryError> {
    let mut registry = AdapterRegistry::new();
    registry.register(PythonAdapter)?;
    registry.register(VinglishAdapter)?;
    registry.register(JavaAdapter)?;
    registry.register(CAdapter)?;

    // These are explicit official-frontend integration points. They are
    // registered now so detection is stable, but do not invent parsers when
    // their official compiler API is not part of this build.
    registry.register(UnavailableSourceAdapter::new(
        "Rust",
        &["rs"],
        "rust-analyzer or a stable Rust frontend integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "Go",
        &["go"],
        "go/parser and go/ast integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "C++",
        &["cc", "cpp", "cxx", "hpp", "hxx"],
        "an official C++ frontend integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "C#",
        &["cs"],
        "Roslyn integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "Kotlin",
        &["kt", "kts"],
        "Kotlin compiler frontend integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "Swift",
        &["swift"],
        "SwiftSyntax or official Swift parser integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "JavaScript",
        &["js", "mjs", "cjs"],
        "TypeScript compiler API integration is not enabled in this build",
    ))?;
    registry.register(UnavailableSourceAdapter::new(
        "TypeScript",
        &["ts", "tsx"],
        "TypeScript compiler API integration is not enabled in this build",
    ))?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::default_registry;

    #[test]
    fn selects_adapters_from_extensions_without_cli_branches() {
        let registry = default_registry().unwrap();

        assert_eq!(
            registry
                .adapter_for(Path::new("example.py"))
                .unwrap()
                .language(),
            "Python"
        );
        assert_eq!(
            registry
                .adapter_for(Path::new("example.ving"))
                .unwrap()
                .language(),
            "Vinglish"
        );
        assert_eq!(
            registry
                .adapter_for(Path::new("example.java"))
                .unwrap()
                .language(),
            "Java"
        );
    }
}
