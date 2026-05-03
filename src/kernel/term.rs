use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use std::rc::Rc;

pub type Symbol = Rc<str>;

/// The variant order matters: it defines the kernel's canonical total order on
/// terms (`App < Var < Lam < Nat < Int < Rat`), which `simplify` uses to sort AC
/// operands into a canonical form. Apps come first, then variables and lambdas,
/// then literals — this puts numeric constants last in printed output (`a + 5`,
/// not `5 + a`). Within a variant, the derived order falls back to field comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    App(Symbol, Vec<Term>),
    Var(Symbol),
    /// `(param : ty) ↦ body` — anonymous function abstraction
    Lam(Symbol, Box<Term>, Box<Term>),
    Nat(BigUint),
    Int(BigInt),
    Rat(BigRational),
}

pub fn sym(s: &str) -> Symbol {
    Rc::from(s)
}
