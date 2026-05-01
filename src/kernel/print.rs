use num_bigint::BigInt;

use crate::ast::{Expr, Op};
use crate::kernel::term::Term;

#[derive(Debug)]
pub struct UnprintableError(pub String);

/// Lift a kernel term back into the surface AST. Binary applications whose
/// head matches a known infix operator become `BinOp`; other shapes have no
/// surface form yet and produce an error.
pub fn to_surface(t: &Term) -> Result<Expr, UnprintableError> {
    match t {
        Term::Nat(n) => Ok(Expr::Int(BigInt::from(n.clone()))),
        Term::Int(n) => Ok(Expr::Int(n.clone())),
        Term::Rat(r) => Ok(Expr::BinOp(
            Op::Div,
            Box::new(Expr::Int(r.numer().clone())),
            Box::new(Expr::Int(r.denom().clone())),
        )),
        Term::Var(s) => Ok(Expr::Ident(s.to_string())),
        Term::App(head, args) => match (op_for(head), args.as_slice()) {
            (Some(op), [l, r]) => Ok(Expr::BinOp(
                op,
                Box::new(to_surface(l)?),
                Box::new(to_surface(r)?),
            )),
            (Some(op), rest) if rest.len() > 2 => {
                let mut it = rest.iter();
                let mut acc = to_surface(it.next().unwrap())?;
                for a in it {
                    acc = Expr::BinOp(op, Box::new(acc), Box::new(to_surface(a)?));
                }
                Ok(acc)
            }
            _ => Err(UnprintableError(format!(
                "no surface form for application: head={head:?}, arity={}",
                args.len()
            ))),
        },
    }
}

fn op_for(head: &str) -> Option<Op> {
    match head {
        "+" => Some(Op::Add),
        "-" => Some(Op::Sub),
        "·" => Some(Op::Mul),
        "/" => Some(Op::Div),
        "^" => Some(Op::Pow),
        "=" => Some(Op::Eq),
        _ => None,
    }
}
