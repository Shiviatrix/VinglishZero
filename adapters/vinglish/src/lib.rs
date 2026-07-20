//! Vinglish export-format adapter.
//!
//! This crate deliberately does not import Vinglish compiler crates. It accepts
//! only the compiler-owned JSON interchange contract and lowers it into the
//! shared Semantic IR. Source parsing, type checking, and all reasoning remain
//! outside this boundary.

use std::{env, ffi::OsString, fmt, fs, path::Path, process::Command};

use serde::Deserialize;
use vz_adapters::{
    AdapterCapabilities, AdapterManifest, SemanticAdapter, SourceAdapter, SourceAdapterError,
};
use vz_common::{
    LanguageTag, Mutability, NodeId, NodeIdAllocator, SemanticSpan, SourceId, Visibility,
};
use vz_semantic_ir::{
    node::{Argument, LiteralValue, Param, Scope},
    CollectionKind, LoopKind, Metadata, SemanticGraph, SemanticNode, SemanticOp, SemanticUnOp,
    TypeConcept,
};

const FORMAT_NAME: &str = "vinglish.semantic-export";
const SUPPORTED_VERSION: u32 = 1;
const COMPILER_ENVIRONMENT_VARIABLE: &str = "VZ_VINGLISH_COMPILER";

/// The deterministic importer for the compiler-owned Vinglish export format.
#[derive(Debug, Default)]
pub struct VinglishAdapter;

impl VinglishAdapter {
    pub fn import_json(&self, input: &str) -> Result<SemanticGraph, ImportError> {
        let document: Document = serde_json::from_str(input).map_err(ImportError::invalid_json)?;
        self.import_document(document)
    }

    fn import_document(&self, document: Document) -> Result<SemanticGraph, ImportError> {
        if document.format != FORMAT_NAME {
            return Err(ImportError::UnsupportedFormat(document.format));
        }
        if document.version != SUPPORTED_VERSION {
            return Err(ImportError::UnsupportedVersion(document.version));
        }

        let mut graph = SemanticGraph::new();
        let mut ids = NodeIdAllocator::new();
        let module_ids = document
            .program
            .modules
            .into_iter()
            .map(|module| self.module(&mut graph, &mut ids, module))
            .collect();

        graph.insert(SemanticNode::Program {
            id: NodeId::ROOT,
            modules: module_ids,
            span: span(SourceRange::default()),
            metadata: Metadata::default(),
        });
        graph
            .validate()
            .map_err(|error| ImportError::InvalidSemanticGraph(error.to_string()))?;
        Ok(graph)
    }

    fn module(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        module: Module,
    ) -> NodeId {
        let mut children = Vec::new();
        for function in module.functions {
            children.push(self.function(graph, ids, function));
        }
        for statement in module.statements {
            children.push(self.statement(graph, ids, statement));
        }
        let id = ids.next();
        graph.insert(SemanticNode::Module {
            id,
            name: module.name,
            language: LanguageTag::Vinglish,
            children,
            span: span(SourceRange::default()),
            metadata: Metadata::default(),
        });
        id
    }

    fn function(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        function: Function,
    ) -> NodeId {
        let body = function
            .body
            .into_iter()
            .map(|statement| self.statement(graph, ids, statement))
            .collect();
        let id = ids.next();
        graph.insert(SemanticNode::Function {
            id,
            name: function.name,
            params: function
                .parameters
                .into_iter()
                .map(|parameter| Param {
                    name: parameter.name,
                    type_concept: Some(type_concept(parameter.ty)),
                    mutability: Mutability::Immutable,
                    has_default: false,
                    span: span(parameter.span),
                })
                .collect(),
            return_type: Some(type_concept(function.return_type)),
            body,
            visibility: Visibility::Untracked,
            is_async: false,
            is_foreign: function.foreign,
            generic_params: Vec::new(),
            span: span(function.span),
            metadata: Metadata::default(),
        });
        id
    }

