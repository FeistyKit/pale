#![allow(clippy::or_fun_call)]

use crate::callable::IntrinsicOp;
use crate::error::LispErrors;
use crate::tokens::{KeyWord, Token, TokenType};
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
    status: AstParserStatus,
}

#[derive(Debug, Clone)]
enum AstParserStatus {
    Normal,
    Identifiers(usize, Vec<usize>),
}

#[derive(Debug)]
enum IdentParserStatus<'a> {
    Normal,
    Specific {
        introducing_loc: &'a Location,
        ident: Option<&'a str>,
        has_value: bool, // Whether a value has been inserted in the scope
    },
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
        let value = value.unwrap_or(Var::new(LispType::Nil));
        let ident = ident.to_string();
        if self.idents.vars.contains_key(&ident) {
            //TODO: Shadowing
            return Err(LispErrors::new()
                .error(loc, "Shadowing is not currently allowed!")
                .note(None, "Change its name."));
        }
        self.idents.vars.insert(ident, value);
        Ok(())
    }

    fn process_identifiers(&mut self, tokens: &[Token]) -> Result<(), LispErrors> {
        let mut to_introduce: Vec<(&str, Option<Var>, &Location)> = Vec::new();
        let mut status = IdentParserStatus::Normal;
        for tok in tokens {
            match (&tok.dat, &mut status) {
                (TokenType::Ident(id), IdentParserStatus::Normal) => {
                    to_introduce.push((id, None, &tok.loc))
                }
                (TokenType::StartStmt, IdentParserStatus::Normal) => {
                    status = IdentParserStatus::Specific {
                        introducing_loc: &tok.loc,
                        ident: None,
                        has_value: false,
                    }
                }
                (
                    TokenType::StartStmt,
                    IdentParserStatus::Specific {
                        introducing_loc: _,
                        ident: None,
                        has_value: _,
                    },
                ) => {
                    return Err(
                        LispErrors::new().error(&tok.loc, "Variable names must be literals!")
                    )
                }
                (
                    TokenType::Ident(id),
                    IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: None,
                        has_value: _,
                    },
                ) => {
                    status = IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(id),
                        has_value: false,
                    }
                }
                (
                    TokenType::Ident(id),
                    IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(new_id),
                        has_value: false,
                    },
                ) => match self.idents.vars.get(id.as_str()) {
                    None => {
                        return Err(LispErrors::new()
                            .error(&tok.loc, format!("Unknown identifier {id:?}!")))
                    }
                    Some(s) => {
                        to_introduce.push((new_id, Some(s.new_ref()), &tok.loc));
                        status = IdentParserStatus::Specific {
                            introducing_loc: l,
                            ident: Some(new_id),
                            has_value: true,
                        }
                    }
                },
                (
                    TokenType::Ident(_),
                    IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(_),
                        has_value: true,
                    },
                ) => {
                    return Err(LispErrors::new()
                        .error(l, "Identifier not allowed here!")
                        .note(*l, "Remove it"))
                }
                (
                    TokenType::Recognizable(value),
                    IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(id),
                        has_value: _,
                    },
                ) => {
                    to_introduce.push((id, Some(Var::new(value.clone())), &tok.loc));
                    status = IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(id),
                        has_value: true,
                    }
                }
                (
                    TokenType::EndStmt,
                    IdentParserStatus::Specific {
                        introducing_loc: l,
                        ident: Some(_),
                        has_value: false,
                    },
                ) => {
                    return Err(LispErrors::new()
                        .error(
                            l,
                            "Variable defined in parentheses must have an initial value.",
                        )
                        .note(*l, "Remove the parentheses around it."))
                }
                (
                    TokenType::EndStmt,
                    IdentParserStatus::Specific {
                        introducing_loc: _,
                        ident: Some(_),
                        has_value: true,
                    },
                ) => {
                    status = IdentParserStatus::Normal;
                }
                (TokenType::KeyWord(_), _) => {
                    return Err(LispErrors::new().error(
                        &tok.loc,
                        "Keywords are not allowed in variable assignments!",
                    ))
                }
                (
                    TokenType::StartStmt,
                    &mut IdentParserStatus::Specific {
                        introducing_loc: _,
                        ident: Some(_id),
                        has_value: false,
                    },
                ) => {
                    return Err(
                        LispErrors::new().error(
                            &tok.loc,
                            "Variables must be literals or other values (not expressions)!",
                        ), // .note(
                           //     None,
                           //     "You can express this as `(let {_id}) (set id <value>)`",
                           // )
                           // @set
                           // TODOO: arbitrary values in `let` expressions
                    );
                }
                (
                    TokenType::StartStmt,
                    &mut IdentParserStatus::Specific {
                        introducing_loc: _,
                        ident: Some(_id),
                        has_value: true,
                    },
                ) => {
                    return Err(LispErrors::new()
                        .error(&tok.loc, "Unknown opening parenthesis.")
                        .note(&tok.loc, "Delete it."));
                }
                (TokenType::EndStmt, _) => unreachable!(),
                (TokenType::Recognizable(_), IdentParserStatus::Normal) => {
                    return Err(LispErrors::new()
                        .error(&tok.loc, "Unknown literal in `let` statement.")
                        .note(None, "Bind it to a variable name.")
                        .note(&tok.loc, "Delete it."))
                }
                (
                    TokenType::Recognizable(_),
                    IdentParserStatus::Specific {
                        introducing_loc: _,
                        ident: None,
                        has_value: _,
                    },
                ) => {
                    return Err(LispErrors::new().error(&tok.loc, "Cannot assign to literal value!"))
                }
            }
        }
        for (ident, value, loc) in to_introduce {
            self.introduce_identifier(ident, value, loc)?;
        }
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
                        self.process_identifiers(&self.ts[t + 2..i])?;
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
        if let LispType::Func(_) = *s.get() {
        } else {
            // TODOO(#8): Making raw lists
            return Err(LispErrors::new()
                .error(self.start, "Raw lists are not available (Yet...)!")
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
