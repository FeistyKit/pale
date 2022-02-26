use crate::ast::{Statement, Var};
use crate::callable::Callable;
use std::fmt::{Debug, Display};

pub(crate) enum LispValue {
    Integer(isize),
    Str(String),
    Func(Box<dyn Callable>),
    Statement(Statement),
    #[allow(dead_code)]
    List(Vec<Var>),
    Floating(f64),
    Nil,
    //FIXME: Having a variable inside a lisptype is a hack that is required for the current implementation of lisp functions, but it's not good.
    Var(Var), // TODO(#2): Add custom newtypes.
}

impl Debug for LispValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(arg0) => f.debug_tuple("Integer").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Func(func) => f
                .debug_tuple("Func")
                .field(&func.maybe_debug_info().unwrap_or("<function>".into()))
                .finish(),
            Self::Statement(arg0) => f.debug_tuple("Statement").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Floating(arg0) => f.debug_tuple("Floating").field(arg0).finish(),
            Self::Nil => write!(f, "Nil"),
            Self::Var(v) => write!(f, "{:?}", v),
        }
    }
}

impl Clone for LispValue {
    fn clone(&self) -> Self {
        match self {
            Self::Integer(item) => Self::Integer(item.clone()),
            Self::Str(item) => Self::Str(item.clone()),
            Self::Func(_) => panic!("Tried to clone a function! If you see this, this is an internal error and you should report it at <https://github.com/FeistyKit/pale/issues/new>!"),
            Self::Statement(_) => panic!("Tried to clone a statement! If you see this, this is an internal error and you should report it at <https://github.com/FeistyKit/pale/issues/new>!"),
            Self::List(l) => Self::List(l.iter().map(Var::maybe_clone).collect()),
            Self::Floating(item) => Self::Floating(item.clone()),
            Self::Nil => Self::Nil,
            Self::Var(v) => Self::Var(v.maybe_clone())
        }
    }
}

const FLOATING_EQ_RANGE: f64 = 0.001; // If two floats are less than this far apart, they are considered equal

impl PartialEq for LispValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&LispValue::Integer(lhs), &LispValue::Integer(rhs)) => lhs == rhs,
            (LispValue::Str(lhs), LispValue::Str(rhs)) => lhs == rhs,
            (LispValue::Statement(lhs), LispValue::Statement(rhs)) => lhs == rhs,
            (LispValue::Func(_), LispValue::Func(_)) => false,
            (LispValue::Nil, LispValue::Nil) => true,
            (LispValue::Floating(lhs), LispValue::Floating(rhs)) => {
                (lhs - rhs).abs() < FLOATING_EQ_RANGE
            }
            (LispValue::List(lhs), LispValue::List(rhs)) => lhs == rhs,
            // TODOO(#10): Comparing floats and integers
            _ => false,
        }
    }
}

impl LispValue {
    pub(crate) fn unwrap_func(&self) -> &dyn Callable {
        match self {
            LispValue::Func(f) => f.as_ref(),
            _ => panic!("Expected to be LispType::Func but was actually {self}!"),
        }
    }

    pub(crate) fn is_func(&self) -> bool {
        match self {
            LispValue::Func(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_stmt(&self) -> bool {
        match self {
            LispValue::Statement(_) => true,
            _ => false,
        }
    }
}

impl Display for LispValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispValue::Integer(i) => write!(f, "{i}"),
            LispValue::Str(s) => write!(f, "{s}"),
            LispValue::Func(_) => write!(f, "<Function>"),
            LispValue::Statement(s) => match s.resolve() {
                Ok(s) => write!(f, "{s}"),
                Err(e) => write!(f, "{e}"),
            },
            LispValue::List(l) => {
                let mut t = String::new();
                for item in l {
                    t = format!("{t} {item}");
                }
                write!(f, "({t})")
            }
            LispValue::Floating(fl) => write!(f, "{fl}"),
            LispValue::Nil => write!(f, "nil"),
            LispValue::Var(v) => write!(f, "{v}"),
        }
    }
}

impl From<Box<dyn Callable>> for LispValue {
    fn from(other: Box<dyn Callable>) -> Self {
        LispValue::Func(other)
    }
}

impl From<isize> for LispValue {
    fn from(i: isize) -> Self {
        LispValue::Integer(i)
    }
}
impl From<String> for LispValue {
    fn from(i: String) -> Self {
        LispValue::Str(i)
    }
}
impl From<&str> for LispValue {
    fn from(i: &str) -> Self {
        LispValue::Str(i.to_string())
    }
}
impl<T: Callable + 'static> From<T> for LispValue {
    fn from(i: T) -> Self {
        LispValue::Func(Box::new(i))
    }
}
impl From<Statement> for LispValue {
    fn from(i: Statement) -> Self {
        LispValue::Statement(i)
    }
}
impl From<f64> for LispValue {
    fn from(i: f64) -> Self {
        LispValue::Floating(i)
    }
}
