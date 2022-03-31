pub fn run_lisp<'a>(source: &str, source_name: impl Into<Option<&'a str>>) -> Result<(), String> {
    todo!()
}

#[derive(Debug, Clone)]
struct Scanner {
    source: Vec<char>,
    current: usize,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            current: 0,
        }
    }
    fn scan_tokens(&mut self) -> Vec<Token> {
        let mut toks = Vec::new();
        while !self.finished() {
            toks.push(self.next_token());
        }
        for tok in &toks {
            println!("{tok:?}")
        }
        toks
    }
    fn next_token(&mut self) -> Token {
        todo!()
    }
    fn finished(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, Clone)]
enum Token {}
