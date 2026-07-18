//! Java Compiler API source adapter.
//!
//! The embedded helper uses `javax.tools.JavaCompiler` and `com.sun.source`
//! trees. It does not parse Java itself; this crate only lowers its DTO into
//! language-agnostic Semantic IR.

use std::{env, ffi::OsString, fs, path::Path, process::Command};

use serde_json::Value;
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

const HELPER: &str = include_str!("../frontend/VzJavaFrontend.java");

#[derive(Debug, Default)]
pub struct JavaAdapter;

impl SemanticAdapter for JavaAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new(
            "java-compiler-api",
            LanguageTag::Custom("java".to_owned()),
            "1",
        )
    }
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(true, false, true)
    }
}

impl SourceAdapter for JavaAdapter {
    fn extensions(&self) -> &[&str] {
        &["java"]
    }
    fn language(&self) -> &'static str {
        "Java"
    }
    fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        let payload = run_frontend(source)?;
        let root: Value =
            serde_json::from_str(&payload).map_err(|error| SourceAdapterError::AdapterFailure {
                language: self.language(),
                message: format!("invalid Java Compiler API payload: {error}"),
            })?;
        lower(source, &root).map_err(|message| SourceAdapterError::AdapterFailure {
            language: self.language(),
            message,
        })
    }
}

fn run_frontend(source: &Path) -> Result<String, SourceAdapterError> {
    if !source.is_file() {
        return Err(SourceAdapterError::ReadSource {
            path: source.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "source file does not exist"),
        });
    }
    let javac = env::var_os("VZ_JAVAC").unwrap_or_else(|| OsString::from("javac"));
    let java = env::var_os("VZ_JAVA").unwrap_or_else(|| OsString::from("java"));
    let directory = env::temp_dir().join("vinglish-zero-java-frontend-v1");
    fs::create_dir_all(&directory).map_err(|error| SourceAdapterError::AdapterFailure {
        language: "Java",
        message: error.to_string(),
    })?;
    let helper = directory.join("VzJavaFrontend.java");
    let class = directory.join("VzJavaFrontend.class");
    if !class.is_file() {
        fs::write(&helper, HELPER).map_err(|error| SourceAdapterError::AdapterFailure {
            language: "Java",
            message: error.to_string(),
        })?;
        let output = Command::new(&javac)
            .arg("-d")
            .arg(&directory)
            .arg(&helper)
            .output()
            .map_err(|error| frontend_launch("Java", &javac, error))?;
        if !output.status.success() {
            return Err(frontend_failed("Java", output));
        }
    }
    let output = Command::new(&java)
        .arg("-cp")
        .arg(&directory)
        .arg("VzJavaFrontend")
        .arg(source)
        .output()
        .map_err(|error| frontend_launch("Java", &java, error))?;
    if !output.status.success() {
        return Err(frontend_failed("Java", output));
    }
    String::from_utf8(output.stdout).map_err(|_| SourceAdapterError::AdapterFailure {
        language: "Java",
        message: "Java frontend emitted non-UTF-8 output".to_owned(),
    })
}

