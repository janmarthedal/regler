use num_bigint::BigInt;

use crate::ast::{Expr, Op};
use crate::kernel::term::Term;

#[derive(Debug)]
pub struct UnprintableError(pub String);

pub fn to_surface(t: &Term) -> Result<Expr, UnprintableError> {
    match t {
        Term::Nat(n) => Ok(Expr::Int(BigInt::from(n.clone()))),
        Term::Var(s) => Ok(Expr::Ident(s.to_string())),
        Term::App(head, args) => match (op_for(head), args.as_slice()) {
            (Some(op), [l, r]) => Ok(Expr::BinOp(
                op,
                Box::new(to_surface(l)?),
                Box::new(to_surface(r)?),
            )),
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
        "·" => Some(Op::Mul),
        "^" => Some(Op::Pow),
        "=" => Some(Op::Eq),
        _ => None,
    }
}
