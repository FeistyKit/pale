use std::{
    env, fs,
    io::{self, BufRead, Write},
};

use pale::run_lisp;

fn main() -> Result<(), io::Error> {
    let mut args = env::args();
    let name = args.next();
    match args.next() {
        None => run_interpreter()?,
        Some(s) => run_file(s.as_str())?,
    }
    Ok(())
}

fn run_file(name: &str) -> Result<(), io::Error> {
    let source = fs::read_to_string(name)?;
    if let Err(e) = run_lisp(source.as_str(), name) {
        eprintln!("{e}");
    }
    Ok(())
}

fn run_interpreter() -> Result<(), io::Error> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();

    loop {
        //TODO: Customize prompt
        const PROMPT: &'static str = "> ";
        print_flushed(PROMPT)?;

        stdin.read_line(&mut line)?;

        if line.trim() == "" || line.trim() == "exit" {
            return Ok(());
        }

        if let Err(e) = run_lisp(line.as_str(), "<repl>") {
            eprintln!("{e}");
        }
    }
}

fn print_flushed(val: &str) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write(val.as_bytes())?;
    stdout.flush()?;
    Ok(())
}
