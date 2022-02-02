use std::fmt::Display;

use crate::types::LispType;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) loc: Location,
    pub(crate) dat: TokenType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub(crate) filename: String,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenType {
    OpenParens,
    CloseParens,
    Recognizable(LispType),
    Ident(String),
}

impl<T: ToString> From<T> for TokenType {
    fn from(orig: T) -> Self {
        let mut s = orig.to_string();
        if let Ok(i) = s.parse::<isize>() {
            Self::Recognizable(i.into())
        } else if let Ok(f) = s.parse::<f64>() {
            Self::Recognizable(f.into())
        } else if s.starts_with('\"') && s.ends_with('\"') {
            s.remove(0);
            s.remove(s.len() - 1);
            Self::Recognizable(LispType::Str(s))
        } else if &s == "nil" {
            Self::Recognizable(LispType::Nil)
        } else {
            Self::Ident(orig.to_string())
        }
    }
}

// Guess the number of tokens that will be produced by tokenize from a single string
// TODO(#6): Improve the algorithm of `guess_capacity` for better performance
fn guess_capacity(input: &str) -> usize {
    input.len() / 5
}

pub(crate) fn tokenize(input: &str, name: &str) -> Result<Vec<Token>, String> {
    let mut to_return = Vec::with_capacity(guess_capacity(input));

    let mut token_buf = String::with_capacity(16);
    let mut token_col = 0;
    let mut token_line = 0;

    let mut in_string = false;
    for (line_number, line_data) in input.lines().enumerate() {
        for (col_number, character) in line_data.trim().char_indices() {
            match (character, in_string) {
                ('\"', true) => {
                    // TODOO(#9): Support escaping in string literals.
                    token_buf.push(character);
                    let tok = Token {
                        loc: Location {
                            line: token_line,
                            col: token_col,
                            filename: name.to_string(),
                        },
                        dat: token_buf.into(),
                    };
                    to_return.push(tok);
                    token_buf = String::with_capacity(16);
                    token_col = col_number + 1;
                    token_line = line_number;
                    in_string = false;
                }
                (_, true) => {
                    token_buf.push(character);
                }
                ('\"', false) => {
                    token_buf.push(character);
                    in_string = true;
                    token_col = col_number;
                    token_line = line_number;
                }
                (' ', false) => {
                    if token_buf.trim() != "" {
                        let tok = Token {
                            loc: Location {
                                line: token_line,
                                col: token_col,
                                filename: name.to_string(),
                            },
                            dat: token_buf.into(),
                        };
                        to_return.push(tok);
                        token_buf = String::with_capacity(16);
                        token_col = col_number + 1;
                        token_line = line_number;
                    }
                }
                ('(', false) => {
                    let tok = Token {
                        loc: Location {
                            line: token_line,
                            col: token_col,
                            filename: name.to_string(),
                        },
                        dat: TokenType::OpenParens,
                    };
                    to_return.push(tok);
                    token_col = col_number + 1;
                    token_line = line_number;
                }
                (')', false) => {
                    if token_buf.trim() != "" {
                        let tok = Token {
                            loc: Location {
                                line: token_line,
                                col: token_col,
                                filename: name.to_string(),
                            },
                            dat: token_buf.into(),
                        };
                        to_return.push(tok);
                        token_buf = String::with_capacity(16);
                        token_col = col_number;
                        token_line = line_number;
                    }
                    let tok2 = Token {
                        loc: Location {
                            line: token_line,
                            col: token_col,
                            filename: name.to_string(),
                        },
                        dat: TokenType::CloseParens,
                    };
                    to_return.push(tok2);
                    token_col = col_number + 1;
                    token_line = line_number;
                }
                (_, false) => token_buf.push(character),
            }
        }
    }
    Ok(to_return)
}
