use std::{env, process};
use sul::{run_lisp, run_lisp_dumped};
fn main() {
    let source = env::args().nth(1).unwrap_or("(+ 34 35)".to_string());
    if env::args().any(|v| v.to_lowercase() == "--dump" || v.to_lowercase() == "-d") {
        let res = run_lisp_dumped(&source, "<provided>");
        if let Err(e) = res {
            println!("An error occurred: {e}");
            process::exit(1);
        }
    } else {
        let res = run_lisp(&source, "<provided>");
        if let Err(e) = res {
            println!("An error occurred: {e}");
            process::exit(1);
        }
    }
}
