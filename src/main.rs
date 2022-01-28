use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Display, Debug},
    rc::Rc,
};

fn main() {
    let a1 = Var::new(34);
    let a2 = Var::new(35);
    let stmt = Statement::new(Operation::Add, [a1, a2]);
    let res = stmt.resolve().unwrap();
    Statement::new(
        Operation::Print,
        vec![res],
    )
    .resolve()
    .unwrap();
    let a1 = Var::new("Nice. ( ͡° ͜ʖ ͡°)");
    Statement::new(
        Operation::Print,
        vec![a1],
    )
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

pub trait Callable: Debug {
    // TODO(#5): Decide whether to keep the return type of Callable::call as a trait object or an
    // associated type
    fn call(&self, args: &Vec<Var>) -> Result<Var, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Print,
}

impl Callable for Operation {
    fn call(&self, args: &Vec<Var>) -> Result<Var, Box<dyn std::error::Error>> {
        match self {
            Operation::Add => {
                let mut sum = 0;
                for a in args {
                    if let LispType::Integer(i) = *a.get() {
                        sum += i;
                    } else {
                        // TODO(#4): Better error reporting in Statement::resolve with incorrect types
                        return Err(TypeError::new("Cannot add a non-integer type to an integer!"));
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Subtract => {
                let mut sum = 0;
                for a in args {
                    if let LispType::Integer(i) = *a.get() {
                        sum -= i;
                    } else {
                        return Err(TypeError::new("Cannot subtract a non-integer type from an integer!"));
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Print => {
                if args.len() != 1 {
                    return Err(TypeError::new("Print intrinsic requires only one argument!"));
                } else {
                    println!("{}", args[0]);
                    Ok(Var::new(0))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    args: Vec<Var>,
    op: Box<dyn Callable + 'static>,
}

#[derive(Debug)]
pub struct TypeError {
    msg: String,
    // TODOO(#3): Give location of invalid syntax
    // This will make it *soooo* much easier to debug code written in sul
}

impl TypeError {
    pub fn new<T: ToString>(msg: T) -> Box<Self> {
        Box::new(TypeError { msg: msg.to_string() })
    }
}

impl std::error::Error for TypeError {}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}


impl Statement {
    pub fn resolve(&self) -> Result<Var, Box<dyn std::error::Error>> {
        self.op.call(&self.args)
    }
    pub fn new<Op: Callable + 'static, AL: Into<Vec<Var>>>(o: Op, args: AL) -> Statement {
        let o = Box::new(o);
        let args = args.into();
        Statement {
            op: o,
            args,
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
