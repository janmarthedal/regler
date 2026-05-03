use std::collections::HashMap;

use crate::kernel::term::{Symbol, Term};

/// Substitute every free variable in `t` whose symbol appears in `sigma` with
/// the corresponding replacement term, recursing into application arguments.
pub fn subst(t: &Term, sigma: &HashMap<Symbol, Term>) -> Term {
    match t {
        Term::Nat(_) | Term::Int(_) | Term::Rat(_) => t.clone(),
        Term::Var(s) => match sigma.get(s) {
            Some(replacement) => replacement.clone(),
            None => t.clone(),
        },
        Term::App(head, args) => {
            let new_args = args.iter().map(|a| subst(a, sigma)).collect();
            Term::App(head.clone(), new_args)
        }
        // The bound variable shadows any substitution for the same name.
        Term::Lam(x, ty, body) => {
            let ty2 = subst(ty, sigma);
            let mut inner = sigma.clone();
            inner.remove(x);
            Term::Lam(x.clone(), Box::new(ty2), Box::new(subst(body, &inner)))
        }
    }
}
