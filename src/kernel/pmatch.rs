use std::collections::{HashMap, HashSet};

use crate::kernel::term::{Symbol, Term};

/// Match `pat` against `t`, treating every `Var` in `pat` as a pattern
/// variable. On success returns a substitution that maps each pattern variable
/// to the subterm it bound to. Variables that occur multiple times in `pat`
/// must bind to syntactically equal terms.
///
/// Inside `Lam` binders, bound variables are tracked via an alpha-equivalence
/// map so they are never treated as wildcards.
///
/// Higher-order matching: if a `Var` appears in function-head position
/// (`App(Var("f"), args)`), it matches any subject head of the same arity,
/// binding `f` to that head term. This enables patterns like `D(f(x))` where
/// `D` is concrete and `f` is a higher-order pattern variable.
///
/// Occurs-check: a free pattern variable cannot be bound to a term that
/// contains a lambda-bound subject variable (would capture the binding).
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
        Term::Sym(a) => match t {
            Term::Sym(b) if a == b => Some(sigma),
            _ => None,
        },
        Term::Var(x) => {
            if let Some(y) = bound.get(x) {
                // Bound variable: must match the corresponding variable in the subject.
                match t {
                    Term::Var(z) if z == y => Some(sigma),
                    _ => None,
                }
            } else {
                // Free pattern variable (wildcard).
                // Occurs-check: reject if t contains any subject-side bound variable.
                if contains_bound_subj_var(t, bound) {
                    return None;
                }
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
        Term::App(pat_head, pat_args) => match t {
            Term::App(subj_head, subj_args) if pat_args.len() == subj_args.len() => {
                let s = pmatch_into(pat_head, subj_head, sigma, bound)?;
                let mut s = s;
                for (p, x) in pat_args.iter().zip(subj_args.iter()) {
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

/// Returns true if `t` contains any `Var(s)` where `s` is a subject-side bound
/// variable (i.e., appears in `bound.values()`). Used for the occurs-check when
/// binding a free pattern variable: binding it to such a term would capture the
/// bound variable outside its scope.
fn contains_bound_subj_var(t: &Term, bound: &HashMap<Symbol, Symbol>) -> bool {
    if bound.is_empty() {
        return false;
    }
    let bound_subj: HashSet<&Symbol> = bound.values().collect();
    term_contains_any_var(t, &bound_subj)
}

fn term_contains_any_var(t: &Term, vars: &HashSet<&Symbol>) -> bool {
    match t {
        Term::Var(s) => vars.contains(s),
        Term::Sym(_) | Term::Nat(_) | Term::Int(_) | Term::Rat(_) => false,
        Term::App(head, args) => {
            term_contains_any_var(head, vars)
                || args.iter().any(|a| term_contains_any_var(a, vars))
        }
        Term::Lam(_, ty, body) => {
            term_contains_any_var(ty, vars) || term_contains_any_var(body, vars)
        }
    }
}
