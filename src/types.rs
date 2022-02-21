use crate::ast::{Statement, Var};
use crate::callable::Callable;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum LispType {
    Integer(isize),
    Str(String),
    Func(Box<dyn Callable>),
    Statement(Statement),
    #[allow(dead_code)]
    List(Vec<Var>),
    Floating(f64),
    Nil,
    // TODO(#2): Add custom newtypes.
}

impl Clone for LispType {
    fn clone(&self) -> Self {
        match self {
            Self::Integer(item) => Self::Integer(item.clone()),
            Self::Str(item) => Self::Str(item.clone()),
            Self::Func(_) => panic!("Tried to clone a function! If you see this, this is an internal error and you should report it at <https://github.com/FeistyKit/pale/issues/new>!"),
            Self::Statement(_) => panic!("Tried to clone a statement! If you see this, this is an internal error and you should report it at <https://github.com/FeistyKit/pale/issues/new>!"),
            Self::List(_) => panic!("Tried to clone a list! If you see this, this is an internal error and you should report it at <https://github.com/FeistyKit/pale/issues/new>!"),
            Self::Floating(item) => Self::Floating(item.clone()),
            Self::Nil => Self::Nil,
        }
    }
}

const FLOATING_EQ_RANGE: f64 = 0.001; // If two floats are less than this far apart, they are considered equal

impl PartialEq for LispType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&LispType::Integer(lhs), &LispType::Integer(rhs)) => lhs == rhs,
            (LispType::Str(lhs), LispType::Str(rhs)) => lhs == rhs,
            (LispType::Statement(lhs), LispType::Statement(rhs)) => lhs == rhs,
            (LispType::Func(_), LispType::Func(_)) => false,
            (LispType::Nil, LispType::Nil) => true,
            (LispType::Floating(lhs), LispType::Floating(rhs)) => {
                (lhs - rhs).abs() < FLOATING_EQ_RANGE
            }
            (LispType::List(lhs), LispType::List(rhs)) => lhs == rhs,
            // TODOO(#10): Comparing floats and integers
            _ => false,
        }
    }
}

impl LispType {
    pub(crate) fn unwrap_func(&self) -> &dyn Callable {
        match self {
            LispType::Func(f) => f.as_ref(),
            _ => panic!("Expected to be LispType::Func but was actually {self}!"),
        }
    }

    pub(crate) fn is_func(&self) -> bool {
        match self {
            LispType::Func(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_stmt(&self) -> bool {
        match self {
            LispType::Statement(_) => true,
            _ => false,
        }
    }
}

impl Display for LispType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispType::Integer(i) => write!(f, "{i}"),
            LispType::Str(s) => write!(f, "{s}"),
            LispType::Func(_) => write!(f, "<Function>"),
            LispType::Statement(s) => match s.resolve() {
                Ok(s) => write!(f, "{s}"),
                Err(e) => write!(f, "{e}"),
            },
            LispType::List(l) => {
                let mut t = String::new();
                for item in l {
                    t = format!("{t} {item}");
                }
                write!(f, "({t})")
            }
            LispType::Floating(fl) => write!(f, "{fl}"),
            LispType::Nil => write!(f, "nil"),
        }
    }
}

impl From<isize> for LispType {
    fn from(i: isize) -> Self {
        LispType::Integer(i)
    }
}
impl From<String> for LispType {
    fn from(i: String) -> Self {
        LispType::Str(i)
    }
}
impl From<&str> for LispType {
    fn from(i: &str) -> Self {
        LispType::Str(i.to_string())
    }
}
impl<T: Callable + 'static> From<T> for LispType {
    fn from(i: T) -> Self {
        LispType::Func(Box::new(i))
    }
}
impl From<Statement> for LispType {
    fn from(i: Statement) -> Self {
        LispType::Statement(i)
    }
}
impl From<f64> for LispType {
    fn from(i: f64) -> Self {
        LispType::Floating(i)
    }
}
