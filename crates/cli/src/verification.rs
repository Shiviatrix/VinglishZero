//! Cross-language regression verification over existing adapters and reasoning.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use vz_adapter_vinglish::VinglishAdapter;
use vz_reasoning::{report::HypothesisStatus, ReasoningEngine};

use crate::registry::default_registry;

struct CorpusPattern {
    id: &'static str,
    function: &'static str,
    sources: &'static [(&'static str, &'static str)],
}

const ACCUMULATOR_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/accumulator.py"),
    ("Java", "examples/java/Accumulator.java"),
    ("C", "examples/c/accumulator.c"),
    ("Vinglish", "tests/fixtures/accumulate-v1.json"),
];
const COUNTER_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/counter.py"),
    ("Java", "examples/java/Counter.java"),
    ("C", "examples/c/counter.c"),
    ("Vinglish", "tests/fixtures/counter-v1.json"),
];
const MAXIMUM_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/maximum.py"),
    ("Java", "examples/java/Maximum.java"),
    ("C", "examples/c/maximum.c"),
    ("Vinglish", "tests/fixtures/maximum-v1.json"),
];
const MINIMUM_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/minimum.py"),
    ("Java", "examples/java/Minimum.java"),
    ("C", "examples/c/minimum.c"),
    ("Vinglish", "tests/fixtures/minimum-v1.json"),
];
const AVERAGE_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/average.py"),
    ("Java", "examples/java/Average.java"),
    ("C", "examples/c/average.c"),
    ("Vinglish", "tests/fixtures/average-v1.json"),
];
const PRODUCT_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/product.py"),
    ("Java", "examples/java/Product.java"),
    ("C", "examples/c/product.c"),
    ("Vinglish", "tests/fixtures/product-v1.json"),
];
const REDUCER_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/reducer.py"),
    ("Java", "examples/java/Reducer.java"),
    ("C", "examples/c/reducer.c"),
    ("Vinglish", "tests/fixtures/reducer-v1.json"),
];
const MAPPER_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/mapper.py"),
    ("Java", "examples/java/Mapper.java"),
    ("C", "examples/c/mapper.c"),
    ("Vinglish", "tests/fixtures/mapper-v1.json"),
];
const FILTER_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/filter.py"),
    ("Java", "examples/java/Filter.java"),
    ("C", "examples/c/filter.c"),
    ("Vinglish", "tests/fixtures/filter-v1.json"),
];
const VALIDATOR_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/validator.py"),
    ("Java", "examples/java/Validator.java"),
    ("C", "examples/c/validator.c"),
    ("Vinglish", "tests/fixtures/validator-v1.json"),
];
const LINEAR_SEARCH_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/linear_search.py"),
    ("Java", "examples/java/LinearSearch.java"),
    ("C", "examples/c/linear_search.c"),
    ("Vinglish", "tests/fixtures/linear-search-v1.json"),
];
const BINARY_SEARCH_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/binary_search.py"),
    ("Java", "examples/java/BinarySearch.java"),
    ("C", "examples/c/binary_search.c"),
    ("Vinglish", "tests/fixtures/binary-search-v1.json"),
];

const COLLECTION_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/collection_patterns.py"),
    ("Java", "examples/java/CollectionPatterns.java"),
    ("C", "examples/c/collection_patterns.c"),
    ("Vinglish", "tests/fixtures/collection-patterns-v1.json"),
];
const COMPOSITION_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/compositions.py"),
    ("Java", "examples/java/CompositionPatterns.java"),
    ("C", "examples/c/compositions.c"),
    ("Vinglish", "tests/fixtures/compositions-v1.json"),
];

