use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    rc::Rc,
};

fn main() {
    let a1 = Var::new(34);
    let a2 = Var::new(35);
    let stmt = Statement {
        op: Operation::Add,
        args: vec![a1, a2],
    };
    let res = stmt.resolve().unwrap();
    Statement {
        op: Operation::Print,
        args: vec![res],
    }
    .resolve()
    .unwrap();
    let a1 = Var::new("Nice. ( ͡° ͜ʖ ͡°)");
    Statement {
        op: Operation::Print,
        args: vec![a1],
    }
    .resolve()
    .unwrap();
}

#[derive(Debug, Clone)]
pub enum LispType {
    // TODOOOO(#1): Add more types, like lists and floating points;
    Integer(isize),
    Str(String),
    // TODO(#2): Add custom newtypes.
}

#[allow(dead_code)]
impl LispType {
    fn unwrap_string(self) -> String {
        if let LispType::Str(s) = self {
            s
        } else {
            panic!("Could not unwrap non-string value: {:?}. If you're seeing this, this is an internal error and you should report it at https://github.com/FeistyKit/sul/issues/new", self);
        }
    }
    fn unwrap_int(self) -> isize {
        if let LispType::Integer(i) = self {
            i
        } else {
            panic!("Could not unwrap non-integer value: {:?}. If you're seeing this, this is an internal error and you should report it at https://github.com/FeistyKit/sul/issues/new", self);
        }
    }
}

impl Display for LispType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispType::Integer(i) => write!(f, "{i}"),
            LispType::Str(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Print,
}

#[derive(Debug)]
pub struct Statement {
    args: Vec<Var>,
    op: Operation,
}

#[derive(Debug)]
pub struct SyntaxError {
    msg: String,
    // TODOO(#3): Give location of invalid syntax
    // This will make it *soooo* much easier to debug code written in sul
}

impl std::error::Error for SyntaxError {}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Statement {
    pub fn resolve(&self) -> Result<Var, SyntaxError> {
        match self.op {
            Operation::Add => {
                let mut sum = 0;
                for a in &self.args {
                    if let LispType::Integer(i) = *a.get() {
                        sum += i;
                    } else {
                        // TODO(#4): Better error reporting in Statement::resolve with incorrect types
                        return Err(SyntaxError {
                            msg: "Cannot add a non-integer type to an integer!".into(),
                        });
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Subtract => {
                let mut sum = 0;
                for a in &self.args {
                    if let LispType::Integer(i) = *a.get() {
                        sum -= i;
                    } else {
                        return Err(SyntaxError {
                            msg: "Cannot subtract a non-integer type from an integer!".into(),
                        });
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Print => {
                if self.args.len() != 1 {
                    Err(SyntaxError {
                        msg: "Print intrinsic requires exactly one argument!".into(),
                    })
                } else {
                    println!("{}", self.args[0]);
                    Ok(Var::new(0))
                }
            }
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

#[derive(Debug)]
pub struct Var {
    dat: Rc<RefCell<LispType>>,
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.get())
    }
}

#[allow(dead_code)]
impl Var {
    fn new<T: Into<LispType>>(i: T) -> Var {
        Var {
            dat: Rc::new(RefCell::new(i.into())),
        }
    }
    fn new_ref(&self) -> Var {
        Var {
            dat: Rc::clone(&self.dat),
        }
    }
    fn get(&self) -> Ref<LispType> {
        self.dat.borrow()
    }
    fn get_mut(&self) -> RefMut<LispType> {
        self.dat.borrow_mut()
    }
}

impl std::clone::Clone for Var {
    fn clone(&self) -> Self {
        Var::new((*self.dat.borrow()).clone())
    }
}
