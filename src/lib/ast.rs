use crate::callable::{Callable, IntrinsicOp};
use crate::tokens::{Token, TokenType};
use crate::types::LispType;
use crate::Location;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    fmt::Display,
    rc::Rc,
};

#[derive(Debug, PartialEq)]
pub struct Var {
    pub(crate) dat: Rc<RefCell<LispType>>,
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.get())
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Statement {
    pub(crate) args: Vec<Var>,
    pub(crate) op: Var, // The inner value must be callable, so this won't panic (I hope)
    pub(crate) res: RefCell<Option<Var>>,
    pub(crate) loc: Location,
}

#[derive(Debug)]
pub struct TypeError {
    pub(crate) msg: String,
    // TODOO(#3): Give location of invalid syntax
    // This will make it *soooo* much easier to debug code written in sul
}

impl TypeError {
    pub fn new<T: ToString>(msg: T) -> Box<Self> {
        Box::new(TypeError {
            msg: msg.to_string(),
        })
    }
}

impl std::error::Error for TypeError {}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Statement {
    pub(crate) fn resolve(&self) -> Result<Var, Box<dyn std::error::Error>> {
        let r = self.op.get().unwrap_func().call(&self.args, &self.loc);
        if let Ok(s) = &r {
            *self.res.borrow_mut() = Some(s.new_ref());
        }
        r
    }
    pub(crate) fn new<Op: Callable + 'static, AL: Into<Vec<Var>>>(
        o: Op,
        args: AL,
        loc: Location,
    ) -> Statement {
        let o = Box::new(o);
        let args = args.into();
        Statement {
            op: Var::new(LispType::Func(o)),
            args,
            res: RefCell::new(None),
            loc,
        }
    }
}

#[allow(dead_code)]
impl Var {
    pub(crate) fn new<T: Into<LispType>>(i: T) -> Var {
        Var {
            dat: Rc::new(RefCell::new(i.into())),
        }
    }
    pub(crate) fn new_ref(&self) -> Var {
        Var {
            dat: Rc::clone(&self.dat),
        }
    }
    pub(crate) fn get(&self) -> Ref<LispType> {
        self.dat.borrow()
    }
    pub(crate) fn get_mut(&self) -> RefMut<LispType> {
        self.dat.borrow_mut()
    }
    pub(crate) fn resolve(&self) -> Result<Self, Box<dyn std::error::Error>> {
        match &*self.dat.borrow() {
            LispType::Statement(s) => s.resolve(),
            _ => Ok(self.new_ref()),
        }
    }
    pub(crate) fn unwrap(self) -> LispType {
        Rc::try_unwrap(self.dat).unwrap().into_inner()
    }
}

#[derive(Debug)]
pub(crate) struct Scope {
    pub(crate) vars: BTreeMap<String, Var>,
}

impl std::default::Default for Scope {
    fn default() -> Self {
        let items = [
            ("print", IntrinsicOp::Print),
            ("+", IntrinsicOp::Add),
            ("-", IntrinsicOp::Subtract),
            ("*", IntrinsicOp::Multiply),
        ];
        Scope {
            vars: items
                .into_iter()
                .map(|x| (x.0.to_string(), Var::new(x.1)))
                .collect(),
        }
    }
}

pub(crate) fn make_ast(
    ts: &[Token],
    idents: &Scope,
    start: &Location,
) -> Result<Statement, String> {
    // TODOOOOOOOOOOO(#7): Declaring variables
    let mut open_stack = Vec::new();
    let mut args = Vec::new();
    let mut loc = None;

    let mut start_idx = 0;
    if let TokenType::StartStmt = ts[start_idx].dat {
        start_idx = 1;
    }
    let mut end_idx = ts.len() - 1;
    if let TokenType::EndStmt = ts[end_idx].dat {
        end_idx -= 1;
    }
    for i in start_idx..=end_idx {
        match &ts[i].dat {
            TokenType::StartStmt => {
                open_stack.push(i);
            }
            TokenType::EndStmt => {
                if let Some(o) = open_stack.pop() {
                    if open_stack.is_empty() {
                        args.push(Var::new(make_ast(&ts[o..=i], &idents, &ts[o + 1].loc)?));
                    }
                } else {
                    return Err(format!("{} - Unmatched closing parenthesis!", ts[i].loc));
                }
            }
            TokenType::Recognizable(n) => {
                if open_stack.is_empty() {
                    args.push(Var::new(n.clone()));
                }
            }
            TokenType::Ident(id) => match idents.vars.get(&id.to_string()) {
                None => return Err(format!("{} - Unknown identifier `{id}`!", ts[i].loc)),
                Some(s) => {
                    if open_stack.is_empty() {
                        args.push(s.new_ref());
                        loc = Some(ts[i].loc.clone());
                    }
                }
            },
        }
    }
    if !open_stack.is_empty() {
        return Err(format!(
            "{} - Unmatched opening parenthesis!",
            ts[open_stack.pop().unwrap()].loc
        ));
    }
    if args.first().is_none() {
        return Err(format!("{} - Empty statements are not allowed!", start));
    }
    let s = args.remove(0);
    if let LispType::Func(_) = *s.get() {
    } else {
        // TODOO(#8): Making raw lists
        return Err(format!("{start} - Cannot make a raw list (Yet..)!"));
    }
    Ok(Statement {
        args,
        op: s,
        res: RefCell::new(None),
        loc: loc.unwrap(),
    })
}
