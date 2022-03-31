# Perhaps Another Lisp Engine

Pale is a homemade, interpreted lisp, written in Rust. I made it while following [this guide on interpreters](http://craftinginterpreters.com).

Pale isn't meant to be a true production-ready language. It's meant to be used for very small scripts where a small amount of logic is needed, especially in an isolated environment (e.g. a Discord bot). Another factor of its design is that I want it to be easily used to allow users to define logic, but you ultimately get full control over what they can or can't do. 

## Goals
 - [ ] As few dependencies as possible (for the library at least).
 - [ ] Easily put into any project.
   - [ ] Well documented.
 - [ ] Moderately fast
 - [ ] Simple to manipulate the functions provided.
 
## Using Pale in your project
The main part of this repository is a library, not the interpreter, so it can be added to any Rust project simply by putting it into your Cargo.toml like so:
``` toml
[dependencies]
pale = { git = "https://www.github.com/FeistyKit/pale" }
```

## Running the Pale interpreter
```bash
$ git clone https://www.github.com/FeistyKit/pale.git
$ cd pale
$ make
$ ./pale "(print \"Hello, World!\")"
```

## Requirements
Requires [rust](https://rustup.rs/) installed.

## Documentation
Documentation can be found in the [doc folder](./doc).
