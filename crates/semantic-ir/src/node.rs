use vz_common::{LanguageTag, Mutability, NodeId, SemanticSpan, Visibility};

use crate::{metadata::SemanticValue, Metadata, TypeConcept};

#[derive(Debug, Clone)]
pub enum SemanticNode {
    Program {
        id: NodeId,
        modules: Vec<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Module {
        id: NodeId,
        name: String,
        language: LanguageTag,
        children: Vec<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Namespace {
        id: NodeId,
        name: String,
        children: Vec<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Function {
        id: NodeId,
        name: String,
        params: Vec<Param>,
        return_type: Option<TypeConcept>,
        body: Vec<NodeId>,
        visibility: Visibility,
        is_async: bool,
        is_foreign: bool,
        generic_params: Vec<String>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Lambda {
        id: NodeId,
        params: Vec<Param>,
        return_type: Option<TypeConcept>,
        body: Vec<NodeId>,
        captured: Vec<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Variable {
        id: NodeId,
        name: String,
        type_concept: Option<TypeConcept>,
        mutability: Mutability,
        initializer_id: Option<NodeId>,
        scope: Scope,
        span: SemanticSpan,
        metadata: Metadata,
    },
    TypeDefinition {
        id: NodeId,
        name: String,
        kind: TypeDefKind,
        fields: Vec<FieldDef>,
        generic_params: Vec<String>,
        visibility: Visibility,
        span: SemanticSpan,
        metadata: Metadata,
    },
    EnumDefinition {
        id: NodeId,
        name: String,
        variants: Vec<EnumVariant>,
        generic_params: Vec<String>,
        visibility: Visibility,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Literal {
        id: NodeId,
        value: LiteralValue,
        type_concept: TypeConcept,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Identifier {
        id: NodeId,
        name: String,
        resolved_to: Option<NodeId>,
        type_concept: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    BinaryOp {
        id: NodeId,
        op: SemanticOp,
        left: NodeId,
        right: NodeId,
        result_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    UnaryOp {
        id: NodeId,
        op: SemanticUnOp,
        operand: NodeId,
        result_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Call {
        id: NodeId,
        callee: NodeId,
        arguments: Vec<Argument>,
        return_type: Option<TypeConcept>,
        is_async_call: bool,
        span: SemanticSpan,
        metadata: Metadata,
    },
    FieldAccess {
        id: NodeId,
        object: NodeId,
        field_name: String,
        result_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    IndexAccess {
        id: NodeId,
        object: NodeId,
        index: NodeId,
        result_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Collection {
        id: NodeId,
        kind: CollectionKind,
        elements: Vec<NodeId>,
        element_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    MapLiteral {
        id: NodeId,
        entries: Vec<(NodeId, NodeId)>,
        key_type: Option<TypeConcept>,
        value_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    StructLiteral {
        id: NodeId,
        type_name: String,
        fields: Vec<(String, NodeId)>,
        type_concept: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Reference {
        id: NodeId,
        target: NodeId,
        mutability: Mutability,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Dereference {
        id: NodeId,
        target: NodeId,
        result_type: Option<TypeConcept>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Assignment {
        id: NodeId,
        target: NodeId,
        value: NodeId,
        op: Option<SemanticOp>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Return {
        id: NodeId,
        value: Option<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Conditional {
        id: NodeId,
        condition: NodeId,
        then_body: Vec<NodeId>,
        else_body: Option<Vec<NodeId>>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Loop {
        id: NodeId,
        kind: LoopKind,
        body: Vec<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Break {
        id: NodeId,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Continue {
        id: NodeId,
        span: SemanticSpan,
        metadata: Metadata,
    },
    PatternMatch {
        id: NodeId,
        subject: NodeId,
        arms: Vec<MatchArm>,
        otherwise: Option<Vec<NodeId>>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Allocation {
        id: NodeId,
        kind: AllocKind,
        initializer: Option<NodeId>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Drop {
        id: NodeId,
        target: NodeId,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Async {
        id: NodeId,
        kind: AsyncKind,
        inner: NodeId,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Transaction {
        id: NodeId,
        body: Vec<NodeId>,
        on_commit: Option<Vec<NodeId>>,
        on_rollback: Option<Vec<NodeId>>,
        span: SemanticSpan,
        metadata: Metadata,
    },
    ApiUsage {
        id: NodeId,
        api_ref: ApiRef,
        call_node: NodeId,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Import {
        id: NodeId,
        path: Vec<String>,
        alias: Option<String>,
        kind: ImportKind,
        span: SemanticSpan,
        metadata: Metadata,
    },
    Extension {
        id: NodeId,
        tag: String,
        payload: SemanticValue,
        span: SemanticSpan,
        metadata: Metadata,
    },
}

impl SemanticNode {
    pub fn id(&self) -> NodeId {
        match self {
            Self::Program { id, .. }
            | Self::Module { id, .. }
            | Self::Namespace { id, .. }
            | Self::Function { id, .. }
            | Self::Lambda { id, .. }
            | Self::Variable { id, .. }
            | Self::TypeDefinition { id, .. }
            | Self::EnumDefinition { id, .. }
            | Self::Literal { id, .. }
            | Self::Identifier { id, .. }
            | Self::BinaryOp { id, .. }
            | Self::UnaryOp { id, .. }
            | Self::Call { id, .. }
            | Self::FieldAccess { id, .. }
            | Self::IndexAccess { id, .. }
            | Self::Collection { id, .. }
            | Self::MapLiteral { id, .. }
            | Self::StructLiteral { id, .. }
            | Self::Reference { id, .. }
            | Self::Dereference { id, .. }
            | Self::Assignment { id, .. }
            | Self::Return { id, .. }
            | Self::Conditional { id, .. }
            | Self::Loop { id, .. }
            | Self::Break { id, .. }
            | Self::Continue { id, .. }
            | Self::PatternMatch { id, .. }
            | Self::Allocation { id, .. }
            | Self::Drop { id, .. }
            | Self::Async { id, .. }
            | Self::Transaction { id, .. }
            | Self::ApiUsage { id, .. }
            | Self::Import { id, .. }
            | Self::Extension { id, .. } => *id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_concept: Option<TypeConcept>,
    pub mutability: Mutability,
    pub has_default: bool,
    pub span: SemanticSpan,
}
#[derive(Debug, Clone)]
pub struct Argument {
    pub label: Option<String>,
    pub value: NodeId,
}
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_concept: TypeConcept,
    pub mutability: Mutability,
    pub visibility: Visibility,
    pub index: usize,
    pub span: SemanticSpan,
}
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<TypeConcept>,
    pub span: SemanticSpan,
}
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<NodeId>,
    pub body: Vec<NodeId>,
    pub span: SemanticSpan,
}
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Literal(LiteralValue),
    Variable(String),
    EnumVariant {
        name: String,
        fields: Vec<Pattern>,
    },
    Struct {
        type_name: String,
        fields: Vec<(String, Pattern)>,
    },
    Or(Vec<Pattern>),
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
}
#[derive(Debug, Clone)]
pub enum Scope {
    Local,
    Parameter,
    Module,
    Captured,
}
#[derive(Debug, Clone)]
pub enum LoopKind {
    While { condition: NodeId },
    ForEach { variable: String, iterable: NodeId },
    Count { times: NodeId },
    Infinite,
}
#[derive(Debug, Clone)]
pub enum AllocKind {
    Heap,
    Stack,
    Arena,
    Unknown,
}
#[derive(Debug, Clone)]
pub enum AsyncKind {
    Block,
    Await,
    Spawn,
}
#[derive(Debug, Clone)]
pub enum CollectionKind {
    List,
    Array,
    Set,
    Tuple,
    Other(String),
}
#[derive(Debug, Clone)]
pub enum ImportKind {
    Glob,
    Named,
    Aliased,
}
#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct,
    Class,
    Record,
    Interface,
    Trait,
    TypeAlias,
    Other(String),
}
#[derive(Debug, Clone)]
pub struct ApiRef {
    pub path: Vec<String>,
    pub operation: String,
    pub original_name: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Concat,
    RangeExclusive,
    RangeInclusive,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticUnOp {
    Negate,
    Not,
    BitNot,
    Spread,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Text(String),
    Null,
    None,
}