    fn statement(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        statement: Statement,
    ) -> NodeId {
        match statement {
            Statement::Variable {
                name,
                mutable,
                ty,
                initializer,
                span: source_span,
            } => {
                let initializer_id = self.expression(graph, ids, initializer);
                let id = ids.next();
                graph.insert(SemanticNode::Variable {
                    id,
                    name,
                    type_concept: Some(type_concept(ty)),
                    mutability: if mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                    initializer_id: Some(initializer_id),
                    scope: Scope::Local,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Statement::Assignment {
                target,
                value,
                span: source_span,
            } => {
                let target = self.expression(graph, ids, target);
                let value = self.expression(graph, ids, value);
                let id = ids.next();
                graph.insert(SemanticNode::Assignment {
                    id,
                    target,
                    value,
                    op: None,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Statement::Mutation {
                target,
                operation,
                value,
                span: source_span,
            } => {
                let target = self.expression(graph, ids, target);
                let value = self.expression(graph, ids, value);
                let id = ids.next();
                graph.insert(SemanticNode::Assignment {
                    id,
                    target,
                    value,
                    op: Some(match operation {
                        MutationOperation::Add => SemanticOp::Add,
                        MutationOperation::Subtract => SemanticOp::Sub,
                        MutationOperation::Multiply => SemanticOp::Mul,
                        MutationOperation::Divide => SemanticOp::Div,
                    }),
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Statement::Call { call, span } => self.call(graph, ids, call, Some(span)),
            Statement::Return {
                value,
                span: source_span,
            } => {
                let value = value.map(|value| self.expression(graph, ids, value));
                let id = ids.next();
                graph.insert(SemanticNode::Return {
                    id,
                    value,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Statement::Loop {
                loop_kind,
                body,
                span: source_span,
            } => {
                let loop_kind = match loop_kind {
                    TransportLoopKind::While { condition } => LoopKind::While {
                        condition: self.expression(graph, ids, condition),
                    },
                };
                let body = body
                    .into_iter()
                    .map(|statement| self.statement(graph, ids, statement))
                    .collect();
                let id = ids.next();
                graph.insert(SemanticNode::Loop {
                    id,
                    kind: loop_kind,
                    body,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Statement::Conditional {
                condition,
                then_body,
                else_body,
                span: source_span,
            } => {
                let condition = self.expression(graph, ids, condition);
                let then_body = then_body
                    .into_iter()
                    .map(|statement| self.statement(graph, ids, statement))
                    .collect();
                let else_body = else_body.map(|body| {
                    body.into_iter()
                        .map(|statement| self.statement(graph, ids, statement))
                        .collect()
                });
                let id = ids.next();
                graph.insert(SemanticNode::Conditional {
                    id,
                    condition,
                    then_body,
                    else_body,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
        }
    }

    fn expression(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        expression: Expression,
    ) -> NodeId {
        match expression {
            Expression::Identifier {
                name,
                span: source_span,
            } => self.insert_identifier(graph, ids, name, source_span),
            Expression::Literal {
                value,
                span: source_span,
            } => {
                let type_concept = match &value {
                    TransportLiteral::Integer(_) => TypeConcept::Integer {
                        bits: None,
                        signed: true,
                    },
                    TransportLiteral::Decimal(_) => TypeConcept::FloatingPoint { bits: None },
                    TransportLiteral::Text(_) => TypeConcept::Text,
                    TransportLiteral::Boolean(_) => TypeConcept::Boolean,
                    TransportLiteral::Unit => TypeConcept::Unit,
                };
                let value = match value {
                    TransportLiteral::Integer(value) => LiteralValue::Integer(value),
                    TransportLiteral::Decimal(value) => LiteralValue::Float(value),
                    TransportLiteral::Text(value) => LiteralValue::Text(value),
                    TransportLiteral::Boolean(value) => LiteralValue::Boolean(value),
                    TransportLiteral::Unit => LiteralValue::Null,
                };
                let id = ids.next();
                graph.insert(SemanticNode::Literal {
                    id,
                    value,
                    type_concept,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Expression::Call(call) => self.call(graph, ids, call, None),
            Expression::Binary {
                operation,
                left,
                right,
                span: source_span,
            } => {
                let left = self.expression(graph, ids, *left);
                let right = self.expression(graph, ids, *right);
                let id = ids.next();
                graph.insert(SemanticNode::BinaryOp {
                    id,
                    op: binary_operation(operation),
                    left,
                    right,
                    result_type: None,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Expression::Unary {
                operation,
                operand,
                span: source_span,
            } => {
                let operand = self.expression(graph, ids, *operand);
                let id = ids.next();
                graph.insert(SemanticNode::UnaryOp {
                    id,
                    op: match operation {
                        UnaryOperation::Negate => SemanticUnOp::Negate,
                        UnaryOperation::Not => SemanticUnOp::Not,
                    },
                    operand,
                    result_type: None,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Expression::Collection {
                elements,
                span: source_span,
            } => {
                let elements = elements
                    .into_iter()
                    .map(|element| self.expression(graph, ids, element))
                    .collect();
                let id = ids.next();
                graph.insert(SemanticNode::Collection {
                    id,
                    kind: CollectionKind::List,
                    elements,
                    element_type: None,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
            Expression::Unsupported { span: source_span } => {
                let id = ids.next();
                graph.insert(SemanticNode::Extension {
                    id,
                    tag: "vinglish_export::unsupported_expression".to_owned(),
                    payload: vz_semantic_ir::metadata::SemanticValue::Null,
                    span: span(source_span),
                    metadata: Metadata::default(),
                });
                id
            }
        }
    }

    fn call(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        call: Call,
        statement_span: Option<SourceRange>,
    ) -> NodeId {
        let callee = self.expression(graph, ids, *call.callee);
        let arguments = call
            .arguments
            .into_iter()
            .map(|value| Argument {
                label: None,
                value: self.expression(graph, ids, value),
            })
            .collect();
        let id = ids.next();
        graph.insert(SemanticNode::Call {
            id,
            callee,
            arguments,
            return_type: None,
            is_async_call: false,
            span: span(statement_span.unwrap_or(call.span)),
            metadata: Metadata::default(),
        });
        id
    }

    fn insert_identifier(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        name: String,
        source_span: SourceRange,
    ) -> NodeId {
        let id = ids.next();
        graph.insert(SemanticNode::Identifier {
            id,
            name,
            resolved_to: None,
            type_concept: None,
            span: span(source_span),
            metadata: Metadata::default(),
        });
        id
    }
}

impl SemanticAdapter for VinglishAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new("vinglish-export-v1", LanguageTag::Vinglish, "1")
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(true, false, true)
    }
}

impl SourceAdapter for VinglishAdapter {
    fn extensions(&self) -> &[&str] {
        &["ving", "json"]
    }

    fn language(&self) -> &'static str {
        "Vinglish"
    }

    fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        let input = match source.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("json") => fs::read_to_string(source)
                .map_err(|source_error| SourceAdapterError::ReadSource {
                    path: source.to_path_buf(),
                    source: source_error,
                })?,
            Some(extension) if extension.eq_ignore_ascii_case("ving") => {
                self.export_source(source)?
            }
            _ => {
                return Err(SourceAdapterError::AdapterFailure {
                    language: self.language(),
                    message: "expected a .ving source file or Vinglish semantic-export JSON"
                        .to_owned(),
                })
            }
        };
        self.import_json(&input)
            .map_err(|error| SourceAdapterError::InvalidSource {
                language: self.language(),
                message: error.to_string(),
            })
    }
}

impl VinglishAdapter {
    // The compiler remains an external process. Its only output consumed here
    // is the versioned JSON contract; no Vinglish crate or HIR is imported.
    fn export_source(&self, source: &Path) -> Result<String, SourceAdapterError> {
        let compiler =
            env::var_os(COMPILER_ENVIRONMENT_VARIABLE).unwrap_or_else(|| OsString::from("vng"));
        let output = Command::new(&compiler)
            .arg("--emit-ir")
            .arg(source)
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    SourceAdapterError::CompilerNotFound {
                        language: self.language(),
                        executable: compiler.to_string_lossy().into_owned(),
                    }
                } else {
                    SourceAdapterError::AdapterFailure {
                        language: self.language(),
                        message: format!("cannot launch '{}': {error}", compiler.to_string_lossy()),
                    }
                }
            })?;
        if !output.status.success() {
            return Err(SourceAdapterError::CompilerFailed {
                language: self.language(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        String::from_utf8(output.stdout).map_err(|_| SourceAdapterError::AdapterFailure {
            language: self.language(),
            message: "compiler emitted non-UTF-8 semantic export output".to_owned(),
        })
    }
}

#[derive(Debug)]
pub enum ImportError {
    InvalidJson(serde_json::Error),
    UnsupportedFormat(String),
    UnsupportedVersion(u32),
    InvalidSemanticGraph(String),
}

impl ImportError {
    fn invalid_json(error: serde_json::Error) -> Self {
        Self::InvalidJson(error)
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid Vinglish export JSON: {error}"),
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported export format: {format}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported export version: {version}")
            }
            Self::InvalidSemanticGraph(message) => {
                write!(
                    formatter,
                    "invalid Vinglish semantic export graph: {message}"
                )
            }
        }
    }
}

impl std::error::Error for ImportError {}

fn span(range: SourceRange) -> SemanticSpan {
    SemanticSpan::new(
        SourceId::UNKNOWN,
        range.start,
        range.end,
        LanguageTag::Vinglish,
    )
}

fn type_concept(ty: TransportType) -> TypeConcept {
    match ty {
        TransportType::Integer => TypeConcept::Integer {
            bits: None,
            signed: true,
        },
        TransportType::Decimal => TypeConcept::FloatingPoint { bits: None },
        TransportType::Boolean => TypeConcept::Boolean,
        TransportType::Text => TypeConcept::Text,
        TransportType::Unit => TypeConcept::Unit,
        TransportType::Collection { element } => {
            TypeConcept::Collection(Box::new(type_concept(*element)))
        }
        TransportType::Map { key, value } => TypeConcept::Map {
            key: Box::new(type_concept(*key)),
            value: Box::new(type_concept(*value)),
        },
        TransportType::Optional { inner } => TypeConcept::Optional(Box::new(type_concept(*inner))),
        TransportType::Result { ok, err } => TypeConcept::Result {
            ok: Box::new(type_concept(*ok)),
            err: Box::new(type_concept(*err)),
        },
        TransportType::Reference { mutable, inner } => TypeConcept::Reference {
            inner: Box::new(type_concept(*inner)),
            mutability: if mutable {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            },
        },
        TransportType::Pointer { inner } => TypeConcept::RawPointer(Box::new(type_concept(*inner))),
        TransportType::Function {
            parameters,
            returns,
        } => TypeConcept::Callable {
            params: parameters.into_iter().map(type_concept).collect(),
            return_type: Box::new(type_concept(*returns)),
            is_async: false,
        },
        TransportType::Named { name, arguments } => TypeConcept::Named {
            name,
            type_args: arguments.into_iter().map(type_concept).collect(),
        },
        TransportType::Unknown => TypeConcept::Unknown,
    }
}

fn binary_operation(operation: BinaryOperation) -> SemanticOp {
    match operation {
        BinaryOperation::Add => SemanticOp::Add,
        BinaryOperation::Subtract => SemanticOp::Sub,
        BinaryOperation::Multiply => SemanticOp::Mul,
        BinaryOperation::Divide => SemanticOp::Div,
        BinaryOperation::Remainder => SemanticOp::Rem,
        BinaryOperation::Equal => SemanticOp::Eq,
        BinaryOperation::NotEqual => SemanticOp::NotEq,
        BinaryOperation::LessThan => SemanticOp::Lt,
        BinaryOperation::GreaterThan => SemanticOp::Gt,
        BinaryOperation::LessThanOrEqual => SemanticOp::LtEq,
        BinaryOperation::GreaterThanOrEqual => SemanticOp::GtEq,
        BinaryOperation::And => SemanticOp::And,
        BinaryOperation::Or => SemanticOp::Or,
    }
}

// This mirror is intentionally private. It is a JSON decoder for the compiler
// contract, not a dependency on compiler implementation types.
#[derive(Deserialize)]
struct Document {
    format: String,
    version: u32,
    program: Program,
}
#[derive(Deserialize)]
struct Program {
    modules: Vec<Module>,
}
#[derive(Deserialize)]
struct Module {
    name: String,
    functions: Vec<Function>,
    #[serde(default)]
    statements: Vec<Statement>,
}
#[derive(Deserialize)]
struct Function {
    name: String,
    parameters: Vec<Parameter>,
    return_type: TransportType,
    body: Vec<Statement>,
    #[serde(default)]
    foreign: bool,
    span: SourceRange,
}
#[derive(Deserialize)]
struct Parameter {
    name: String,
    #[serde(rename = "type")]
    ty: TransportType,
    span: SourceRange,
}
#[derive(Clone, Copy, Default, Deserialize)]
struct SourceRange {
    start: u32,
    end: u32,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TransportType {
    Integer,
    Decimal,
    Boolean,
    Text,
    Unit,
    Collection {
        element: Box<TransportType>,
    },
    Map {
        key: Box<TransportType>,
        value: Box<TransportType>,
    },
    Optional {
        inner: Box<TransportType>,
    },
    Result {
        ok: Box<TransportType>,
        err: Box<TransportType>,
    },
    Reference {
        mutable: bool,
        inner: Box<TransportType>,
    },
    Pointer {
        inner: Box<TransportType>,
    },
    Function {
        parameters: Vec<TransportType>,
        returns: Box<TransportType>,
    },
    Named {
        name: String,
        arguments: Vec<TransportType>,
    },
    Unknown,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Statement {
    Variable {
        name: String,
        mutable: bool,
        #[serde(rename = "type")]
        ty: TransportType,
        initializer: Expression,
        span: SourceRange,
    },
    Assignment {
        target: Expression,
        value: Expression,
        span: SourceRange,
    },
    Mutation {
        target: Expression,
        operation: MutationOperation,
        value: Expression,
        span: SourceRange,
    },
    Call {
        call: Call,
        span: SourceRange,
    },
    Return {
        value: Option<Expression>,
        span: SourceRange,
    },
    Loop {
        loop_kind: TransportLoopKind,
        body: Vec<Statement>,
        span: SourceRange,
    },
    Conditional {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
        span: SourceRange,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum MutationOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TransportLoopKind {
    While { condition: Expression },
}
#[derive(Deserialize)]
struct Call {
    callee: Box<Expression>,
    arguments: Vec<Expression>,
    span: SourceRange,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Expression {
    Identifier {
        name: String,
        span: SourceRange,
    },
    Literal {
        value: TransportLiteral,
        span: SourceRange,
    },
    Call(Call),
    Binary {
        operation: BinaryOperation,
        left: Box<Expression>,
        right: Box<Expression>,
        span: SourceRange,
    },
    Unary {
        operation: UnaryOperation,
        operand: Box<Expression>,
        span: SourceRange,
    },
    Collection {
        elements: Vec<Expression>,
        span: SourceRange,
    },
    Unsupported {
        span: SourceRange,
    },
}
#[derive(Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum TransportLiteral {
    Integer(i64),
    Decimal(f64),
    Text(String),
    Boolean(bool),
    Unit,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum UnaryOperation {
    Negate,
    Not,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_the_v1_fixture_without_compiler_types() {
        let graph = VinglishAdapter
            .import_json(include_str!("../../../tests/fixtures/accumulate-v1.json"))
            .unwrap();
        assert!(matches!(
            graph.get(NodeId::ROOT),
            Some(SemanticNode::Program { .. })
        ));
        assert!(graph.iter().any(
            |(_, node)| matches!(node, SemanticNode::Function { name, .. } if name == "calculate")
        ));
    }

    #[test]
    fn rejects_unknown_contract_versions() {
        let error = VinglishAdapter
            .import_json(
                r#"{"format":"vinglish.semantic-export","version":99,"program":{"modules":[]}}"#,
            )
            .unwrap_err();
        assert!(matches!(error, ImportError::UnsupportedVersion(99)));
    }
}
