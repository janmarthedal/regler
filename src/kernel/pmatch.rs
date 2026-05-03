use std::collections::HashMap;

use crate::kernel::term::{Symbol, Term};

/// Match `pat` against `t`, treating every `Var` in `pat` as a pattern
/// variable. On success returns a substitution that maps each pattern variable
/// to the subterm it bound to. Variables that occur multiple times in `pat`
/// must bind to syntactically equal terms.
///
/// Inside `Lam` binders, bound variables are tracked via an alpha-equivalence
/// map so they are never treated as wildcards.
pub fn pmatch(pat: &Term, t: &Term) -> Option<HashMap<Symbol, Term>> {
    pmatch_into(pat, t, HashMap::new(), &HashMap::new())
}

fn pmatch_into(
    pat: &Term,
    t: &Term,
    mut sigma: HashMap<Symbol, Term>,
    // Maps each lambda-bound variable name in `pat` to the corresponding bound
    // variable name in `t`. A Var in `pat` whose name appears here is NOT a
    // wildcard; it must match exactly the Var it is mapped to.
    bound: &HashMap<Symbol, Symbol>,
) -> Option<HashMap<Symbol, Term>> {
    match pat {
        Term::Var(x) => {
            if let Some(y) = bound.get(x) {
                // Bound variable: must match the corresponding variable in the subject.
                match t {
                    Term::Var(z) if z == y => Some(sigma),
                    _ => None,
                }
            } else {
                // Free pattern variable (wildcard).
                match sigma.get(x) {
                    Some(existing) if existing == t => Some(sigma),
                    Some(_) => None,
                    None => {
                        sigma.insert(x.clone(), t.clone());
                        Some(sigma)
                    }
                }
            }
        }
        Term::Nat(a) => match t {
            Term::Nat(b) if a == b => Some(sigma),
            _ => None,
        },
        Term::Int(a) => match t {
            Term::Int(b) if a == b => Some(sigma),
            _ => None,
        },
        Term::Rat(a) => match t {
            Term::Rat(b) if a == b => Some(sigma),
            _ => None,
        },
        Term::App(f, args) => match t {
            Term::App(g, args2) if f == g && args.len() == args2.len() => {
                let mut s = sigma;
                for (p, x) in args.iter().zip(args2.iter()) {
                    s = pmatch_into(p, x, s, bound)?;
                }
                Some(s)
            }
            _ => None,
        },
        Term::Lam(p, ty_pat, body_pat) => match t {
            Term::Lam(q, ty_subj, body_subj) => {
                // Match type annotations first (in the outer scope).
                let s = pmatch_into(ty_pat, ty_subj, sigma, bound)?;
                // Extend the bound-variable map: p in pat corresponds to q in subj.
                let mut new_bound = bound.clone();
                new_bound.insert(p.clone(), q.clone());
                pmatch_into(body_pat, body_subj, s, &new_bound)
            }
            _ => None,
        },
    }
}
