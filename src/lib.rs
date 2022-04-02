use core::fmt;

pub fn run_lisp<'a>(source: &str, source_name: impl Into<Option<&'a str>>) -> Result<(), String> {
    todo!()
}

#[derive(Debug, Clone)]
pub struct Location {
    col: usize,
    line: usize,
    source_name: String,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}:", self.source_name, self.line, self.col)
    }
}

#[derive(Debug, Clone)]
pub struct LispError {
    msg: String,
    loc: Location,
}

#[derive(Debug, Clone)]
struct Scanner {
    source: Vec<char>,
    current: usize,
    start: usize,
    line: usize,
    col: usize,
    source_name: String,
    toks: Vec<Token>,
}

impl Scanner {
    fn new(source: &str, source_name: &impl ToString) -> Self {
        Scanner {
            source: source.chars().collect(),
            current: 0,
            start: 0,
            line: 0,
            col: 0,
            source_name: source_name.to_string(),
            toks: Vec::new(),
        }
    }

    fn scan_tokens(mut self) -> Result<Vec<Token>, LispError> {
        while !self.finished() {
            self.start = self.current;
            self.next_token()?;
        }
        self.toks.push(Token::new(
            TokenType::End,
            &self.source[self.start..=self.current],
            self.current_loc(),
        ));
        for tok in &self.toks {
            println!("{tok:?}")
        }
        Ok(self.toks)
    }

    fn next_token(&mut self) -> Result<(), LispError> {
        self.current += 1;
        todo!()
    }

    fn current_loc(&self) -> Location {
        Location {
            col: self.col,
            line: self.line,
            source_name: self.source_name,
        }
    }

    fn finished(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, Clone)]
enum TokenType {
    OpenParen,
    CloseParen,
    Hash,
    Quote,
    Identifier(Vec<char>),
    String(String),
    Number(i128),
    Group,
    End,
}

#[derive(Clone)]
struct Token {
    loc: Location,
    original: Vec<char>,
    toktype: TokenType,
}

impl Token {
    fn new(toktype: TokenType, orig: &[char], loc: Location) -> Self {
        Self {
            toktype,
            loc,
            original: orig.to_owned(),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:?} ({})",
            self.loc,
            self.toktype,
            self.original.iter().collect::<String>()
        )
    }
}
