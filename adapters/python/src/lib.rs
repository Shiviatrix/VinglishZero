//! Python standard-library AST adapter.
//!
//! The adapter embeds Python in-process and calls only `ast.parse`; it never
//! starts a subprocess and does not use a third-party parser. A private DTO
//! separates the Python AST producer from the language-agnostic Semantic IR
//! lowering, so a future parser can replace that producer alone.

use std::{collections::BTreeSet, ffi::CString, fmt, fs, path::Path};

use pyo3::{prelude::*, types::PyModule};
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

/// Translates Python's standard `ast` output into Semantic IR.
#[derive(Debug, Default)]
pub struct PythonAdapter;

impl PythonAdapter {
    /// Parses Python source through the embedded standard library and lowers
    /// the supported AST subset into Semantic IR.
    pub fn from_source(
        &self,
        source_name: impl Into<String>,
        source: &str,
    ) -> Result<SemanticGraph, PythonAdapterError> {
        let source_name = source_name.into();
        let program = parse_standard_ast(source)?;
        Ok(self.lower(source_name, source, program))
    }

    fn lower(&self, source_name: String, source: &str, program: PyProgram) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        let mut ids = NodeIdAllocator::new();
        let source_map = SourceMap::new(source);
        let mut scope = ScopeState::default();
        let children =
            self.lower_block(&mut graph, &mut ids, &source_map, program.body, &mut scope);
        let module_id = ids.next();
        graph.insert(SemanticNode::Module {
            id: module_id,
            name: source_name,
            language: LanguageTag::Python,
            children,
            span: source_map.full_span(),
            metadata: Metadata::default(),
        });
        graph.insert(SemanticNode::Program {
            id: NodeId::ROOT,
            modules: vec![module_id],
            span: source_map.full_span(),
            metadata: Metadata::default(),
        });
        graph
    }

    fn lower_block(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        source_map: &SourceMap,
        statements: Vec<PyStmt>,
        scope: &mut ScopeState,
    ) -> Vec<NodeId> {
        statements
            .into_iter()
            .filter_map(|statement| self.lower_statement(graph, ids, source_map, statement, scope))
            .collect()
    }

    fn lower_statement(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        source_map: &SourceMap,
        statement: PyStmt,
        scope: &mut ScopeState,
    ) -> Option<NodeId> {
        match statement {
            PyStmt::Function {
                name,
                parameters,
                return_annotation,
                body,
                location,
            } => {
                let mut function_scope = ScopeState::default();
                for parameter in &parameters {
                    function_scope.declared.insert(parameter.name.clone());
                }
                let body = self.lower_block(graph, ids, source_map, body, &mut function_scope);
                let id = ids.next();
                graph.insert(SemanticNode::Function {
                    id,
                    name,
                    params: parameters
                        .into_iter()
                        .map(|parameter| Param {
                            name: parameter.name,
                            type_concept: parameter.annotation.map(type_concept),
                            mutability: Mutability::Untracked,
                            has_default: false,
                            span: source_map.span(parameter.location),
                        })
                        .collect(),
                    return_type: return_annotation.map(type_concept),
                    body,
                    visibility: Visibility::Untracked,
                    is_async: false,
                    is_foreign: false,
                    generic_params: Vec::new(),
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
            PyStmt::Assign {
                target,
                value,
                location,
            } => {
                let target_name = identifier_name(&target)?;
                let value_id = self.lower_expression(graph, ids, source_map, value.clone());
                let id = ids.next();
                if scope.declared.insert(target_name.clone()) {
                    graph.insert(SemanticNode::Variable {
                        id,
                        name: target_name,
                        type_concept: infer_type(&value),
                        mutability: Mutability::Mutable,
                        initializer_id: Some(value_id),
                        scope: Scope::Local,
                        span: source_map.span(location),
                        metadata: Metadata::default(),
                    });
                } else {
                    let target_id = self.lower_expression(graph, ids, source_map, target);
                    graph.insert(SemanticNode::Assignment {
                        id,
                        target: target_id,
                        value: value_id,
                        op: None,
                        span: source_map.span(location),
                        metadata: Metadata::default(),
                    });
                }
                Some(id)
            }
            PyStmt::Mutation {
                target,
                operation,
                value,
                location,
            } => {
                let target = self.lower_expression(graph, ids, source_map, target);
                let value = self.lower_expression(graph, ids, source_map, value);
                let id = ids.next();
                graph.insert(SemanticNode::Assignment {
                    id,
                    target,
                    value,
                    op: Some(operation.into()),
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
            PyStmt::Call { call, location } => {
                let call = self.lower_call(graph, ids, source_map, call);
                if let Some(SemanticNode::Call { span, .. }) = graph.get_mut(call) {
                    *span = source_map.span(location);
                }
                Some(call)
            }
            PyStmt::Return { value, location } => {
                let value = value.map(|value| self.lower_expression(graph, ids, source_map, value));
                let id = ids.next();
                graph.insert(SemanticNode::Return {
                    id,
                    value,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
            PyStmt::If {
                condition,
                then_body,
                else_body,
                location,
            } => {
                let condition = self.lower_expression(graph, ids, source_map, condition);
                let then_body = self.lower_block(graph, ids, source_map, then_body, scope);
                let else_body = if else_body.is_empty() {
                    None
                } else {
                    Some(self.lower_block(graph, ids, source_map, else_body, scope))
                };
                let id = ids.next();
                graph.insert(SemanticNode::Conditional {
                    id,
                    condition,
                    then_body,
                    else_body,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
            PyStmt::For {
                variable,
                iterable,
                body,
                location,
            } => {
                scope.declared.insert(variable.clone());
                let iterable = self.lower_expression(graph, ids, source_map, iterable);
                let body = self.lower_block(graph, ids, source_map, body, scope);
                let id = ids.next();
                graph.insert(SemanticNode::Loop {
                    id,
                    kind: LoopKind::ForEach { variable, iterable },
                    body,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
            PyStmt::While {
                condition,
                body,
                location,
            } => {
                let condition = self.lower_expression(graph, ids, source_map, condition);
                let body = self.lower_block(graph, ids, source_map, body, scope);
                let id = ids.next();
                graph.insert(SemanticNode::Loop {
                    id,
                    kind: LoopKind::While { condition },
                    body,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                Some(id)
            }
        }
    }

    fn lower_expression(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        source_map: &SourceMap,
        expression: PyExpr,
    ) -> NodeId {
        match expression {
            PyExpr::Identifier { name, location } => {
                let id = ids.next();
                graph.insert(SemanticNode::Identifier {
                    id,
                    name,
                    resolved_to: None,
                    type_concept: None,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                id
            }
            PyExpr::Literal { value, location } => {
                let (value, type_concept) = match value {
                    PyLiteral::Integer(value) => (
                        LiteralValue::Integer(value),
                        TypeConcept::Integer {
                            bits: None,
                            signed: true,
                        },
                    ),
                    PyLiteral::Float(value) => (
                        LiteralValue::Float(value),
                        TypeConcept::FloatingPoint { bits: None },
                    ),
                    PyLiteral::Text(value) => (LiteralValue::Text(value), TypeConcept::Text),
                    PyLiteral::Boolean(value) => {
                        (LiteralValue::Boolean(value), TypeConcept::Boolean)
                    }
                    PyLiteral::None => (LiteralValue::None, TypeConcept::Unit),
                };
                let id = ids.next();
                graph.insert(SemanticNode::Literal {
                    id,
                    value,
                    type_concept,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                id
            }
            PyExpr::Call {
                callee,
                arguments,
                location,
            } => self.lower_call(
                graph,
                ids,
                source_map,
                PyCall {
                    callee,
                    arguments,
                    location,
                },
            ),
            PyExpr::Binary {
                operation,
                left,
                right,
                location,
            } => {
                let left = self.lower_expression(graph, ids, source_map, *left);
                let right = self.lower_expression(graph, ids, source_map, *right);
                let id = ids.next();
                graph.insert(SemanticNode::BinaryOp {
                    id,
                    op: operation.into(),
                    left,
                    right,
                    result_type: None,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                id
            }
            PyExpr::Unary {
                operation,
                operand,
                location,
            } => {
                let operand = self.lower_expression(graph, ids, source_map, *operand);
                let id = ids.next();
                graph.insert(SemanticNode::UnaryOp {
                    id,
                    op: operation.into(),
                    operand,
                    result_type: None,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                id
            }
            PyExpr::Collection { elements, location } => {
                let elements = elements
                    .into_iter()
                    .map(|element| self.lower_expression(graph, ids, source_map, element))
                    .collect();
                let id = ids.next();
                graph.insert(SemanticNode::Collection {
                    id,
                    kind: CollectionKind::List,
                    elements,
                    element_type: None,
                    span: source_map.span(location),
                    metadata: Metadata::default(),
                });
                id
            }
            PyExpr::Unsupported { location } => unsupported(graph, ids, source_map.span(location)),
        }
    }

    fn lower_call(
        &self,
        graph: &mut SemanticGraph,
        ids: &mut NodeIdAllocator,
        source_map: &SourceMap,
        call: PyCall,
    ) -> NodeId {
        let callee = self.lower_expression(graph, ids, source_map, *call.callee);
        let arguments = call
            .arguments
            .into_iter()
            .map(|value| Argument {
                label: None,
                value: self.lower_expression(graph, ids, source_map, value),
            })
            .collect();
        let id = ids.next();
        graph.insert(SemanticNode::Call {
            id,
            callee,
            arguments,
            return_type: None,
            is_async_call: false,
            span: source_map.span(call.location),
            metadata: Metadata::default(),
        });
        id
    }
}

impl SemanticAdapter for PythonAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new("python-standard-ast", LanguageTag::Python, "1")
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(true, false, true)
    }
}

impl SourceAdapter for PythonAdapter {
    fn extensions(&self) -> &[&str] {
        &["py", "pyw"]
    }

    fn language(&self) -> &'static str {
        "Python"
    }

    fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        let input =
            fs::read_to_string(source).map_err(|source_error| SourceAdapterError::ReadSource {
                path: source.to_path_buf(),
                source: source_error,
            })?;
        self.from_source(source.display().to_string(), &input)
            .map_err(|error| SourceAdapterError::InvalidSource {
                language: self.language(),
                message: error.to_string(),
            })
    }
}

#[derive(Debug)]
pub enum PythonAdapterError {
    Python(PyErr),
    InvalidAstPayload(serde_json::Error),
}

impl fmt::Display for PythonAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python(error) => write!(formatter, "python ast error: {error}"),
            Self::InvalidAstPayload(error) => {
                write!(formatter, "invalid normalized python ast: {error}")
            }
        }
    }
}

impl std::error::Error for PythonAdapterError {}

fn parse_standard_ast(source: &str) -> Result<PyProgram, PythonAdapterError> {
    let payload = Python::with_gil(|py| -> PyResult<String> {
        let code = CString::new(NORMALIZER).expect("normalizer has no NUL byte");
        let file_name = CString::new("<vz-python-adapter>").expect("static name has no NUL byte");
        let module_name = CString::new("normalizer").expect("static name has no NUL byte");
        let module = PyModule::from_code(
            py,
            code.as_c_str(),
            file_name.as_c_str(),
            module_name.as_c_str(),
        )?;
        module.getattr("parse_program")?.call1((source,))?.extract()
    })
    .map_err(PythonAdapterError::Python)?;
    serde_json::from_str(&payload).map_err(PythonAdapterError::InvalidAstPayload)
}

fn identifier_name(expression: &PyExpr) -> Option<String> {
    match expression {
        PyExpr::Identifier { name, .. } => Some(name.clone()),
        _ => None,
    }
}

fn infer_type(expression: &PyExpr) -> Option<TypeConcept> {
    match expression {
        PyExpr::Literal { value, .. } => Some(match value {
            PyLiteral::Integer(_) => TypeConcept::Integer {
                bits: None,
                signed: true,
            },
            PyLiteral::Float(_) => TypeConcept::FloatingPoint { bits: None },
            PyLiteral::Text(_) => TypeConcept::Text,
            PyLiteral::Boolean(_) => TypeConcept::Boolean,
            PyLiteral::None => TypeConcept::Unit,
        }),
        PyExpr::Collection { .. } => Some(TypeConcept::Collection(Box::new(TypeConcept::Unknown))),
        _ => None,
    }
}

fn type_concept(annotation: PyType) -> TypeConcept {
    match annotation {
        PyType::Integer => TypeConcept::Integer {
            bits: None,
            signed: true,
        },
        PyType::Float => TypeConcept::FloatingPoint { bits: None },
        PyType::Boolean => TypeConcept::Boolean,
        PyType::Text => TypeConcept::Text,
        PyType::Unit => TypeConcept::Unit,
        PyType::Collection { element } => TypeConcept::Collection(Box::new(type_concept(*element))),
        PyType::Named { name } => TypeConcept::Named {
            name,
            type_args: Vec::new(),
        },
        PyType::Unknown => TypeConcept::Unknown,
    }
}

fn unsupported(graph: &mut SemanticGraph, ids: &mut NodeIdAllocator, span: SemanticSpan) -> NodeId {
    let id = ids.next();
    graph.insert(SemanticNode::Extension {
        id,
        tag: "python_ast::unsupported_expression".to_owned(),
        payload: vz_semantic_ir::metadata::SemanticValue::Null,
        span,
        metadata: Metadata::default(),
    });
    id
}

#[derive(Default)]
struct ScopeState {
    declared: BTreeSet<String>,
}

#[derive(Clone, Copy, Deserialize)]
struct Location {
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}

struct SourceMap {
    line_starts: Vec<usize>,
    len: usize,
}

impl SourceMap {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self {
            line_starts,
            len: source.len(),
        }
    }

    fn span(&self, location: Location) -> SemanticSpan {
        let start = self.offset(location.line, location.column);
        let end = self.offset(location.end_line, location.end_column);
        SemanticSpan::new(
            SourceId::UNKNOWN,
            start as u32,
            end as u32,
            LanguageTag::Python,
        )
    }

    fn full_span(&self) -> SemanticSpan {
        SemanticSpan::new(SourceId::UNKNOWN, 0, self.len as u32, LanguageTag::Python)
    }

    fn offset(&self, line: u32, column: u32) -> usize {
        self.line_starts
            .get(line.saturating_sub(1) as usize)
            .copied()
            .unwrap_or(self.len)
            .saturating_add(column as usize)
            .min(self.len)
    }
}

#[derive(Deserialize)]
struct PyProgram {
    body: Vec<PyStmt>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PyStmt {
    Function {
        name: String,
        parameters: Vec<PyParameter>,
        return_annotation: Option<PyType>,
        body: Vec<PyStmt>,
        location: Location,
    },
    Assign {
        target: PyExpr,
        value: PyExpr,
        location: Location,
    },
    Mutation {
        target: PyExpr,
        operation: PyBinaryOperation,
        value: PyExpr,
        location: Location,
    },
    Call {
        call: PyCall,
        location: Location,
    },
    Return {
        value: Option<PyExpr>,
        location: Location,
    },
    If {
        condition: PyExpr,
        then_body: Vec<PyStmt>,
        else_body: Vec<PyStmt>,
        location: Location,
    },
    For {
        variable: String,
        iterable: PyExpr,
        body: Vec<PyStmt>,
        location: Location,
    },
    While {
        condition: PyExpr,
        body: Vec<PyStmt>,
        location: Location,
    },
}

#[derive(Deserialize)]
struct PyParameter {
    name: String,
    annotation: Option<PyType>,
    location: Location,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PyExpr {
    Identifier {
        name: String,
        location: Location,
    },
    Literal {
        value: PyLiteral,
        location: Location,
    },
    Call {
        callee: Box<PyExpr>,
        arguments: Vec<PyExpr>,
        location: Location,
    },
    Binary {
        operation: PyBinaryOperation,
        left: Box<PyExpr>,
        right: Box<PyExpr>,
        location: Location,
    },
    Unary {
        operation: PyUnaryOperation,
        operand: Box<PyExpr>,
        location: Location,
    },
    Collection {
        elements: Vec<PyExpr>,
        location: Location,
    },
    Unsupported {
        location: Location,
    },
}

#[derive(Clone, Deserialize)]
struct PyCall {
    callee: Box<PyExpr>,
    arguments: Vec<PyExpr>,
    location: Location,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum PyLiteral {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    None,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PyType {
    Integer,
    Float,
    Boolean,
    Text,
    Unit,
    Collection { element: Box<PyType> },
    Named { name: String },
    Unknown,
}

/// Adapter-local operations avoid exposing Semantic IR's internal data shapes
/// through the parser-normalization boundary.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PyBinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl From<PyBinaryOperation> for SemanticOp {
    fn from(operation: PyBinaryOperation) -> Self {
        match operation {
            PyBinaryOperation::Add => Self::Add,
            PyBinaryOperation::Sub => Self::Sub,
            PyBinaryOperation::Mul => Self::Mul,
            PyBinaryOperation::Div => Self::Div,
            PyBinaryOperation::Rem => Self::Rem,
            PyBinaryOperation::Eq => Self::Eq,
            PyBinaryOperation::NotEq => Self::NotEq,
            PyBinaryOperation::Lt => Self::Lt,
            PyBinaryOperation::LtEq => Self::LtEq,
            PyBinaryOperation::Gt => Self::Gt,
            PyBinaryOperation::GtEq => Self::GtEq,
            PyBinaryOperation::And => Self::And,
            PyBinaryOperation::Or => Self::Or,
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PyUnaryOperation {
    Negate,
    Not,
}

impl From<PyUnaryOperation> for SemanticUnOp {
    fn from(operation: PyUnaryOperation) -> Self {
        match operation {
            PyUnaryOperation::Negate => Self::Negate,
            PyUnaryOperation::Not => Self::Not,
        }
    }
}

const NORMALIZER: &str = r#"
import ast
import json

def loc(node):
    return {
        "line": getattr(node, "lineno", 1),
        "column": getattr(node, "col_offset", 0),
        "end_line": getattr(node, "end_lineno", getattr(node, "lineno", 1)),
        "end_column": getattr(node, "end_col_offset", getattr(node, "col_offset", 0)),
    }

def typ(node):
    if node is None:
        return None
    if isinstance(node, ast.Name):
        names = {"int": "integer", "float": "float", "bool": "boolean", "str": "text", "None": "unit"}
        kind = names.get(node.id)
        return {"kind": kind} if kind else {"kind": "named", "name": node.id}
    if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name) and node.value.id in ("list", "set", "tuple"):
        return {"kind": "collection", "element": typ(node.slice) or {"kind": "unknown"}}
    return {"kind": "unknown"}

BINOPS = {
    ast.Add: "add", ast.Sub: "sub", ast.Mult: "mul", ast.Div: "div", ast.FloorDiv: "div", ast.Mod: "rem",
    ast.Eq: "eq", ast.NotEq: "not_eq", ast.Lt: "lt", ast.LtE: "lt_eq",
    ast.Gt: "gt", ast.GtE: "gt_eq", ast.And: "and", ast.Or: "or",
}
UNARYOPS = {ast.USub: "negate", ast.Not: "not"}

def expr(node):
    location = loc(node)
    if isinstance(node, ast.Name):
        return {"kind": "identifier", "name": node.id, "location": location}
    if isinstance(node, ast.Constant):
        if node.value is None:
            value = {"kind": "none"}
        elif isinstance(node.value, bool):
            value = {"kind": "boolean", "value": node.value}
        elif isinstance(node.value, int):
            value = {"kind": "integer", "value": node.value}
        elif isinstance(node.value, float):
            value = {"kind": "float", "value": node.value}
        elif isinstance(node.value, str):
            value = {"kind": "text", "value": node.value}
        else:
            return {"kind": "unsupported", "location": location}
        return {"kind": "literal", "value": value, "location": location}
    if isinstance(node, ast.Call):
        return {"kind": "call", "callee": expr(node.func), "arguments": [expr(arg) for arg in node.args], "location": location}
    if isinstance(node, ast.BinOp):
        operation = BINOPS.get(type(node.op))
        if operation:
            return {"kind": "binary", "operation": operation, "left": expr(node.left), "right": expr(node.right), "location": location}
    if isinstance(node, ast.BoolOp) and node.values:
        operation = BINOPS.get(type(node.op))
        if operation:
            value = expr(node.values[0])
            for next_value in node.values[1:]:
                value = {"kind": "binary", "operation": operation, "left": value, "right": expr(next_value), "location": location}
            return value
    if isinstance(node, ast.Compare) and node.ops and node.comparators:
        operation = BINOPS.get(type(node.ops[0]))
        if operation:
            return {"kind": "binary", "operation": operation, "left": expr(node.left), "right": expr(node.comparators[0]), "location": location}
    if isinstance(node, ast.UnaryOp):
        operation = UNARYOPS.get(type(node.op))
        if operation:
            return {"kind": "unary", "operation": operation, "operand": expr(node.operand), "location": location}
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return {"kind": "collection", "elements": [expr(element) for element in node.elts], "location": location}
    return {"kind": "unsupported", "location": location}

def statement(node):
    location = loc(node)
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        return {"kind": "function", "name": node.name,
                "parameters": [{"name": arg.arg, "annotation": typ(arg.annotation), "location": loc(arg)} for arg in node.args.args],
                "return_annotation": typ(node.returns), "body": block(node.body), "location": location}
    if isinstance(node, ast.Assign) and node.targets:
        return {"kind": "assign", "target": expr(node.targets[0]), "value": expr(node.value), "location": location}
    if isinstance(node, ast.AnnAssign) and node.value is not None:
        return {"kind": "assign", "target": expr(node.target), "value": expr(node.value), "location": location}
    if isinstance(node, ast.AugAssign):
        operation = BINOPS.get(type(node.op))
        if operation:
            return {"kind": "mutation", "target": expr(node.target), "operation": operation, "value": expr(node.value), "location": location}
    if isinstance(node, ast.Expr) and isinstance(node.value, ast.Call):
        return {"kind": "call", "call": expr(node.value), "location": location}
    if isinstance(node, ast.Return):
        return {"kind": "return", "value": expr(node.value) if node.value else None, "location": location}
    if isinstance(node, ast.If):
        return {"kind": "if", "condition": expr(node.test), "then_body": block(node.body), "else_body": block(node.orelse), "location": location}
    if isinstance(node, ast.For) and isinstance(node.target, ast.Name):
        return {"kind": "for", "variable": node.target.id, "iterable": expr(node.iter), "body": block(node.body), "location": location}
    if isinstance(node, ast.While):
        return {"kind": "while", "condition": expr(node.test), "body": block(node.body), "location": location}
    return None

def block(nodes):
    return [item for item in (statement(node) for node in nodes) if item is not None]

def parse_program(source):
    return json.dumps({"body": block(ast.parse(source).body)}, sort_keys=True, separators=(",", ":"))
"#;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PythonAdapter;
    use vz_adapter_vinglish::VinglishAdapter;
    use vz_adapters::SourceAdapter;
    use vz_diagnostics::{
        model::Interpretation, CompilerDiagnostic, DiagnosticCategory, SemanticDiagnosticEngine,
        Severity,
    };
    use vz_reasoning::{report::HypothesisStatus, ReasoningEngine};
    use vz_semantic_ir::SemanticNode;

    const ACCUMULATOR: &str = include_str!("../../../examples/python/accumulator.py");
    const COUNTER: &str = include_str!("../../../examples/python/counter.py");
    const AVERAGE: &str = include_str!("../../../examples/python/average.py");
    const VALIDATOR: &str = include_str!("../../../examples/python/validator.py");
    const MAPPER: &str = include_str!("../../../examples/python/mapper.py");
    const FILTER: &str = include_str!("../../../examples/python/filter.py");
    const MAXIMUM: &str = include_str!("../../../examples/python/maximum.py");

    #[test]
    fn lowers_the_supported_python_subset() {
        let source = r#"
def transform(values: list[int]) -> int:
    total = 0
    for value in values:
        total += -value
    if total > 0:
        return helper(total)
    while total < 0:
        total += 1
    return total
"#;
        let graph = PythonAdapter::default()
            .from_source("transform.py", source)
            .unwrap();

        assert!(graph
            .iter()
            .any(|(_, node)| matches!(node, SemanticNode::Program { .. })));
        assert!(graph
            .iter()
            .any(|(_, node)| matches!(node, SemanticNode::Module { .. })));
        assert!(graph.iter().any(|(_, node)| matches!(node, SemanticNode::Function { name, params, .. } if name == "transform" && params.len() == 1)));
        assert!(graph.iter().any(
            |(_, node)| matches!(node, SemanticNode::Variable { name, .. } if name == "total")
        ));
        assert_eq!(
            graph
                .iter()
                .filter(|(_, node)| matches!(node, SemanticNode::Assignment { op: Some(_), .. }))
                .count(),
            2
        );
        assert!(graph
            .iter()
            .any(|(_, node)| matches!(node, SemanticNode::Call { .. })));
        assert_eq!(
            graph
                .iter()
                .filter(|(_, node)| matches!(node, SemanticNode::Return { .. }))
                .count(),
            2
        );
        assert!(graph
            .iter()
            .any(|(_, node)| matches!(node, SemanticNode::Conditional { .. })));
        assert_eq!(
            graph
                .iter()
                .filter(|(_, node)| matches!(node, SemanticNode::Loop { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn source_adapter_reads_python_files_directly() {
        let graph = PythonAdapter
            .semantic_graph(Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/python/accumulator.py"
            )))
            .unwrap();

        assert!(graph.iter().any(
            |(_, node)| matches!(node, SemanticNode::Function { name, .. } if name == "calculate")
        ));
    }

    #[test]
    fn examples_produce_expected_active_hypotheses_deterministically() {
        for (name, source, expected) in [
            ("accumulator", ACCUMULATOR, "accumulator"),
            ("counter", COUNTER, "counter"),
            ("average", AVERAGE, "average"),
            ("validator", VALIDATOR, "validator"),
            ("mapper", MAPPER, "mapper"),
            ("filter", FILTER, "filter"),
            ("maximum", MAXIMUM, "maximum"),
        ] {
            let graph = PythonAdapter::default()
                .from_source(format!("{name}.py"), source)
                .unwrap();
            let engine = ReasoningEngine::new();
            let first = engine.analyze(&graph);
            let second = engine.analyze(&graph);

            assert_eq!(first, second, "{name} report must be deterministic");
            assert!(
                first.function_reports[0]
                    .hypotheses
                    .iter()
                    .any(|hypothesis| {
                        hypothesis.id == expected && hypothesis.status == HypothesisStatus::Active
                    }),
                "{name} should classify as {expected}"
            );
        }
    }

    #[test]
    fn python_and_vinglish_accumulators_have_the_same_intent() {
        let python_graph = PythonAdapter
            .semantic_graph(Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/python/accumulator.py"
            )))
            .unwrap();
        let vinglish_graph = VinglishAdapter
            .semantic_graph(Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/accumulate-v1.json"
            )))
            .unwrap();
        let engine = ReasoningEngine::new();

        let python_report = engine.analyze(&python_graph);
        let vinglish_report = engine.analyze(&vinglish_graph);
        let python = active_hypothesis(&python_report, "accumulator");
        let vinglish = active_hypothesis(&vinglish_report, "accumulator");

        assert_eq!(python.confidence, vinglish.confidence);
        assert_eq!(python.status, vinglish.status);
    }

    #[test]
    fn python_intent_report_is_accepted_by_language_agnostic_diagnostics() {
        let graph = PythonAdapter::default()
            .from_source("calculate.py", ACCUMULATOR)
            .unwrap();
        let report = ReasoningEngine::new().analyze(&graph);
        let diagnostic = CompilerDiagnostic {
            code: "PY-001".to_owned(),
            severity: Severity::Error,
            category: DiagnosticCategory::TypeMismatch,
            message: "unsupported operand types for +=".to_owned(),
            location: None,
            function_name: Some("calculate".to_owned()),
        };

        let semantic_diagnostic = SemanticDiagnosticEngine::new()
            .diagnose(diagnostic, &report)
            .unwrap();

        assert_eq!(
            semantic_diagnostic.interpretation,
            Interpretation::AccumulatorTypeConflict
        );
        assert_eq!(
            semantic_diagnostic.intent.unwrap().hypothesis_id,
            "accumulator"
        );
    }

    fn active_hypothesis<'a>(
        report: &'a vz_reasoning::IntentReport,
        id: &str,
    ) -> &'a vz_reasoning::HypothesisReport {
        report.function_reports[0]
            .hypotheses
            .iter()
            .find(|hypothesis| hypothesis.id == id && hypothesis.status == HypothesisStatus::Active)
            .unwrap()
    }
}