fn frontend_launch(
    language: &'static str,
    executable: &OsString,
    error: std::io::Error,
) -> SourceAdapterError {
    if error.kind() == std::io::ErrorKind::NotFound {
        SourceAdapterError::CompilerNotFound {
            language,
            executable: executable.to_string_lossy().into_owned(),
        }
    } else {
        SourceAdapterError::AdapterFailure {
            language,
            message: format!("cannot launch '{}': {error}", executable.to_string_lossy()),
        }
    }
}
fn frontend_failed(language: &'static str, output: std::process::Output) -> SourceAdapterError {
    SourceAdapterError::CompilerFailed {
        language,
        exit_code: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

fn lower(source: &Path, root: &Value) -> Result<SemanticGraph, String> {
    let mut graph = SemanticGraph::new();
    let mut ids = NodeIdAllocator::new();
    let mut functions = Vec::new();
    for method in root
        .get("methods")
        .and_then(Value::as_array)
        .ok_or("missing methods")?
    {
        functions.push(function(&mut graph, &mut ids, method)?);
    }
    let module = ids.next();
    let span = span();
    graph.insert(SemanticNode::Module {
        id: module,
        name: source.display().to_string(),
        language: LanguageTag::Custom("java".to_owned()),
        children: functions,
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Program {
        id: NodeId::ROOT,
        modules: vec![module],
        span,
        metadata: Metadata::default(),
    });
    Ok(graph)
}
fn function(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, v: &Value) -> Result<NodeId, String> {
    let body = block(g, ids, v.get("body").ok_or("method body")?)?;
    let params = v
        .get("parameters")
        .and_then(Value::as_array)
        .ok_or("parameters")?
        .iter()
        .map(|p| {
            Ok(Param {
                name: strv(p, "name")?,
                type_concept: Some(ty(strv(p, "type")?)),
                mutability: Mutability::Untracked,
                has_default: false,
                span: span(),
            })
        })
        .collect::<Result<_, String>>()?;
    let id = ids.next();
    g.insert(SemanticNode::Function {
        id,
        name: strv(v, "name")?,
        params,
        return_type: Some(ty(strv(v, "return_type")?)),
        body,
        visibility: Visibility::Untracked,
        is_async: false,
        is_foreign: false,
        generic_params: Vec::new(),
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn block(
    g: &mut SemanticGraph,
    ids: &mut NodeIdAllocator,
    v: &Value,
) -> Result<Vec<NodeId>, String> {
    v.as_array()
        .ok_or("expected statement array")?
        .iter()
        .map(|x| stmt(g, ids, x))
        .collect()
}
fn stmt(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, v: &Value) -> Result<NodeId, String> {
    match kind(v)? {
        "variable" => {
            let init = expr(g, ids, v.get("initializer").ok_or("initializer")?)?;
            let id = ids.next();
            g.insert(SemanticNode::Variable {
                id,
                name: strv(v, "name")?,
                type_concept: Some(ty(strv(v, "type")?)),
                mutability: Mutability::Mutable,
                initializer_id: Some(init),
                scope: Scope::Local,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "assignment" | "mutation" => assignment(g, ids, v),
        "call" => expr(g, ids, v),
        "return" => {
            let value = Some(expr(g, ids, v.get("value").ok_or("return")?)?);
            let id = ids.next();
            g.insert(SemanticNode::Return {
                id,
                value,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "if" => {
            let c = expr(g, ids, v.get("condition").ok_or("condition")?)?;
            let then_body = block(g, ids, v.get("then").ok_or("then")?)?;
            let e = block(g, ids, v.get("else").ok_or("else")?)?;
            let id = ids.next();
            g.insert(SemanticNode::Conditional {
                id,
                condition: c,
                then_body,
                else_body: if e.is_empty() { None } else { Some(e) },
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "while" => {
            let c = expr(g, ids, v.get("condition").ok_or("condition")?)?;
            let b = block(g, ids, v.get("body").ok_or("body")?)?;
            loop_node(g, ids, c, b)
        }
        "for" => {
            let mut body = block(g, ids, v.get("init").ok_or("init")?)?;
            body.extend(block(g, ids, v.get("body").ok_or("body")?)?);
            body.extend(block(g, ids, v.get("update").ok_or("update")?)?);
            let c = expr(g, ids, v.get("condition").ok_or("condition")?)?;
            loop_node(g, ids, c, body)
        }
        "foreach" => {
            let iter = expr(g, ids, v.get("iterable").ok_or("iterable")?)?;
            let body = block(g, ids, v.get("body").ok_or("body")?)?;
            let id = ids.next();
            g.insert(SemanticNode::Loop {
                id,
                kind: LoopKind::ForEach {
                    variable: strv(v, "variable")?,
                    iterable: iter,
                },
                body,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        _ => unsupported(g, ids),
    }
}
fn loop_node(
    g: &mut SemanticGraph,
    ids: &mut NodeIdAllocator,
    c: NodeId,
    body: Vec<NodeId>,
) -> Result<NodeId, String> {
    let id = ids.next();
    g.insert(SemanticNode::Loop {
        id,
        kind: LoopKind::While { condition: c },
        body,
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn assignment(
    g: &mut SemanticGraph,
    ids: &mut NodeIdAllocator,
    v: &Value,
) -> Result<NodeId, String> {
    let target = expr(g, ids, v.get("target").ok_or("target")?)?;
    let value = expr(g, ids, v.get("value").ok_or("value")?)?;
    let op = if kind(v)? == "mutation" {
        Some(op(strv(v, "op")?))
    } else {
        None
    };
    let id = ids.next();
    g.insert(SemanticNode::Assignment {
        id,
        target,
        value,
        op,
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn expr(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, v: &Value) -> Result<NodeId, String> {
    match kind(v)? {
        "identifier" => {
            let id = ids.next();
            g.insert(SemanticNode::Identifier {
                id,
                name: strv(v, "name")?,
                resolved_to: None,
                type_concept: None,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "literal" => {
            let s = strv(v, "value")?;
            let k = strv(v, "literal_kind")?;
            let (value, t) = match k.as_str() {
                "boolean" => (LiteralValue::Boolean(s == "true"), TypeConcept::Boolean),
                "number" => {
                    if s.contains('.') {
                        (
                            LiteralValue::Float(s.parse().unwrap_or(0.0)),
                            TypeConcept::FloatingPoint { bits: None },
                        )
                    } else {
                        (
                            LiteralValue::Integer(s.parse().unwrap_or(0)),
                            TypeConcept::Integer {
                                bits: None,
                                signed: true,
                            },
                        )
                    }
                }
                "text" => (LiteralValue::Text(s), TypeConcept::Text),
                _ => (LiteralValue::Null, TypeConcept::Unit),
            };
            let id = ids.next();
            g.insert(SemanticNode::Literal {
                id,
                value,
                type_concept: t,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "binary" => {
            let l = expr(g, ids, v.get("left").ok_or("left")?)?;
            let r = expr(g, ids, v.get("right").ok_or("right")?)?;
            let id = ids.next();
            g.insert(SemanticNode::BinaryOp {
                id,
                op: op(strv(v, "op")?),
                left: l,
                right: r,
                result_type: None,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "unary" => {
            let a = expr(g, ids, v.get("operand").ok_or("operand")?)?;
            let id = ids.next();
            g.insert(SemanticNode::UnaryOp {
                id,
                op: if strv(v, "op")? == "not" {
                    SemanticUnOp::Not
                } else {
                    SemanticUnOp::Negate
                },
                operand: a,
                result_type: None,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "call" => {
            let c = expr(g, ids, v.get("callee").ok_or("callee")?)?;
            let arguments = v
                .get("arguments")
                .and_then(Value::as_array)
                .ok_or("arguments")?
                .iter()
                .map(|a| {
                    Ok(Argument {
                        label: None,
                        value: expr(g, ids, a)?,
                    })
                })
                .collect::<Result<_, String>>()?;
            let id = ids.next();
            g.insert(SemanticNode::Call {
                id,
                callee: c,
                arguments,
                return_type: None,
                is_async_call: false,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        "assignment" | "mutation" => assignment(g, ids, v),
        "collection" => {
            let e = v
                .get("elements")
                .and_then(Value::as_array)
                .ok_or("elements")?
                .iter()
                .map(|a| expr(g, ids, a))
                .collect::<Result<_, _>>()?;
            let id = ids.next();
            g.insert(SemanticNode::Collection {
                id,
                kind: CollectionKind::List,
                elements: e,
                element_type: None,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        _ => unsupported(g, ids),
    }
}
fn unsupported(g: &mut SemanticGraph, ids: &mut NodeIdAllocator) -> Result<NodeId, String> {
    let id = ids.next();
    g.insert(SemanticNode::Extension {
        id,
        tag: "java::unsupported".to_owned(),
        payload: vz_semantic_ir::metadata::SemanticValue::Null,
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn kind(v: &Value) -> Result<&str, String> {
    v.get("kind")
        .and_then(Value::as_str)
        .ok_or("missing kind".to_owned())
}
fn strv(v: &Value, k: &str) -> Result<String, String> {
    v.get(k)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing {k}"))
}
fn span() -> SemanticSpan {
    SemanticSpan::new(
        SourceId::UNKNOWN,
        0,
        0,
        LanguageTag::Custom("java".to_owned()),
    )
}
fn ty(s: String) -> TypeConcept {
    if s.contains("boolean") {
        TypeConcept::Boolean
    } else if s.contains("double") || s.contains("float") {
        TypeConcept::FloatingPoint { bits: None }
    } else if s.contains("int") || s.contains("long") || s.contains("short") || s.contains("byte") {
        TypeConcept::Integer {
            bits: None,
            signed: true,
        }
    } else if s.contains("String") {
        TypeConcept::Text
    } else {
        TypeConcept::Unknown
    }
}
fn op(s: String) -> SemanticOp {
    match s.as_str() {
        "add" => SemanticOp::Add,
        "subtract" => SemanticOp::Sub,
        "multiply" => SemanticOp::Mul,
        "divide" => SemanticOp::Div,
        "remainder" => SemanticOp::Rem,
        "equal" => SemanticOp::Eq,
        "not_equal" => SemanticOp::NotEq,
        "less_than" => SemanticOp::Lt,
        "greater_than" => SemanticOp::Gt,
        "less_equal" => SemanticOp::LtEq,
        "greater_equal" => SemanticOp::GtEq,
        "and" => SemanticOp::And,
        "or" => SemanticOp::Or,
        _ => SemanticOp::Eq,
    }
}
