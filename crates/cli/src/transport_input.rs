//! Acquisition of compiler-owned semantic transport for the CLI.
//!
//! Vinglish Zero never reads Vinglish source or compiler implementation data.
//! For `.ving` input this boundary invokes the installed compiler and retains
//! only its versioned JSON export, which follows the same import path as a
//! transport file supplied directly by the user.

use std::{
    env,
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

const COMPILER_ENVIRONMENT_VARIABLE: &str = "VZ_VINGLISH_COMPILER";

/// Obtains Vinglish semantic-export JSON from a transport file or source file.
#[derive(Debug, Clone)]
pub struct SemanticTransportAcquirer {
    compiler: OsString,
}

impl Default for SemanticTransportAcquirer {
    fn default() -> Self {
        Self::new(
            env::var_os(COMPILER_ENVIRONMENT_VARIABLE).unwrap_or_else(|| OsString::from("vng")),
        )
    }
}

impl SemanticTransportAcquirer {
    /// Uses `compiler` as the executable for `.ving` source inputs.
    pub fn new(compiler: impl Into<OsString>) -> Self {
        Self {
            compiler: compiler.into(),
        }
    }

    /// Returns the compiler-owned transport JSON for `input`.
    ///
    /// `.json` is read directly. `.ving` is supplied unchanged to the compiler
    /// through `vng --emit-ir <source>` and its stdout is kept in memory.
    pub fn acquire(&self, input: impl AsRef<Path>) -> Result<String, TransportInputError> {
        let input = input.as_ref();
        match input.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("json") => fs::read_to_string(input)
                .map_err(|source| TransportInputError::Read {
                    path: input.to_path_buf(),
                    source,
                }),
            Some(extension) if extension.eq_ignore_ascii_case("ving") => self.export_source(input),
            _ => Err(TransportInputError::UnsupportedInput {
                path: input.to_path_buf(),
            }),
        }
    }

    fn export_source(&self, source: &Path) -> Result<String, TransportInputError> {
        let output = Command::new(&self.compiler)
            .arg("--emit-ir")
            .arg(source)
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    TransportInputError::CompilerNotFound {
                        compiler: self.compiler.clone(),
                    }
                } else {
                    TransportInputError::CompilerLaunch {
                        compiler: self.compiler.clone(),
                        source: error,
                    }
                }
            })?;

        if !output.status.success() {
            return Err(TransportInputError::CompilerFailed {
                compiler: self.compiler.clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }

        String::from_utf8(output.stdout).map_err(|_| TransportInputError::NonUtf8CompilerOutput {
            compiler: self.compiler.clone(),
        })
    }
}

#[derive(Debug)]
pub enum TransportInputError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    UnsupportedInput {
        path: PathBuf,
    },
    CompilerNotFound {
        compiler: OsString,
    },
    CompilerLaunch {
        compiler: OsString,
        source: std::io::Error,
    },
    CompilerFailed {
        compiler: OsString,
        exit_code: Option<i32>,
        stderr: String,
    },
    NonUtf8CompilerOutput {
        compiler: OsString,
    },
}

impl fmt::Display for TransportInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "cannot read '{}': {source}", path.display()),
            Self::UnsupportedInput { path } => write!(
                formatter,
                "unsupported input '{}': expected a .ving source file or .json semantic export",
                path.display()
            ),
            Self::CompilerNotFound { compiler } => write!(
                formatter,
                "Vinglish compiler '{}' was not found; install it or set {COMPILER_ENVIRONMENT_VARIABLE}",
                compiler.to_string_lossy()
            ),
            Self::CompilerLaunch { compiler, source } => write!(
                formatter,
                "cannot launch Vinglish compiler '{}': {source}",
                compiler.to_string_lossy()
            ),
            Self::CompilerFailed {
                compiler,
                exit_code,
                stderr,
            } => {
                write!(
                    formatter,
                    "Vinglish compiler '{}' failed{}",
                    compiler.to_string_lossy(),
                    exit_code.map(|code| format!(" with exit code {code}")).unwrap_or_default()
                )?;
                if !stderr.is_empty() {
                    write!(formatter, ": {stderr}")?;
                }
                Ok(())
            }
            Self::NonUtf8CompilerOutput { compiler } => write!(
                formatter,
                "Vinglish compiler '{}' emitted non-UTF-8 semantic export output",
                compiler.to_string_lossy()
            ),
        }
    }
}

impl std::error::Error for TransportInputError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inputs_that_are_neither_source_nor_transport() {
        let error = SemanticTransportAcquirer::default()
            .acquire("program.txt")
            .unwrap_err();
        assert!(matches!(
            error,
            TransportInputError::UnsupportedInput { .. }
        ));
    }
}
