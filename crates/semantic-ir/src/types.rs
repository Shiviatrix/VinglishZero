use vz_common::Mutability;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConcept {
    Integer {
        bits: Option<u8>,
        signed: bool,
    },
    FloatingPoint {
        bits: Option<u8>,
    },
    Boolean,
    Text,
    Unit,
    Collection(Box<TypeConcept>),
    Array {
        element: Box<TypeConcept>,
        size: Option<usize>,
    },
    Set(Box<TypeConcept>),
    Tuple(Vec<TypeConcept>),
    Map {
        key: Box<TypeConcept>,
        value: Box<TypeConcept>,
    },
    Optional(Box<TypeConcept>),
    Result {
        ok: Box<TypeConcept>,
        err: Box<TypeConcept>,
    },
    Reference {
        inner: Box<TypeConcept>,
        mutability: Mutability,
    },
    RawPointer(Box<TypeConcept>),
    Callable {
        params: Vec<TypeConcept>,
        return_type: Box<TypeConcept>,
        is_async: bool,
    },
    Named {
        name: String,
        type_args: Vec<TypeConcept>,
    },
    Unknown,
    Dynamic,
    Inferred,
    Extension {
        tag: String,
        payload: crate::metadata::SemanticValue,
    },
}

impl TypeConcept {
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::Integer { .. } | Self::FloatingPoint { .. } | Self::Boolean | Self::Unit
        )
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Integer {
                bits: Some(bits),
                signed: true,
            } => format!("signed {bits}-bit integer"),
            Self::Integer {
                bits: Some(bits),
                signed: false,
            } => format!("unsigned {bits}-bit integer"),
            Self::Integer { signed: true, .. } => "integer".to_owned(),
            Self::Integer { .. } => "unsigned integer".to_owned(),
            Self::FloatingPoint { bits: Some(bits) } => format!("{bits}-bit float"),
            Self::FloatingPoint { .. } => "floating-point".to_owned(),
            Self::Boolean => "boolean".to_owned(),
            Self::Text => "text".to_owned(),
            Self::Unit => "unit".to_owned(),
            Self::Collection(inner) => format!("collection of {}", inner.describe()),
            Self::Array { element, .. } => format!("array of {}", element.describe()),
            Self::Set(inner) => format!("set of {}", inner.describe()),
            Self::Tuple(elements) => format!(
                "({})",
                elements
                    .iter()
                    .map(Self::describe)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Map { key, value } => {
                format!("map from {} to {}", key.describe(), value.describe())
            }
            Self::Optional(inner) => format!("optional {}", inner.describe()),
            Self::Result { ok, err } => {
                format!("result of {} or {}", ok.describe(), err.describe())
            }
            Self::Reference { inner, .. } => format!("reference to {}", inner.describe()),
            Self::RawPointer(inner) => format!("pointer to {}", inner.describe()),
            Self::Callable { .. } => "function".to_owned(),
            Self::Named { name, .. } => name.clone(),
            Self::Unknown => "unknown".to_owned(),
            Self::Dynamic => "dynamic".to_owned(),
            Self::Inferred => "inferred".to_owned(),
            Self::Extension { tag, .. } => format!("<{tag}>"),
        }
    }
}