const CORPUS: &[CorpusPattern] = &[
    CorpusPattern {
        id: "accumulator",
        function: "calculate",
        sources: ACCUMULATOR_SOURCES,
    },
    CorpusPattern {
        id: "counter",
        function: "count_items",
        sources: COUNTER_SOURCES,
    },
    CorpusPattern {
        id: "maximum",
        function: "find_maximum",
        sources: MAXIMUM_SOURCES,
    },
    CorpusPattern {
        id: "minimum",
        function: "find_minimum",
        sources: MINIMUM_SOURCES,
    },
    CorpusPattern {
        id: "average",
        function: "average",
        sources: AVERAGE_SOURCES,
    },
    CorpusPattern {
        id: "product",
        function: "product",
        sources: PRODUCT_SOURCES,
    },
    CorpusPattern {
        id: "reducer",
        function: "reducer",
        sources: REDUCER_SOURCES,
    },
    CorpusPattern {
        id: "mapper",
        function: "mapper",
        sources: MAPPER_SOURCES,
    },
    CorpusPattern {
        id: "filter",
        function: "filter_positive",
        sources: FILTER_SOURCES,
    },
    CorpusPattern {
        id: "validator",
        function: "validate",
        sources: VALIDATOR_SOURCES,
    },
    CorpusPattern {
        id: "linear_search",
        function: "linear_search",
        sources: LINEAR_SEARCH_SOURCES,
    },
    CorpusPattern {
        id: "binary_search",
        function: "binary_search",
        sources: BINARY_SEARCH_SOURCES,
    },
    CorpusPattern {
        id: "sum",
        function: "sum_values",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "count_if",
        function: "count_if",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "any",
        function: "any_match",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "all",
        function: "all_positive",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "find_first",
        function: "find_first",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "find_last",
        function: "find_last",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "reverse_traversal",
        function: "reverse_traversal",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "prefix_sum",
        function: "prefix_sum",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "frequency_counter",
        function: "frequency_counter",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "histogram",
        function: "histogram",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "max_by",
        function: "max_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "min_by",
        function: "min_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "group_by",
        function: "group_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "partition",
        function: "partition",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "unique",
        function: "unique",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "distinct",
        function: "distinct",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "zip",
        function: "zip_items",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "flatten",
        function: "flatten",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "contains",
        function: "contains",
        sources: COLLECTION_SOURCES,
    },
];

#[derive(Debug, Serialize)]
pub struct VerificationReport {
    pub(crate) version: u32,
    pub(crate) patterns: Vec<PatternReport>,
}
impl VerificationReport {
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct PatternReport {
    pub(crate) pattern: String,
    pub(crate) entries: Vec<VerificationEntry>,
    pub(crate) passed: bool,
}
#[derive(Debug, Serialize)]
pub(crate) struct VerificationEntry {
    pub(crate) language: String,
    pub(crate) source: String,
    pub(crate) nodes: usize,
    pub(crate) primary_intent: Option<String>,
    pub(crate) confidence: Option<u8>,
    pub(crate) semantic_pipeline: Vec<String>,
}

#[derive(Debug)]
pub enum VerificationError {
    Registry(String),
    Adapter(String),
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Import(String),
    ExpectedIntentMissing {
        pattern: String,
        language: String,
        function: String,
    },
    PipelineMismatch {
        language: String,
        function: String,
        expected: Vec<String>,
        actual: Vec<String>,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Serialize(serde_json::Error),
}
impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(message) | Self::Adapter(message) | Self::Import(message) => {
                f.write_str(message)
            }
            Self::Read { path, source } | Self::Write { path, source } => {
                write!(f, "cannot access '{}': {source}", path.display())
            }
            Self::ExpectedIntentMissing {
                pattern,
                language,
                function,
            } => write!(
                f,
                "{language} did not activate '{pattern}' for function '{function}'"
            ),
            Self::PipelineMismatch {
                language,
                function,
                expected,
                actual,
            } => write!(
                f,
                "{language} produced pipeline [{}] instead of [{}] for function '{function}'",
                actual.join(", "),
                expected.join(", ")
            ),
            Self::Serialize(error) => write!(f, "cannot serialize verification report: {error}"),
        }
    }
}
impl std::error::Error for VerificationError {}

