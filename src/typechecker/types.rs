use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Ty {
    Int,
    Float,
    Bool,
    String,
    #[default]
    Unit,
    Struct(String),
    Union(Vec<Ty>),
    Generic {
        name: String,
        param: Box<Ty>,
    },
    Fn {
        params: Vec<Ty>,
        ret: Box<Ty>,
    },
    Infer,
    Error,
    Ref {
        inner: Box<Ty>,
        is_mut: bool,
    },
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "int"),
            Ty::Float => write!(f, "float"),
            Ty::Bool => write!(f, "bool"),
            Ty::String => write!(f, "string"),
            Ty::Struct(name) => write!(f, "{}", name),
            Ty::Unit => write!(f, "void"),
            Ty::Ref {
                inner,
                is_mut: true,
            } => write!(f, "&var {}", inner),
            Ty::Ref {
                inner,
                is_mut: false,
            } => write!(f, "&{}", inner),
            Ty::Fn { params, ret } => {
                let p = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "func({}) [{}]", p, ret)
            }
            Ty::Union(variants) => {
                let v = variants
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");

                write!(f, "{}", v)
            }
            Ty::Infer => write!(f, "<infer>"),
            Ty::Error => write!(f, "<error>"),
            _ => write!(f, "{:?}", self),
        }
    }
}
