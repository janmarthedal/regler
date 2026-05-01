use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};

use crate::kernel::eval::{rat_to_term, term_to_rat};
use crate::kernel::pmatch::pmatch;
use crate::kernel::subst::subst;
use crate::kernel::term::{Symbol, Term};
use crate::kernel::theory::Theory;

pub use crate::kernel::theory::{orient, Orient, Rule};

/// Reduce `t` to a normal form against `theory`. The fixed-point loop combines
/// three strictly-decreasing reductions:
/// 1. closed literal arithmetic on ℕ/ℤ/ℚ for `+`, `-`, `·`, `/`, `^`,
/// 2. AC normalization (flatten + sort + drop identity operands + fold literal
///    operands) for any operator the theory has promoted to AC,
/// 3. binary identity-element absorption for non-AC operators that have an
///    identity fact registered,
/// 4. KBO-oriented rewrite rules (with optional side condition checked before
///    firing).
/// Each reduction strictly decreases either the KBO weight or the operand
/// count, so the loop terminates.
pub fn simplify(t: &Term, theory: &Theory) -> Term {
    let t1 = match t {
        Term::Nat(_) | Term::Var(_) | Term::Int(_) | Term::Rat(_) => t.clone(),
        Term::App(head, args) => {
            let new_args: Vec<Term> = args.iter().map(|a| simplify(a, theory)).collect();
            let folded = arith_fold(head, new_args);
            normalize_app(folded, theory)
        }
    };
    for r in &theory.rules {
        if let Some(sigma) = pmatch(&r.lhs, &t1) {
            if let Some(cond) = &r.condition {
                let cond_inst = subst(cond, &sigma);
                if !condition_holds(&cond_inst) {
                    continue;
                }
            }
            let t2 = subst(&r.rhs, &sigma);
            return simplify(&t2, theory);
        }
    }
    t1
}

/// Apply the equality `lhs = rhs` as a single rewrite step to `target`,
/// trying the top level first, then descending leftmost-outermost until the
/// first matching subterm is found and rewritten. Returns `None` if no
/// subterm matches.
pub fn apply_eq(lhs: &Term, rhs: &Term, target: &Term) -> Option<Term> {
    if let Some(sigma) = pmatch(lhs, target) {
        return Some(subst(rhs, &sigma));
    }
    match target {
        Term::App(head, args) => {
            for (i, arg) in args.iter().enumerate() {
                if let Some(rewritten) = apply_eq(lhs, rhs, arg) {
                    let mut new_args = args.clone();
                    new_args[i] = rewritten;
                    return Some(Term::App(head.clone(), new_args));
                }
            }
            None
        }
        _ => None,
    }
}

/// Evaluate a condition term (after substituting the match) and return whether
/// it is verifiably true. Conservatively returns `false` when the condition
/// contains unresolved variables — the rule does not fire.
fn condition_holds(t: &Term) -> bool {
    match t {
        Term::App(head, args) if args.len() == 2 => {
            let a = term_to_rat(&args[0]);
            let b = term_to_rat(&args[1]);
            match (head.as_ref(), a, b) {
                ("≠", Some(a), Some(b)) => a != b,
                _ => false,
            }
        }
        _ => false,
    }
}

fn normalize_app(t: Term, theory: &Theory) -> Term {
    let (head, args) = match t {
        Term::App(head, args) => (head, args),
        other => return other,
    };
    if theory.is_ac(&head) {
        ac_normalize(&head, args, theory)
    } else {
        identity_drop_binary(head, args, theory)
    }
}

fn ac_normalize(head: &Symbol, args: Vec<Term>, theory: &Theory) -> Term {
    let mut flat: Vec<Term> = Vec::with_capacity(args.len());
    for a in args {
        match a {
            Term::App(h, sub) if &h == head => flat.extend(sub),
            other => flat.push(other),
        }
    }

    let identity = theory.right_identity(head).cloned();
    if let Some(id) = &identity {
        flat.retain(|x| x != id);
    }

    fold_literals(head, &mut flat);

    flat.sort();

    match flat.len() {
        0 => identity.unwrap_or(Term::App(head.clone(), Vec::new())),
        1 => flat.into_iter().next().unwrap(),
        _ => Term::App(head.clone(), flat),
    }
}

fn fold_literals(head: &Symbol, flat: &mut Vec<Term>) {
    let is_add = head.as_ref() == "+";
    let is_mul = head.as_ref() == "·";
    if !is_add && !is_mul {
        return;
    }
    let identity: BigRational = if is_add {
        BigRational::zero()
    } else {
        BigRational::one()
    };

    let mut acc: Option<BigRational> = None;
    flat.retain(|x| {
        if let Some(r) = term_to_rat(x) {
            acc = Some(match acc.take() {
                Some(a) => if is_add { a + r } else { a * r },
                None => r,
            });
            false
        } else {
            true
        }
    });
    if let Some(r) = acc {
        if r != identity || flat.is_empty() {
            flat.push(rat_to_term(r));
        }
    }
}

fn identity_drop_binary(head: Symbol, args: Vec<Term>, theory: &Theory) -> Term {
    if args.len() == 2 {
        if let Some(e) = theory.right_identity(&head) {
            if &args[1] == e {
                return args.into_iter().next().unwrap();
            }
        }
        if let Some(e) = theory.left_identity(&head) {
            if &args[0] == e {
                return args.into_iter().nth(1).unwrap();
            }
        }
    }
    Term::App(head, args)
}

fn arith_fold(head: &Symbol, args: Vec<Term>) -> Term {
    if args.len() == 2 {
        match head.as_ref() {
            "+" | "-" | "·" | "/" => {
                if let (Some(a), Some(b)) = (term_to_rat(&args[0]), term_to_rat(&args[1])) {
                    let result = match head.as_ref() {
                        "+" => a + b,
                        "-" => a - b,
                        "·" => a * b,
                        "/" if !b.is_zero() => a / b,
                        _ => return Term::App(head.clone(), args),
                    };
                    return rat_to_term(result);
                }
            }
            "^" => {
                if let (Term::Nat(a), Term::Nat(b)) = (&args[0], &args[1]) {
                    if let Some(e) = b.to_u32() {
                        return Term::Nat(a.pow(e));
                    }
                }
            }
            _ => {}
        }
    }
    Term::App(head.clone(), args)
}
