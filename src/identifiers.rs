use std::fmt::Debug;

use crate::{
    ast::{Scope, Var},
    error::LispErrors,
    tokens::{Location, Token, TokenType},
};

pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Debug, R: Debug> Debug for Either<L, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(arg0) => f.debug_tuple("Left").field(arg0).finish(),
            Self::Right(arg0) => f.debug_tuple("Right").field(arg0).finish(),
        }
    }
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

#[derive(Debug)]
pub(crate) struct Identifier<'a> {
    pub(crate) ident: &'a str,
    pub(crate) data: Either<&'a str, Var>,
    pub(crate) loc_introduced: &'a Location,
}

impl<'a> From<(&'a str, Either<&'a str, Var>, &'a Location)> for Identifier<'a> {
    fn from(other: (&'a str, Either<&'a str, Var>, &'a Location)) -> Self {
        Identifier {
            ident: other.0,
            data: other.1,
            loc_introduced: other.2,
        }
    }
}

impl<'a> From<(&'a str, &'a str, &'a Location)> for Identifier<'a> {
    fn from(other: (&'a str, &'a str, &'a Location)) -> Self {
        Identifier {
            ident: other.0,
            data: Either::Left(other.1),
            loc_introduced: other.2,
        }
    }
}

impl<'a> From<(&'a str, Var, &'a Location)> for Identifier<'a> {
    fn from(other: (&'a str, Var, &'a Location)) -> Self {
        Identifier {
            ident: other.0,
            data: Either::Right(other.1),
            loc_introduced: other.2,
        }
    }
}

pub(crate) fn process_identifiers<'a>(
    tokens: &'a [Token],
    idents: &Scope,
) -> Result<Vec<Identifier<'a>>, LispErrors> {
    let mut to_introduce: Vec<Identifier> = Vec::new();
    let mut status = IdentParserStatus::Normal;
    for tok in tokens {
        match (&tok.dat, &mut status) {
            (TokenType::Ident(id), IdentParserStatus::Normal) => {
                //TODO: Refactor this
                to_introduce.push((id.as_str(), Var::new_nil(), &tok.loc).into())
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
            ) => return Err(LispErrors::new().error(&tok.loc, "Variable names must be literals!")),
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
            ) => match idents.vars.get(id.as_str()) {
                None => {
                    return Err(
                        LispErrors::new().error(&tok.loc, format!("Unknown identifier {id:?}!"))
                    )
                }
                Some(s) => {
                    to_introduce.push((*new_id, s.new_ref(), &tok.loc).into());
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
                to_introduce.push((*id, Var::new(value.clone()), &tok.loc).into());
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
                       // TODOO(#13): arbitrary values in `let` expressions
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
            ) => return Err(LispErrors::new().error(&tok.loc, "Cannot assign to literal value!")),
        }
    }
    Ok(to_introduce)
}
