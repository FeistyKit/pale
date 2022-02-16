use crate::error::LispErrors;
use crate::types::LispType;
use crate::Location;
use crate::Var;
use std::fmt::Debug;
pub trait Callable: Debug {
    fn call(&self, args: &[Var], loc_called: &Location) -> Result<Var, LispErrors>;
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
                    if let LispType::Integer(i) = *a.resolve()?.get() {
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
                if let LispType::Integer(i) = *t.resolve()?.get() {
                    product = i
                } else {
                    return Err(LispErrors::new()
                        .error(loc_called, "Cannot multiply with non-integer type!"));
                }
                for a in args.iter().skip(1) {
                    if let LispType::Integer(i) = *a.resolve()?.get() {
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
                if let LispType::Integer(i) = *t.resolve()?.get() {
                    sum = i
                } else {
                    return Err(
                        LispErrors::new().error(loc_called, "Cannot subtract from a non-integer!")
                    );
                }
                for a in args.iter().skip(1) {
                    if let LispType::Integer(i) = *a.resolve()?.get() {
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
