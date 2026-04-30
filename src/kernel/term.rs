use num_bigint::BigUint;
use std::rc::Rc;

pub type Symbol = Rc<str>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Nat(BigUint),
    Var(Symbol),
    App(Symbol, Vec<Term>),
}

pub fn sym(s: &str) -> Symbol {
    Rc::from(s)
}
