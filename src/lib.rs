use error::LispErrors;

use crate::ast::{make_ast, Scope, Var};
use crate::tokens::{tokenize, Location};

mod ast;
mod callable;
mod error;
mod tokens;
mod types;

pub fn run_lisp(source: &str, file: &str) -> Result<String, LispErrors> {
    let toks = tokenize(source, file.to_string())?;
    let ast = make_ast(
        &toks,
        &mut Scope::default(),
        &Location {
            filename: file.to_string(),
            col: 0,
            line: 0,
        },
    )?;
    Ok(format!("{}", ast.resolve()?))
}

#[cfg(feature = "debug")]
pub fn run_lisp_dumped(source: &str, file: &str) -> Result<String, LispErrors> {
    let toks = tokenize(source, file.to_string())?;
    for tok in &toks {
        println!("{} => {:?}", tok.loc, tok.dat);
    }
    let ast = make_ast(
        &toks,
        &mut Scope::default(),
        &Location {
            filename: file.to_string(),
            col: 0,
            line: 0,
        },
    )?;
    println!("Ast = {ast:#?}");
    Ok(format!("{}", ast.resolve()?))
}

#[cfg(test)]
mod tests {
    use crate::{
        run_lisp, tokenize,
        tokens::{Location, Token, TokenType},
        types::LispType,
    };
    #[test]
    fn test_tokenizer() {
        let expected_res = [
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 0,
                },
                dat: TokenType::StartStmt,
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 1,
                },
                dat: TokenType::Ident("+".to_string()),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 3,
                },
                dat: TokenType::StartStmt,
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 4,
                },
                dat: TokenType::Ident("-".to_string()),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 6,
                },
                dat: TokenType::Recognizable(LispType::Integer(1)),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 8,
                },
                dat: TokenType::Recognizable(LispType::Integer(23)),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 11,
                },
                dat: TokenType::Recognizable(LispType::Integer(23423423)),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 19,
                },
                dat: TokenType::EndStmt,
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 20,
                },
                dat: TokenType::Ident("\"sliijioo\"".to_string()),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 31,
                },
                dat: TokenType::EndStmt,
            },
        ];
        assert_eq!(
            Ok(expected_res.to_vec()),
            tokenize("(+ (- 1 23 23423423) \"sliijioo\")", "-")
        );
    }
    #[test]
    fn test_addition() {
        let source = "(+ 34 (+ 34 1))";
        assert_eq!(
            *run_lisp(source, "<provided>").unwrap().get(),
            LispType::Integer(69)
        );
    }
}
