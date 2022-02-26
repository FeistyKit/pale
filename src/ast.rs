#![allow(clippy::or_fun_call)]

use crate::callable::IntrinsicOp;
use crate::error::LispErrors;
use crate::identifiers::{process_identifiers, Either, Identifier};
use crate::tokens::{KeyWord, Token, TokenType};
use crate::types::LispValue;
use crate::Location;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    fmt::Display,
    rc::Rc,
};

#[derive(Debug, PartialEq)]
pub struct Var {
    pub(crate) dat: Rc<RefCell<LispValue>>,
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
    pub(crate) fn maybe_clone(&self) -> Self {
        match &*self.dat.borrow() {
            LispValue::Func(f) => match f.try_clone() {
                Some(f) => Var::new(f),
                None => self.new_ref(),
            },
            LispValue::Statement(_) => {
                unimplemented!()
            }
            _ => Var::new(self.dat.borrow().clone()),
        }
    }

    pub(crate) fn new<T: Into<LispValue>>(i: T) -> Var {
        Var {
            dat: Rc::new(RefCell::new(i.into())),
        }
    }

    #[inline(always)]
    pub(crate) fn new_nil() -> Var {
        Var {
            dat: Rc::new(RefCell::new(LispValue::Nil)),
        }
    }

    pub(crate) fn new_ref(&self) -> Var {
        Var {
            dat: Rc::clone(&self.dat),
        }
    }
    pub(crate) fn get(&self) -> Ref<LispValue> {
        self.dat.borrow()
    }
    pub(crate) fn get_mut(&self) -> RefMut<LispValue> {
        self.dat.borrow_mut()
    }
    pub(crate) fn resolve(&self) -> Result<Self, LispErrors> {
        match &*self.dat.borrow() {
            LispValue::Statement(s) => s.resolve(),
            _ => Ok(self.new_ref()),
        }
    }
    pub(crate) fn unwrap(self) -> LispValue {
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
    status: AstParserStatus,
}

#[derive(Debug, Clone)]
enum AstParserStatus {
    Normal,
    Identifiers(usize, Vec<usize>),
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
            status: AstParserStatus::Normal,
        }
    }

    fn introduce_identifier(
        &mut self,
        ident: &str,
        value: Option<Var>,
        loc: &Location,
    ) -> Result<(), LispErrors> {
        let value = value.unwrap_or(Var::new(LispValue::Nil));
        let ident = ident.to_string();
        if self.idents.vars.contains_key(&ident) {
            //TODO(#12): Shadowing
            return Err(LispErrors::new()
                .error(loc, "Shadowing is not currently allowed!")
                .note(None, "Change its name."));
        }
        self.idents.vars.insert(ident, value);
        Ok(())
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
        if start_idx > end_idx {
            return Err(LispErrors::new().error(self.start, "Empty statements are not allowed!"));
        }
        for i in start_idx..=end_idx {
            match (&mut self.status, &self.ts[i].dat) {
                (AstParserStatus::Normal, TokenType::StartStmt) => {
                    self.open_stack.push(i);
                }
                (AstParserStatus::Normal, TokenType::EndStmt) => {
                    if let Some(o) = self.open_stack.pop() {
                        if self.open_stack.is_empty() {
                            self.args.push(Var::new(make_ast(
                                &self.ts[o..=i],
                                self.idents,
                                &self.ts[o + 1].loc,
                            )?));
                        }
                    } else {
                        return Err(LispErrors::new()
                            .error(&self.ts[i].loc, "Unmatched closing parentheses!")
                            .note(None, "Delete it."));
                    }
                }
                (AstParserStatus::Normal, TokenType::KeyWord(word)) => match word {
                    KeyWord::Let => {
                        self.status = AstParserStatus::Identifiers(i, Vec::new());
                    }
                    KeyWord::Lambda => unimplemented!(),
                },
                (AstParserStatus::Normal, TokenType::Recognizable(n)) => {
                    if self.open_stack.is_empty() {
                        self.args.push(Var::new(n.clone()));
                    }
                }
                (AstParserStatus::Normal, TokenType::Ident(id)) => match self.idents.vars.get(id) {
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
                (AstParserStatus::Identifiers(_, positions), TokenType::StartStmt) => {
                    positions.push(i)
                }
                (AstParserStatus::Identifiers(start, positions), TokenType::EndStmt) => {
                    positions.pop();
                    if positions.is_empty() {
                        let t = *start; // For some reason this is required for the borrow checker to allow it.
                        let vals = process_identifiers(&self.ts[t + 2..i], &mut self.idents)?;
                        for Identifier {
                            ident: i,
                            data: d,
                            loc_introduced: l,
                        } in vals
                        {
                            match d {
                                Either::Right(real_value) => {
                                    self.introduce_identifier(i, Some(real_value), l)?
                                }
                                //TODO: Making variables depend upon others in a statement.
                                // For example: "(let ((x 8) (y x)) ...)"
                                Either::Left(_name) => return Err(LispErrors::new().error(l, "Making a variable depend upon another in the statement is not currently implemented!")),
                            }
                        }
                        self.status = AstParserStatus::Normal;
                    }
                }
                (_, _) => {}
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
        if s.get().is_func() {
            Ok(Statement {
                args: self.args,
                op: s,
                res: RefCell::new(None),
                loc: self.loc.unwrap(),
            })
        } else if self.args.is_empty() {
            if s.get().is_stmt() {
                let s = s.unwrap();
                match s {
                    LispValue::Statement(s) => Ok(s),
                    _ => Err(LispErrors::new()
                        .error(self.start, "Raw lists are not available (Yet...)!")
                        .note(None, "This is not a function.")
                        .note(None, "Use the `list` intrinsic to convert this to a list.")),
                }
            } else {
                Err(LispErrors::new()
                    .error(self.start, "Raw lists are not available (Yet...)!")
                    .note(None, "This is not a function.")
                    .note(None, "Use the `list` intrinsic to convert this to a list."))
            }
        } else {
            // TODOO(#8): Making raw lists
            Err(LispErrors::new()
                .error(self.start, "Raw lists are not available (Yet...)!")
                .note(None, "This is not a function.")
                .note(None, "Use the `list` intrinsic to convert this to a list."))
        }
    }
}

pub(crate) fn make_ast(
    ts: &[Token],
    idents: &mut Scope,
    start: &Location,
) -> Result<Statement, LispErrors> {
    let ast_parser = AstParser::new(ts, idents, start);
    ast_parser.parse()
}
