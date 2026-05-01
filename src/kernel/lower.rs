use num_bigint::Sign;

use crate::ast::{Expr, UnaryOp};
use crate::kernel::term::{sym, Term};

#[derive(Debug)]
pub struct LowerError(pub String);

/// Translate a surface AST into the kernel's uniform-prefix `Term`
/// representation. Binary operators become applications keyed by the operator
/// symbol; non-negative integer literals become `Nat`, negative ones `Int`.
/// `∀` binders are stripped — the body is lowered directly, with variables
/// remaining as `Term::Var` pattern variables. Domain annotations are ignored
/// until sets are first-class in the kernel.
pub fn lower(e: &Expr) -> Result<Term, LowerError> {
    match e {
        Expr::Ident(s) => Ok(Term::Var(sym(s))),
        Expr::Int(n) => match n.sign() {
            Sign::Minus => Ok(Term::Int(n.clone())),
            _ => Ok(Term::Nat(n.magnitude().clone())),
        },
        Expr::BinOp(op, l, r) => {
            let l = lower(l)?;
            let r = lower(r)?;
            Ok(Term::App(sym(op.symbol()), vec![l, r]))
        }
        Expr::UnaryOp(UnaryOp::Neg, e) => {
            Ok(Term::App(sym("-"), vec![lower(e)?]))
        }
        Expr::Forall(_vars, _domain, body) => lower(body),
    }
}
