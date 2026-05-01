use std::collections::HashMap;

use crate::kernel::term::{Symbol, Term};

/// Match `pat` against `t`, treating every `Var` in `pat` as a pattern
/// variable. On success returns a substitution that maps each pattern variable
/// to the subterm it bound to. Variables that occur multiple times in `pat`
/// must bind to syntactically equal terms.
pub fn pmatch(pat: &Term, t: &Term) -> Option<HashMap<Symbol, Term>> {
    pmatch_into(pat, t, HashMap::new())
}

fn pmatch_into(
    pat: &Term,
    t: &Term,
    mut sigma: HashMap<Symbol, Term>,
) -> Option<HashMap<Symbol, Term>> {
    match pat {
        Term::Var(x) => match sigma.get(x) {
            Some(existing) if existing == t => Some(sigma),
            Some(_) => None,
            None => {
                sigma.insert(x.clone(), t.clone());
                Some(sigma)
            }
        },
        Term::Nat(a) => match t {
            Term::Nat(b) if a == b => Some(sigma),
            _ => None,
        },
        Term::App(f, args) => match t {
            Term::App(g, args2) if f == g && args.len() == args2.len() => {
                let mut s = sigma;
                for (p, x) in args.iter().zip(args2.iter()) {
                    s = pmatch_into(p, x, s)?;
                }
                Some(s)
            }
            _ => None,
        },
    }
}
