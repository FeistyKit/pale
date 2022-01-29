use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    fmt::{Debug, Display},
    rc::Rc,
};

fn main() {
    let r = tokenize("(+ (- 1 23 23423423) \"sliijioo\")", "-");
    println!("{r:#?}")
}

fn main2() {
    let a1 = Var::new(34);
    let a2 = Var::new(35);
    let stmt = Statement::new(Operation::Add, [a1, a2]);
    let res = stmt.resolve().unwrap();
    Statement::new(Operation::Print, vec![res])
        .resolve()
        .unwrap();
    let a1 = Var::new("Nice. ( ͡° ͜ʖ ͡°)");
    Statement::new(Operation::Print, vec![a1])
        .resolve()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{tokenize, Location, Token, TokenType};
    #[test]
    fn test_tokenizer() {
        let expected_res = [
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 0,
                },
                dat: TokenType::OpenParens,
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
                dat: TokenType::OpenParens,
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
                dat: TokenType::Num(1),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 8,
                },
                dat: TokenType::Num(23),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 11,
                },
                dat: TokenType::Num(23423423),
            },
            Token {
                loc: Location {
                    filename: "-".to_string(),
                    line: 0,
                    col: 19,
                },
                dat: TokenType::CloseParens,
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
                dat: TokenType::CloseParens,
            },
        ];
        assert_eq!(
            Ok(expected_res.to_vec()),
            tokenize("(+ (- 1 23 23423423) \"sliijioo\")", "-")
        );
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    loc: Location,
    dat: TokenType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    filename: String,
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    OpenParens,
    CloseParens,
    Num(isize),
    Ident(String),
}

impl<T: ToString> From<T> for TokenType {
    fn from(orig: T) -> Self {
        let s = orig.to_string();
        if let Ok(i) = s.parse::<isize>() {
            Self::Num(i)
        } else {
            Self::Ident(orig.to_string())
        }
    }
}

// Guess the number of tokens that will be produced by tokenize from a single string
// TODO: Improve the algorithm of `guess_capacity` for better performance
fn guess_capacity(input: &str) -> usize {
    input.len() / 5
}

fn tokenize(input: &str, name: &str) -> Result<Vec<Token>, String> {
    let mut to_return = Vec::with_capacity(guess_capacity(input));

    let mut token_buf = String::with_capacity(16);
    let mut token_col = 0;
    let mut token_line = 0;
    for (line_number, line_data) in input.lines().enumerate() {
        for (col_number, character) in line_data.trim().char_indices() {
            match character {
                ' ' => {
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
                '(' => {
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
                ')' => {
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
                _ => token_buf.push(character),
            }
        }
    }
    Ok(to_return)
}

#[derive(Debug)]
pub enum LispType {
    // TODOOOO(#1): Add more types, like lists and floating points;
    Integer(isize),
    Str(String),
    Func(Box<dyn Callable>),
    // TODO(#2): Add custom newtypes.
}

impl Display for LispType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispType::Integer(i) => write!(f, "{i}"),
            LispType::Str(s) => write!(f, "{s}"),
            LispType::Func(fun) => write!(f, "{:?}", fun),
        }
    }
}

pub trait Callable: Debug {
    // TODO(#5): Decide whether to keep the return type of Callable::call as a trait object or an
    // associated type
    fn call(&self, args: &Vec<Var>) -> Result<Var, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Print,
}

impl Callable for Operation {
    fn call(&self, args: &Vec<Var>) -> Result<Var, Box<dyn std::error::Error>> {
        match self {
            Operation::Add => {
                let mut sum = 0;
                for a in args {
                    if let LispType::Integer(i) = *a.get() {
                        sum += i;
                    } else {
                        // TODO(#4): Better error reporting in Statement::resolve with incorrect types
                        return Err(TypeError::new(
                            "Cannot add a non-integer type to an integer!",
                        ));
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Subtract => {
                let mut sum = 0;
                for a in args {
                    if let LispType::Integer(i) = *a.get() {
                        sum -= i;
                    } else {
                        return Err(TypeError::new(
                            "Cannot subtract a non-integer type from an integer!",
                        ));
                    }
                }
                Ok(Var::new(sum))
            }
            Operation::Print => {
                if args.len() != 1 {
                    return Err(TypeError::new(
                        "Print intrinsic requires only one argument!",
                    ));
                } else {
                    println!("{}", args[0]);
                    Ok(Var::new(0))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    args: Vec<Var>,
    op: Box<dyn Callable + 'static>,
}

#[derive(Debug)]
pub struct TypeError {
    msg: String,
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
    pub fn resolve(&self) -> Result<Var, Box<dyn std::error::Error>> {
        self.op.call(&self.args)
    }
    pub fn new<Op: Callable + 'static, AL: Into<Vec<Var>>>(o: Op, args: AL) -> Statement {
        let o = Box::new(o);
        let args = args.into();
        Statement { op: o, args }
    }
}

impl From<isize> for LispType {
    fn from(i: isize) -> Self {
        LispType::Integer(i)
    }
}
impl From<String> for LispType {
    fn from(i: String) -> Self {
        LispType::Str(i)
    }
}
impl From<&str> for LispType {
    fn from(i: &str) -> Self {
        LispType::Str(i.to_string())
    }
}
impl<T: Callable + 'static> From<T> for LispType {
    fn from(i: T) -> Self {
        LispType::Func(Box::new(i))
    }
}

#[derive(Debug)]
pub struct Var {
    dat: Rc<RefCell<LispType>>,
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.get())
    }
}

#[allow(dead_code)]
impl Var {
    fn new<T: Into<LispType>>(i: T) -> Var {
        Var {
            dat: Rc::new(RefCell::new(i.into())),
        }
    }
    fn new_ref(&self) -> Var {
        Var {
            dat: Rc::clone(&self.dat),
        }
    }
    fn get(&self) -> Ref<LispType> {
        self.dat.borrow()
    }
    fn get_mut(&self) -> RefMut<LispType> {
        self.dat.borrow_mut()
    }
}

#[derive(Debug)]
pub struct Scope {
    vars: BTreeMap<String, Var>,
}

impl std::default::Default for Scope {
    fn default() -> Self {
        let items = [
            ("print", Operation::Print),
            ("+", Operation::Add),
            ("-", Operation::Subtract),
        ];
        Scope {
            vars: items
                .into_iter()
                .map(|x| (x.0.to_string(), Var::new(x.1)))
                .collect(),
        }
    }
}
