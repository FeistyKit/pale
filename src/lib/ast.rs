use crate::callable::{Callable, IntrinsicOp};
use crate::error::LispErrors;
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

impl Statement {
    pub(crate) fn resolve(&self) -> Result<Var, LispErrors> {
        let r = self.op.get().unwrap_func().call(&self.args, &self.loc);
        if let Ok(s) = &r {
            *self.res.borrow_mut() = Some(s.new_ref());
        }
        r
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
    pub(crate) fn resolve(&self) -> Result<Self, LispErrors> {
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

#[derive(Debug)]
struct AstParser<'a> {
    ts: &'a [Token],
    idents: &'a mut Scope,
    start: &'a Location,
    open_stack: Vec<usize>,
    args: Vec<Var>,
    loc: Option<Location>,
}

impl<'a> AstParser<'a> {
    fn new(ts: &'a [Token], idents: &'a mut Scope, start: &'a Location) -> Self {
        Self {
            ts,
            idents,
            start,
            loc: None,
            open_stack: Vec::new(),
            args: Vec::new(),
        }
    }
    fn parse(mut self) -> Result<Statement, LispErrors> {
        if self.ts.len() < 2 {
            return Err(LispErrors::new().error(self.start, "Empty statements are not allowed!"));
        }
        let mut start_idx = 0;
        if let TokenType::StartStmt = self.ts[start_idx].dat {
            start_idx = 1;
        }
        let mut end_idx = self.ts.len() - 1;
        if let TokenType::EndStmt = self.ts[end_idx].dat {
            end_idx -= 1;
        }
        for i in start_idx..=end_idx {
            match &self.ts[i].dat {
                TokenType::StartStmt => {
                    self.open_stack.push(i);
                }
                TokenType::EndStmt => {
                    if let Some(o) = self.open_stack.pop() {
                        if self.open_stack.is_empty() {
                            self.args.push(Var::new(make_ast(
                                &self.ts[o..=i],
                                &mut self.idents,
                                &self.ts[o + 1].loc,
                            )?));
                        }
                    } else {
                        return Err(LispErrors::new()
                            .error(&self.ts[i].loc, "Unmatched closing parentheses!")
                            .note(None, "Delete it."));
                    }
                }
                TokenType::KeyWord(_) => todo!(),
                TokenType::Recognizable(n) => {
                    if self.open_stack.is_empty() {
                        self.args.push(Var::new(n.clone()));
                    }
                }
                TokenType::Ident(id) => match self.idents.vars.get(&id.to_string()) {
                    None => {
                        return Err(LispErrors::new()
                            .error(&self.ts[i].loc, format!("Unknown identifier `{id}`!")))
                    }
                    Some(s) => {
                        if self.open_stack.is_empty() {
                            self.args.push(s.new_ref());
                            self.loc = Some(self.ts[i].loc.clone());
                        }
                    }
                },
            }
        }
        if !self.open_stack.is_empty() {
            return Err(LispErrors::new()
                .error(
                    &self.ts[self.open_stack.pop().unwrap()].loc,
                    "Unmatched opening parentheses!",
                )
                .note(None, "Deleting it might fix this error."));
        }
        let s = self.args.remove(0);
        if let LispType::Func(_) = *s.get() {
        } else {
            // TODOO(#8): Making raw lists
            return Err(LispErrors::new()
                .error(&self.start, "Raw lists are not available (Yet...)!")
                .note(None, "This is not a function.")
                .note(None, "Use the `list` intrinsic to convert this to a list."));
        }
        Ok(Statement {
            args: self.args,
            op: s,
            res: RefCell::new(None),
            loc: self.loc.unwrap(),
        })
    }
}

pub(crate) fn make_ast(
    ts: &[Token],
    idents: &mut Scope,
    start: &Location,
) -> Result<Statement, LispErrors> {
    // TODOOOOOOOOOOO(#7): Declaring variables
    let ast_parser = AstParser::new(ts, idents, start);
    ast_parser.parse()
}
