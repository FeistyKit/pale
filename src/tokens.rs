use std::fmt::Display;
use std::mem;
use std::str::FromStr;

use crate::error::LispErrors;
use crate::types::LispValue;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) loc: Location,
    pub(crate) dat: TokenType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub filename: String,
    pub line: usize,
    pub col: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum KeyWord {
    Let,
    Lambda,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenType {
    StartStmt,
    EndStmt,
    KeyWord(KeyWord),
    Recognizable(LispValue),
    Ident(String),
}

impl FromStr for KeyWord {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "let" => Ok(Self::Let),
            "lambda" => Ok(Self::Lambda),
            _ => Err("Unknown keyword!"),
        }
    }
}

impl TokenType {
    fn new_str_lit(source: String) -> Self {
        Self::Recognizable(LispValue::Str(source))
    }
}

impl<T: ToString> From<T> for TokenType {
    fn from(orig: T) -> Self {
        let s = orig.to_string().trim().to_string();
        if let Ok(k) = s.parse::<KeyWord>() {
            Self::KeyWord(k)
        } else if let Ok(i) = s.parse::<isize>() {
            Self::Recognizable(i.into())
        } else if let Ok(f) = s.parse::<f64>() {
            Self::Recognizable(f.into())
        } else if &s == "nil" {
            Self::Recognizable(LispValue::Nil)
        } else {
            Self::Ident(orig.to_string())
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TokenizerStatus {
    String,
    Normal,
    Comment,
}

#[derive(Debug)]
struct Tokenizer<'a> {
    tokens: Vec<Token>,
    right_assocs: usize,
    pos: (usize, usize),
    pos_locked: bool,
    token_buf: String,
    status: TokenizerStatus,
    default_buf_len: usize,
    filename: String,
    source: &'a str,
    last_character: char,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str, filename: String) -> Self {
        // This number can and might change, or I might change the method of getting it.
        let default_buf_len = 16;
        Tokenizer {
            tokens: Vec::with_capacity(default_buf_len),
            pos: (0, 0),
            pos_locked: false,
            token_buf: String::with_capacity(default_buf_len),
            status: TokenizerStatus::Normal,
            default_buf_len,
            filename,
            source: input,
            right_assocs: 0,
            last_character: ' ',
        }
    }

    fn push_tok(&mut self) {
        match self.status {
            TokenizerStatus::Normal => {
                if self.token_buf.trim() != "" {
                    let tok = Token {
                        loc: Location {
                            line: self.pos.1,
                            col: self.pos.0,
                            filename: self.filename.clone(),
                        },
                        dat: mem::replace(
                            &mut self.token_buf,
                            String::with_capacity(self.default_buf_len),
                        )
                        .into(),
                    };
                    self.tokens.push(tok);
                    self.pos_locked = false;
                }
            }
            TokenizerStatus::Comment => unreachable!(),
            TokenizerStatus::String => {
                let tok = Token {
                    loc: Location {
                        line: self.pos.1,
                        col: self.pos.0,
                        filename: self.filename.clone(),
                    },
                    dat: TokenType::new_str_lit(mem::replace(
                        &mut self.token_buf,
                        String::with_capacity(self.default_buf_len),
                    )),
                };
                self.tokens.push(tok);
                self.pos_locked = false;
                self.status = TokenizerStatus::Normal;
            }
        }
    }

    fn start_stmt(&mut self) {
        let tok = Token {
            loc: Location {
                filename: self.filename.clone(),
                line: self.pos.1,
                col: self.pos.0,
            },
            dat: TokenType::StartStmt,
        };
        self.tokens.push(tok);
    }

    fn end_stmt(&mut self) {
        self.token_buf = self.token_buf.trim().to_string();
        if !self.token_buf.is_empty() {
            let tok = Token {
                loc: Location {
                    filename: self.filename.clone(),
                    line: self.pos.1,
                    col: self.pos.0,
                },
                dat: mem::replace(
                    &mut self.token_buf,
                    String::with_capacity(self.default_buf_len),
                )
                .into(),
            };
            self.token_buf = String::with_capacity(self.default_buf_len);
            self.tokens.push(tok);
        }
        for _ in 0..self.right_assocs {
            let tok = Token {
                loc: Location {
                    filename: self.filename.clone(),
                    line: self.pos.1,
                    col: self.pos.0,
                },
                dat: TokenType::EndStmt,
            };
            self.tokens.push(tok);
        }
        self.right_assocs = 0;
        self.pos_locked = false;
        self.status = TokenizerStatus::Normal;
        let tok = Token {
            loc: Location {
                filename: self.filename.clone(),
                line: self.pos.1,
                col: self.pos.0,
            },
            dat: TokenType::EndStmt,
        };
        self.tokens.push(tok);
    }

    fn tokenize(mut self) -> Result<Vec<Token>, LispErrors> {
        'lines: for (line_number, line_data) in self.source.lines().enumerate() {
            for (col_number, character) in line_data.trim().char_indices() {
                match (character, self.status, self.last_character) {
                    ('\"', TokenizerStatus::String, _) => self.push_tok(),
                    (_, TokenizerStatus::String, _) => self.token_buf.push(character),
                    ('\"', TokenizerStatus::Normal, _) => self.status = TokenizerStatus::String,
                    (' ', TokenizerStatus::Normal, _) => self.push_tok(),
                    ('(', TokenizerStatus::Normal, _) => self.start_stmt(),
                    (')', TokenizerStatus::Normal, _) => self.end_stmt(),
                    ('/', TokenizerStatus::Normal, '/') => continue 'lines,
                    ('$', TokenizerStatus::Normal, _) => {
                        self.start_stmt();
                        self.right_assocs += 1;
                    }
                    ('*', TokenizerStatus::Normal, '{') => self.status = TokenizerStatus::Comment,
                    (_, TokenizerStatus::Normal, _) => self.token_buf.push(character),
                    ('}', TokenizerStatus::Comment, '*') => self.status = TokenizerStatus::Normal,
                    (_, TokenizerStatus::Comment, _) => {}
                }
                self.last_character = character;
                if !self.pos_locked {
                    self.pos = (col_number, line_number);
                }
            }
        }

        for _ in 0..self.right_assocs {
            let tok = Token {
                loc: Location {
                    filename: self.filename.clone(),
                    line: self.pos.1,
                    col: self.pos.0,
                },
                dat: TokenType::EndStmt,
            };
            self.tokens.push(tok);
        }
        Ok(self.tokens)
    }
}

pub fn tokenize(source: &str, filename: String) -> Result<Vec<Token>, LispErrors> {
    let tokenizer = Tokenizer::new(source, filename);
    tokenizer.tokenize()
}
