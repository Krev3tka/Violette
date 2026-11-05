use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Ty {
    Int {
        bits: u8,
        signed: bool,
    },
    Sized {
        signed: bool,
    },
    Float {
        bits: u8,
    },
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
    Char,
    Array(Box<Ty>),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int { bits, signed } => {
                let mut res = String::from("Int");

                if !*signed {
                    res = String::from("UInt");
                }

                match bits {
                    8 => res.push('8'),
                    16 => res.push_str("16"),
                    32 => res.push_str("32"),
                    64 => res.push_str("64"),
                    128 => res.push_str("128"),
                    _ => unreachable!(),
                };

                write!(f, "{res}")
            }
            Ty::Sized { signed } => {
                write!(f, "{}Int", if !*signed { "U" } else { "" })
            }
            Ty::Float { bits } => {
                write!(f, "Float{bits}")
            }
            Ty::Bool => write!(f, "Bool"),
            Ty::String => write!(f, "String"),
            Ty::Struct(name) => write!(f, "{}", name),
            Ty::Unit => write!(f, "Void"),
            Ty::Ref {
                inner,
                is_mut: true,
            } => write!(f, "var &{}", inner),
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

                write!(f, "func({}) -> {}", p, ret)
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