pub fn run() -> Result<VerificationReport, VerificationError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry =
        default_registry().map_err(|error| VerificationError::Registry(error.to_string()))?;
    let engine = ReasoningEngine::new();
    let mut patterns = Vec::new();
    for pattern in CORPUS {
        let mut entries = Vec::new();
        for (language, relative) in pattern.sources {
            let path = root.join(relative);
            let graph = if path.extension().and_then(|extension| extension.to_str()) == Some("json")
            {
                let input =
                    fs::read_to_string(&path).map_err(|source| VerificationError::Read {
                        path: path.clone(),
                        source,
                    })?;
                VinglishAdapter
                    .import_json(&input)
                    .map_err(|error| VerificationError::Import(error.to_string()))?
            } else {
                registry
                    .semantic_graph(&path)
                    .map_err(|error| VerificationError::Adapter(error.to_string()))?
            };
            let report = engine.analyze(&graph);
            let matching_function = report
                .function_reports
                .iter()
                .find(|function| function.function_name == pattern.function)
                .ok_or_else(|| VerificationError::ExpectedIntentMissing {
                    pattern: pattern.id.to_owned(),
                    language: (*language).to_owned(),
                    function: pattern.function.to_owned(),
                })?;
            let primary = matching_function.hypotheses.iter().find(|hypothesis| {
                hypothesis.id == pattern.id && hypothesis.status == HypothesisStatus::Active
            });
            if primary.is_none() {
                return Err(VerificationError::ExpectedIntentMissing {
                    pattern: pattern.id.to_owned(),
                    language: (*language).to_owned(),
                    function: pattern.function.to_owned(),
                });
            }
            entries.push(VerificationEntry {
                language: (*language).to_owned(),
                source: (*relative).to_owned(),
                nodes: graph.len(),
                primary_intent: matching_function.primary_intent.clone(),
                confidence: primary.map(|hypothesis| hypothesis.confidence),
                semantic_pipeline: matching_function
                    .semantic_pipeline
                    .iter()
                    .map(|stage| stage.hypothesis_id.clone())
                    .collect(),
            });
        }
        patterns.push(PatternReport {
            pattern: pattern.id.to_owned(),
            entries,
            passed: true,
        });
    }
    let expected_pipeline = ["filter", "mapper", "reducer"];
    let mut entries = Vec::new();
    for (language, relative) in COMPOSITION_SOURCES {
        let path = root.join(relative);
        let graph = if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            let input = fs::read_to_string(&path).map_err(|source| VerificationError::Read {
                path: path.clone(),
                source,
            })?;
            VinglishAdapter
                .import_json(&input)
                .map_err(|error| VerificationError::Import(error.to_string()))?
        } else {
            registry
                .semantic_graph(&path)
                .map_err(|error| VerificationError::Adapter(error.to_string()))?
        };
        let report = engine.analyze(&graph);
        let function = report
            .function_reports
            .iter()
            .find(|function| function.function_name == "filter_map_reduce")
            .ok_or_else(|| VerificationError::ExpectedIntentMissing {
                pattern: "filter_map_reduce".to_owned(),
                language: (*language).to_owned(),
                function: "filter_map_reduce".to_owned(),
            })?;
        let semantic_pipeline = function
            .semantic_pipeline
            .iter()
            .map(|stage| stage.hypothesis_id.clone())
            .collect::<Vec<_>>();
        if semantic_pipeline
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != expected_pipeline
        {
            return Err(VerificationError::PipelineMismatch {
                language: (*language).to_owned(),
                function: "filter_map_reduce".to_owned(),
                expected: expected_pipeline
                    .iter()
                    .map(|stage| (*stage).to_owned())
                    .collect(),
                actual: semantic_pipeline,
            });
        }
        entries.push(VerificationEntry {
            language: (*language).to_owned(),
            source: (*relative).to_owned(),
            nodes: graph.len(),
            primary_intent: Some("filter_map_reduce".to_owned()),
            confidence: None,
            semantic_pipeline: expected_pipeline
                .iter()
                .map(|stage| (*stage).to_owned())
                .collect(),
        });
    }
    patterns.push(PatternReport {
        pattern: "filter_map_reduce".to_owned(),
        entries,
        passed: true,
    });
    let report = VerificationReport {
        version: 1,
        patterns,
    };
    write_reports(&root, &report)?;
    Ok(report)
}

fn write_reports(root: &Path, report: &VerificationReport) -> Result<(), VerificationError> {
    let directory = root.join("verification");
    fs::create_dir_all(&directory).map_err(|source| VerificationError::Write {
        path: directory.clone(),
        source,
    })?;
    let json = directory.join("cross-language-report.json");
    fs::write(
        &json,
        serde_json::to_vec_pretty(report).map_err(VerificationError::Serialize)?,
    )
    .map_err(|source| VerificationError::Write { path: json, source })?;
    let markdown = directory.join("cross-language-report.md");
    let mut body = String::from("# Cross-Language Verification\n\n");
    for pattern in &report.patterns {
        body.push_str(&format!(
            "## {}\n\n| Language | Intent | Pipeline | Confidence | Nodes |\n| --- | --- | --- | ---: | ---: |\n",
            pattern.pattern
        ));
        for entry in &pattern.entries {
            body.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                entry.language,
                entry.primary_intent.as_deref().unwrap_or("none"),
                if entry.semantic_pipeline.is_empty() {
                    "none".to_owned()
                } else {
                    entry.semantic_pipeline.join(" -> ")
                },
                entry
                    .confidence
                    .map_or_else(|| "none".to_owned(), |value| value.to_string()),
                entry.nodes
            ));
        }
        body.push('\n');
    }
    fs::write(&markdown, body).map_err(|source| VerificationError::Write {
        path: markdown,
        source,
    })
}
