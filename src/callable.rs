use crate::ast::Statement;
use crate::error::LispErrors;
use crate::types::LispValue;
use crate::Location;
use crate::Var;
use std::fmt::Debug;

pub trait Callable {
    fn call(&self, args: &[Var], loc_called: &Location) -> Result<Var, LispErrors>;
    fn try_clone(&self) -> Option<Box<dyn Callable>> {
        None
    }
    fn maybe_debug_info(&self) -> Option<String> {
        None
    }
}

impl<T: Clone + 'static + Fn(&[Var], &Location) -> Result<Var, LispErrors>> Callable for T {
    fn call(&self, args: &[Var], loc: &Location) -> Result<Var, LispErrors> {
        self(args, loc)
    }
    fn try_clone(&self) -> Option<Box<dyn Callable>> {
        Some(Box::new(self.clone()) as Box<dyn Callable>)
    }
}

// TODO: Automatically implement Callable for types that don't support Clone.
//
//
// impl<T: !Clone + Fn(&[Var], &Location) -> Result<Var, LispErrors>> Callable for T {
//     fn call(&self, args: &[Var], loc: &Location) -> Result<Var, LispErrors> {
//         self(args, loc)
//     }
// }

#[derive(Debug)]
pub(crate) struct Function {
    vars: Vec<Var>, // The statement depends upon the vars
    dat: Statement,
}

impl Callable for Function {
    fn call(&self, args: &[Var], loc_called: &Location) -> Result<Var, LispErrors> {
        if args.len() < self.vars.len() {
            return Err(LispErrors::new().error(loc_called, "Insufficient arguments provided!"));
        } else if args.len() > self.vars.len() {
            return Err(LispErrors::new()
                .error(loc_called, "Too many arguments provided!")
                .note(loc_called, "Delete them"));
        }
        for (arg, var) in args.iter().zip(self.vars.iter()) {
            *var.get_mut() = LispValue::Var(arg.maybe_clone())
        }
        self.dat.resolve()
    }
}

impl Function {
    pub(crate) fn new(vars: Vec<Var>, dat: Statement) -> Self {
        Function { vars, dat }
    }
}

#[derive(Debug)]
pub enum IntrinsicOp {
    Add,
    Subtract,
    Print,
    Multiply,
}

impl Callable for IntrinsicOp {
    fn call(&self, args: &[Var], loc_called: &Location) -> Result<Var, LispErrors> {
        match self {
            IntrinsicOp::Add => {
                if args.len() < 2 {
                    println!("{} - Addition requires at least two arguments!", loc_called);
                }
                // TODO(#11): Addition of floats and integers.
                let mut sum = 0;
                for a in args {
                    if let LispValue::Integer(i) = *a.resolve()?.get() {
                        sum += i;
                    } else {
                        return Err(LispErrors::new().error(
                            loc_called,
                            format!("Incompatible types for addition: Integer and {}", a.get()),
                        ));
                    }
                }
                Ok(Var::new(sum))
            }
            IntrinsicOp::Multiply => {
                if args.len() < 2 {
                    println!(
                        "{} - Multiplication requires at least two arguments!",
                        loc_called
                    );
                }
                let mut product;
                let t = args.get(0).unwrap();
                if let LispValue::Integer(i) = *t.resolve()?.get() {
                    product = i
                } else {
                    return Err(LispErrors::new()
                        .error(loc_called, "Cannot multiply with non-integer type!"));
                }
                for a in args.iter().skip(1) {
                    if let LispValue::Integer(i) = *a.resolve()?.get() {
                        product *= i;
                    } else {
                        return Err(LispErrors::new()
                            .error(loc_called, "Cannot multiply with non-integer type!"));
                    }
                }
                Ok(Var::new(product))
            }
            IntrinsicOp::Subtract => {
                if args.len() < 2 {
                    println!(
                        "{} - Subtraction requires at least two arguments!",
                        loc_called
                    );
                }
                let mut sum;
                let t = args.get(0).unwrap();
                if let LispValue::Integer(i) = *t.resolve()?.get() {
                    sum = i
                } else {
                    return Err(
                        LispErrors::new().error(loc_called, "Cannot subtract from a non-integer!")
                    );
                }
                for a in args.iter().skip(1) {
                    if let LispValue::Integer(i) = *a.resolve()?.get() {
                        sum -= i;
                    } else {
                        return Err(LispErrors::new().error(
                            loc_called,
                            "Cannot subtract a non-integer type from an integer!",
                        ));
                    }
                }
                Ok(Var::new(sum))
            }
            IntrinsicOp::Print => {
                if args.len() != 1 {
                    Err(LispErrors::new()
                        .error(loc_called, "Print intrinsic requires only one argument!")
                        .note(None, "Try wrapping this in a statement with `$`."))
                } else {
                    println!("{}", args[0]);
                    Ok(Var::new(0))
                }
            }
        }
    }
}
