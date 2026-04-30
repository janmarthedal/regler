use num_bigint::{BigInt, Sign};

use crate::ast::Expr;
use crate::kernel::term::{sym, Term};

#[derive(Debug)]
pub struct LowerError(pub String);

pub fn lower(e: &Expr) -> Result<Term, LowerError> {
    match e {
        Expr::Ident(s) => Ok(Term::Var(sym(s))),
        Expr::Int(n) => to_nat(n).map(Term::Nat),
        Expr::BinOp(op, l, r) => {
            let l = lower(l)?;
            let r = lower(r)?;
            Ok(Term::App(sym(op.symbol()), vec![l, r]))
        }
    }
}

fn to_nat(n: &BigInt) -> Result<num_bigint::BigUint, LowerError> {
    match n.sign() {
        Sign::Minus => Err(LowerError(format!(
            "negative integer literals are not yet supported (got {n})"
        ))),
        _ => Ok(n.magnitude().clone()),
    }
}
