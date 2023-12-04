use std::{borrow::Borrow, collections::HashSet, hash::Hash};

use indexmap::IndexSet;

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(String),
    LocalClass(String),
    ExternalClass { module: String, name: String },
    Any,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(String),
    Str(String),
    None,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub value: Option<Value>,
    pub typ: Option<Type>,
    pub doc: Option<String>,
    pub constant: bool,
}

#[derive(Debug)]
pub struct Enumeration {
    pub name: String,
    pub values: Vec<Var>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Param {
    /// the classic
    Named {
        name: String,
        typ: Type,
        doc: Option<String>,
        optional: bool,
    },
    /// `*args`. these ones cannot be optional
    Variadic {
        name: String,
        typ: Type,
        doc: Option<String>,
    },
    /// in python would be `self`. not called `Self` 'cuz that's a keyword
    Instance,
    /// the `*` that makes the following parameters keyword only
    Star,
}

#[derive(Debug, Clone)]
pub enum FunctionKind {
    Static,
    Method,
    StaticMethod,
    Virtual,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Param>,
    pub return_type: Type,
    pub kind: FunctionKind,
    pub return_doc: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub bases: Vec<Type>,
    pub fields: Vec<Var>,
    pub methods: IndexSet<Function>,
    pub constructor: Function,
    pub doc: Option<String>,
}

/// Contains all the declarations inside a `<namespace />`
pub struct Namespace {
    pub name: String,
    pub imports: HashSet<String>,
    pub constants: Vec<Var>,
    pub enums: Vec<Enumeration>,
    pub functions: Vec<Function>,
    pub classes: IndexSet<Class>,
}

impl Function {
    // pub fn build(name: &str) -> Self {
    //     Self {
    //         name: name.to_string(),
    //         parameters: Vec::new(),
    //         return_type: Type::Any,
    //         kind: FunctionKind::Static,
    //         return_doc: None,
    //         doc: None,
    //     }
    // }
    // pub fn kind(mut self, kind: FunctionKind) -> Self {
    //     self.kind = kind;
    //     self
    // }
    pub fn clear_parameters(mut self) -> Self {
        self.parameters.clear();
        self
    }
    pub fn add_param(mut self, param: Param) -> Self {
        self.parameters.push(param);
        self
    }
    pub fn add_named_param<'a>(
        self,
        name: &str,
        typ: Type,
        optional: bool,
        doc: impl Into<Option<&'a str>>,
    ) -> Self {
        let doc: Option<&str> = doc.into();
        self.add_param(Param::Named {
            name: name.into(),
            typ,
            optional,
            doc: doc.map(|s| String::from(s)),
        })
    }
    pub fn add_self_param(self) -> Self {
        self.add_param(Param::Instance)
    }
}

macro_rules! index_by {
    ($struct:ident::$field:ident: &$type:ident) => {
        impl Hash for $struct {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.$field.hash(state);
                state.finish();
            }
        }

        impl PartialEq for $struct {
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl Eq for $struct {}

        impl Borrow<$type> for $struct {
            fn borrow(&self) -> &$type {
                &self.$field
            }
        }
    };
}

index_by!(Class::name: &str);
index_by!(Function::name: &str);
index_by!(Namespace::name: &str);

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(s) => write!(f, "{}", s),
            Self::LocalClass(c) => write!(f, "{}", c),
            Self::ExternalClass { module, name } => write!(f, "{}.{}", module, name),
            Self::Any => write!(f, "typing.Any"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(typ) = &self.typ {
            if self.constant {
                write!(f, ": typing.Final[{}]", typ)?;
            } else {
                write!(f, ": {}", typ)?;
            }
        }
        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }

        Ok(())
    }
}
