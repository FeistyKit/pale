pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}
pub(crate) fn process_identifiers(tokens: &[Token]) -> Result<Vec<(&str, Either<&str, Var>)>, LispErrors> {

}

    (&mut self, tokens: &[Token]) -> Result<(), LispErrors> {
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
