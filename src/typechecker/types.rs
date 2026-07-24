#[derive(Debug, Clone, PartialEq, Default)]
pub enum Ty {
    Int,
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
    Error,
}
