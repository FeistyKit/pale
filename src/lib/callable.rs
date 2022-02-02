use crate::ast::TypeError;
use crate::types::LispType;
use crate::Location;
use crate::Var;
use std::fmt::Debug;
pub trait Callable: Debug {
    // TODO(#5): Decide whether to keep the return type of Callable::call as a trait object or an
    // associated type
    fn call(
        &self,
        args: &Vec<Var>,
        loc_called: &Location,
    ) -> Result<Var, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum IntrinsicOp {
    Add,
    Subtract,
    Print,
    Multiply,
}

impl Callable for IntrinsicOp {
    fn call(
        &self,
        args: &Vec<Var>,
        loc_called: &Location,
    ) -> Result<Var, Box<dyn std::error::Error>> {
        match self {
            IntrinsicOp::Add => {
                if args.len() < 2 {
                    println!("{} - Addition requires at least two arguments!", loc_called);
                }
                // TODO(#11): Addition of floats and integers.
                let mut sum = 0;
                for a in args {
                    if let LispType::Integer(i) = *a.resolve()?.get() {
                        sum += i;
                    } else {
                        // TODO(#4): Better error reporting in Statement::resolve with incorrect types
                        return Err(TypeError::new(format!(
                            "Cannot add a non-integer type to an integer: {}!",
                            a.get()
                        )));
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
                if let LispType::Integer(i) = *t.resolve()?.get() {
                    product = i
                } else {
                    return Err(TypeError::new("Cannot multiply with a non-integer type!"));
                }
                for a in args.into_iter().skip(1) {
                    if let LispType::Integer(i) = *a.resolve()?.get() {
                        product *= i;
                    } else {
                        return Err(TypeError::new(
                            "Cannot multiply a non-integer type with an integer!",
                        ));
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
                if let LispType::Integer(i) = *t.resolve()?.get() {
                    sum = i
                } else {
                    return Err(TypeError::new("Cannot subtract from a non-integer!"));
                }
                for a in args.into_iter().skip(1) {
                    if let LispType::Integer(i) = *a.resolve()?.get() {
                        sum -= i;
                    } else {
                        return Err(TypeError::new(
                            "Cannot subtract a non-integer type from an integer!",
                        ));
                    }
                }
                Ok(Var::new(sum))
            }
            IntrinsicOp::Print => {
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
