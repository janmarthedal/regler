use std::collections::HashMap;

use crate::kernel::term::{Symbol, Term};

pub fn subst(t: &Term, sigma: &HashMap<Symbol, Term>) -> Term {
    match t {
        Term::Nat(_) => t.clone(),
        Term::Var(s) => match sigma.get(s) {
            Some(replacement) => replacement.clone(),
            None => t.clone(),
        },
        Term::App(head, args) => {
            let new_args = args.iter().map(|a| subst(a, sigma)).collect();
            Term::App(head.clone(), new_args)
        }
    }
}
