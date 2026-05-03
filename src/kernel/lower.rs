use std::collections::HashSet;

use num_bigint::Sign;

use crate::ast::{Expr, UnaryOp};
use crate::kernel::term::{sym, Term};

#[derive(Debug)]
pub struct LowerError(pub String);

/// Translate a surface AST into the kernel's uniform-prefix `Term`
/// representation. Binary operators become applications keyed by the operator
/// symbol; function application becomes `App`; non-negative integer literals
/// become `Nat`, negative ones `Int`. `∀` binders are stripped — the body is
/// lowered directly, with variables remaining as `Term::Var` pattern variables.
/// Domain annotations and set-builder expressions cannot appear as terms.
pub fn lower(e: &Expr) -> Result<Term, LowerError> {
    match e {
        Expr::Ident(s) => Ok(Term::Var(sym(s))),
        Expr::Int(n) => match n.sign() {
            Sign::Minus => Ok(Term::Int(n.clone())),
            _ => Ok(Term::Nat(n.magnitude().clone())),
        },
        Expr::App(f, args) => {
            let term_args: Result<Vec<_>, _> = args.iter().map(lower).collect();
            Ok(Term::App(Box::new(Term::Sym(sym(f))), term_args?))
        }
        Expr::BinOp(op, l, r) => {
            let l = lower(l)?;
            let r = lower(r)?;
            Ok(Term::App(Box::new(Term::Sym(sym(op.symbol()))), vec![l, r]))
        }
        Expr::UnaryOp(UnaryOp::Neg, e) => {
            Ok(Term::App(Box::new(Term::Sym(sym("-"))), vec![lower(e)?]))
        }
        Expr::Lambda(param, ty, body) => {
            let ty_term = lower(ty)?;
            let body_term = lower(body)?;
            Ok(Term::Lam(sym(param), Box::new(ty_term), Box::new(body_term)))
        }
        Expr::Forall(_vars, _domain, body) => lower(body),
        Expr::SetBuilder(_, _, _) => {
            Err(LowerError("set-builder expressions cannot be used as terms".into()))
        }
    }
}

/// Lower a fact body, distinguishing declared constants from pattern variables.
///
/// - Identifiers in `pvars` (bound by the outermost `∀`) → `Term::Var` (pattern wildcards).
/// - Identifiers in `known` (declared with `let`) and not in `pvars` → `Term::Sym` (constants).
/// - All other identifiers → `Term::Var` (implicit pattern wildcards).
/// - Function application `f(args)` where `f` is in `pvars` or unknown →
///   `App(Var(f), args)`, enabling higher-order pattern matching.
///
/// `∀` sub-binders in the body add their bound variables to `pvars` for the recursive call.
pub fn lower_fact_body(
    e: &Expr,
    pvars: &HashSet<String>,
    known: &HashSet<String>,
) -> Result<Term, LowerError> {
    lower_with(e, pvars, known)
}

fn lower_with(e: &Expr, pvars: &HashSet<String>, known: &HashSet<String>) -> Result<Term, LowerError> {
    match e {
        Expr::Ident(s) => {
            if pvars.contains(s) || !known.contains(s) {
                Ok(Term::Var(sym(s)))
            } else {
                Ok(Term::Sym(sym(s)))
            }
        }
        Expr::Int(n) => match n.sign() {
            Sign::Minus => Ok(Term::Int(n.clone())),
            _ => Ok(Term::Nat(n.magnitude().clone())),
        },
        Expr::App(f, args) => {
            let term_args: Result<Vec<_>, _> = args.iter().map(|a| lower_with(a, pvars, known)).collect();
            // Head is a pvar or unknown identifier → Var (wildcard head for higher-order matching).
            // Head is a known constant and not a pvar → Sym (concrete, non-wildcard).
            let head = if pvars.contains(f) || !known.contains(f) {
                Term::Var(sym(f))
            } else {
                Term::Sym(sym(f))
            };
            Ok(Term::App(Box::new(head), term_args?))
        }
        Expr::BinOp(op, l, r) => {
            let l = lower_with(l, pvars, known)?;
            let r = lower_with(r, pvars, known)?;
            Ok(Term::App(Box::new(Term::Sym(sym(op.symbol()))), vec![l, r]))
        }
        Expr::UnaryOp(UnaryOp::Neg, e) => {
            Ok(Term::App(Box::new(Term::Sym(sym("-"))), vec![lower_with(e, pvars, known)?]))
        }
        Expr::Lambda(param, ty, body) => {
            // The type annotation is lowered in the outer scope (param not yet bound).
            let ty_term = lower_with(ty, pvars, known)?;
            // The param is a bound variable inside the body: add it to pvars so it
            // becomes Term::Var (a matchable name) rather than a Sym constant.
            let mut new_pvars = pvars.clone();
            new_pvars.insert(param.clone());
            let body_term = lower_with(body, &new_pvars, known)?;
            Ok(Term::Lam(sym(param), Box::new(ty_term), Box::new(body_term)))
        }
        Expr::Forall(vars, _domain, body) => {
            let mut new_pvars = pvars.clone();
            for v in vars {
                new_pvars.insert(v.clone());
            }
            lower_with(body, &new_pvars, known)
        }
        Expr::SetBuilder(_, _, _) => {
            Err(LowerError("set-builder expressions cannot be used as terms".into()))
        }
    }
}
