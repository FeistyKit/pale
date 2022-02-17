#![allow(clippy::or_fun_call)]
use clap::Parser;
use pale::{run_lisp, run_lisp_dumped};
use std::{error, fs};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short = 'c', long = "command")]
    is_command: bool,

    #[clap(short, long)]
    debug: bool,

    input: Option<String>,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();
    let (source, file) = if args.is_command {
        if let Some(s) = args.input {
            (s, "<provided>".to_string())
        } else {
            return Err("A command must be provided!".into());
        }
    } else {
        if let Some(s) = args.input {
            (fs::read_to_string(&s).unwrap(), s)
        } else {
            // TODOOOOO: Running the interpreter off standard input.
            return Err("Running in REPL mode is not yet implemented!".into());
        }
    };
    if args.debug {
        run_lisp(&source, &file)?;
    } else {
        run_lisp_dumped(&source, &file)?;
    }
    Ok(())
}
