//! C adapter backed exclusively by Clang's JSON AST dump.
use serde_json::Value;
use std::{env, ffi::OsString, path::Path, process::Command};
use vz_adapters::{
    AdapterCapabilities, AdapterManifest, SemanticAdapter, SourceAdapter, SourceAdapterError,
};
use vz_common::{
    LanguageTag, Mutability, NodeId, NodeIdAllocator, SemanticSpan, SourceId, Visibility,
};
use vz_semantic_ir::{
    node::{LiteralValue, Param, Scope},
    LoopKind, Metadata, SemanticGraph, SemanticNode, SemanticOp, TypeConcept,
};

#[derive(Debug, Default)]
pub struct CAdapter;
impl SemanticAdapter for CAdapter {
    fn manifest(&self) -> AdapterManifest {
        AdapterManifest::new("clang-ast", LanguageTag::Custom("c".into()), "1")
    }
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::new(true, false, true)
    }
}
impl SourceAdapter for CAdapter {
    fn extensions(&self) -> &[&str] {
        &["c"]
    }
    fn language(&self) -> &'static str {
        "C"
    }
    fn semantic_graph(&self, p: &Path) -> Result<SemanticGraph, SourceAdapterError> {
        let ast = clang(p)?;
        lower(p, &ast).map_err(|message| SourceAdapterError::AdapterFailure {
            language: "C",
            message,
        })
    }
}
fn clang(p: &Path) -> Result<Value, SourceAdapterError> {
    let c = env::var_os("VZ_CLANG").unwrap_or_else(|| OsString::from("clang"));
    let o = Command::new(&c)
        .args([
            "-Xclang",
            "-ast-dump=json",
            "-fsyntax-only",
            "-Wno-everything",
        ])
        .arg(p)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SourceAdapterError::CompilerNotFound {
                    language: "C",
                    executable: c.to_string_lossy().into_owned(),
                }
            } else {
                SourceAdapterError::AdapterFailure {
                    language: "C",
                    message: e.to_string(),
                }
            }
        })?;
    if !o.status.success() {
        return Err(SourceAdapterError::CompilerFailed {
            language: "C",
            exit_code: o.status.code(),
            stderr: String::from_utf8_lossy(&o.stderr).trim().into(),
        });
    }
    serde_json::from_slice(&o.stdout).map_err(|e| SourceAdapterError::AdapterFailure {
        language: "C",
        message: format!("invalid Clang AST JSON: {e}"),
    })
}
fn lower(p: &Path, r: &Value) -> Result<SemanticGraph, String> {
    let mut g = SemanticGraph::new();
    let mut ids = NodeIdAllocator::new();
    let mut f = Vec::new();
    for n in inner(r) {
        if kind(n) == "FunctionDecl" {
            f.push(function(&mut g, &mut ids, n)?);
        }
    }
    let m = ids.next();
    g.insert(SemanticNode::Module {
        id: m,
        name: p.display().to_string(),
        language: lang(),
        children: f,
        span: span(),
        metadata: Metadata::default(),
    });
    g.insert(SemanticNode::Program {
        id: NodeId::ROOT,
        modules: vec![m],
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(g)
}
fn function(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, n: &Value) -> Result<NodeId, String> {
    let params = inner(n)
        .iter()
        .filter(|x| kind(x) == "ParmVarDecl")
        .map(|x| Param {
            name: name(x),
            type_concept: Some(ty(qtype(x))),
            mutability: Mutability::Untracked,
            has_default: false,
            span: span(),
        })
        .collect();
    let mut body = Vec::new();
    if let Some(b) = inner(n).iter().find(|x| kind(x) == "CompoundStmt") {
        walk(g, ids, b, &mut body)?
    }
    let id = ids.next();
    g.insert(SemanticNode::Function {
        id,
        name: name(n),
        params,
        return_type: Some(ty(qtype(n))),
        body,
        visibility: Visibility::Untracked,
        is_async: false,
        is_foreign: false,
        generic_params: vec![],
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn walk(
    g: &mut SemanticGraph,
    ids: &mut NodeIdAllocator,
    n: &Value,
    out: &mut Vec<NodeId>,
) -> Result<(), String> {
    match kind(n) {
        "DeclStmt" => {
            for d in inner(n).iter().filter(|x| kind(x) == "VarDecl") {
                let init = inner(d).first().map(|x| expr(g, ids, x)).transpose()?;
                let id = ids.next();
                g.insert(SemanticNode::Variable {
                    id,
                    name: name(d),
                    type_concept: Some(ty(qtype(d))),
                    mutability: Mutability::Mutable,
                    initializer_id: init,
                    scope: Scope::Local,
                    span: span(),
                    metadata: Metadata::default(),
                });
                out.push(id)
            }
        }
        "ReturnStmt" => {
            let v = inner(n).first().map(|x| expr(g, ids, x)).transpose()?;
            let id = ids.next();
            g.insert(SemanticNode::Return {
                id,
                value: v,
                span: span(),
                metadata: Metadata::default(),
            });
            out.push(id)
        }
        "IfStmt" => {
            let xs = inner(n);
            let c = expr(g, ids, xs.first().ok_or("if condition")?)?;
            let mut a = Vec::new();
            if let Some(x) = xs.get(1) {
                walk(g, ids, x, &mut a)?
            }
            let mut b = Vec::new();
            if let Some(x) = xs.get(2) {
                walk(g, ids, x, &mut b)?
            }
            let id = ids.next();
            g.insert(SemanticNode::Conditional {
                id,
                condition: c,
                then_body: a,
                else_body: if b.is_empty() { None } else { Some(b) },
                span: span(),
                metadata: Metadata::default(),
            });
            out.push(id)
        }
        "WhileStmt" | "ForStmt" => {
            let mut b = Vec::new();
            for x in inner(n) {
                if kind(x) == "CompoundStmt" {
                    walk(g, ids, x, &mut b)?
                }
            }
            let c = literal(g, ids, LiteralValue::Boolean(true), TypeConcept::Boolean);
            let id = ids.next();
            g.insert(SemanticNode::Loop {
                id,
                kind: LoopKind::While { condition: c },
                body: b,
                span: span(),
                metadata: Metadata::default(),
            });
            out.push(id)
        }
        "BinaryOperator" | "CompoundAssignOperator" => out.push(assign(g, ids, n)?),
        "CallExpr" => out.push(expr(g, ids, n)?),
        _ => {
            for x in inner(n) {
                walk(g, ids, x, out)?
            }
        }
    };
    Ok(())
}
fn assign(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, n: &Value) -> Result<NodeId, String> {
    let xs = inner(n);
    let a = expr(g, ids, xs.first().ok_or("target")?)?;
    let b = expr(g, ids, xs.get(1).ok_or("value")?)?;
    let code = n.get("opcode").and_then(Value::as_str).unwrap_or("=");
    let id = ids.next();
    g.insert(SemanticNode::Assignment {
        id,
        target: a,
        value: b,
        op: if code == "=" { None } else { Some(op(code)) },
        span: span(),
        metadata: Metadata::default(),
    });
    Ok(id)
}
fn expr(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, n: &Value) -> Result<NodeId, String> {
    match kind(n) {
        "ImplicitCastExpr" | "ParenExpr" => expr(g, ids, inner(n).first().ok_or("expression")?),
        "DeclRefExpr" => Ok(identifier(g, ids, name(n))),
        "IntegerLiteral" => Ok(literal(
            g,
            ids,
            LiteralValue::Integer(
                n.get("value")
                    .and_then(Value::as_str)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0),
            ),
            TypeConcept::Integer {
                bits: None,
                signed: true,
            },
        )),
        "BinaryOperator" => {
            let x = inner(n);
            let a = expr(g, ids, x.first().ok_or("left")?)?;
            let b = expr(g, ids, x.get(1).ok_or("right")?)?;
            let id = ids.next();
            g.insert(SemanticNode::BinaryOp {
                id,
                op: op(n.get("opcode").and_then(Value::as_str).unwrap_or("")),
                left: a,
                right: b,
                result_type: None,
                span: span(),
                metadata: Metadata::default(),
            });
            Ok(id)
        }
        _ => Ok(identifier(g, ids, "unsupported".into())),
    }
}
fn identifier(g: &mut SemanticGraph, ids: &mut NodeIdAllocator, name: String) -> NodeId {
    let id = ids.next();
    g.insert(SemanticNode::Identifier {
        id,
        name,
        resolved_to: None,
        type_concept: None,
        span: span(),
        metadata: Metadata::default(),
    });
    id
}
fn literal(
    g: &mut SemanticGraph,
    ids: &mut NodeIdAllocator,
    value: LiteralValue,
    type_concept: TypeConcept,
) -> NodeId {
    let id = ids.next();
    g.insert(SemanticNode::Literal {
        id,
        value,
        type_concept,
        span: span(),
        metadata: Metadata::default(),
    });
    id
}
fn inner(v: &Value) -> &[Value] {
    v.get("inner")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn kind(v: &Value) -> &str {
    v.get("kind").and_then(Value::as_str).unwrap_or("")
}
fn name(v: &Value) -> String {
    v.get("name")
        .and_then(Value::as_str)
        .or_else(|| {
            v.get("referencedDecl")
                .and_then(|x| x.get("name"))
                .and_then(Value::as_str)
        })
        .unwrap_or("unknown")
        .into()
}
fn qtype(v: &Value) -> &str {
    v.get("type")
        .and_then(|x| x.get("qualType"))
        .and_then(Value::as_str)
        .unwrap_or("int")
}
fn lang() -> LanguageTag {
    LanguageTag::Custom("c".into())
}
fn span() -> SemanticSpan {
    SemanticSpan::new(SourceId::UNKNOWN, 0, 0, lang())
}
fn ty(s: &str) -> TypeConcept {
    if s.contains("float") || s.contains("double") {
        TypeConcept::FloatingPoint { bits: None }
    } else {
        TypeConcept::Integer {
            bits: None,
            signed: true,
        }
    }
}
fn op(s: &str) -> SemanticOp {
    match s.trim_end_matches('=') {
        "+" => SemanticOp::Add,
        "-" => SemanticOp::Sub,
        "*" => SemanticOp::Mul,
        "/" => SemanticOp::Div,
        "%" => SemanticOp::Rem,
        "<" => SemanticOp::Lt,
        ">" => SemanticOp::Gt,
        _ => SemanticOp::Eq,
    }
}
