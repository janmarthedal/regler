//! Knuth-Bendix order on kernel terms.
//!
//! KBO is a well-founded simplification order: `s > t` implies every rewrite
//! step `s → t` strictly decreases the term, so any rewrite system whose rules
//! are oriented by KBO terminates.
//!
//! How it is used here:
//!
//! - **Auto-orientation of equalities.** When a `fact l = r` is installed
//!   (see `kernel::rewrite::orient`), KBO compares `l` and `r`. The strictly
//!   larger side becomes the lhs of a rewrite rule; the smaller side becomes
//!   the rhs. The user does not pick a direction — the order does.
//! - **Filtering unsound equalities.** KBO's variable-count condition
//!   (`#x(s) ≥ #x(t)` for every variable `x`) means an equality whose rhs has
//!   a variable not on the lhs (e.g. `x = y`) is incomparable in either
//!   direction and is rejected as a rewrite. This is exactly the soundness
//!   condition a rewrite rule needs.
//! - **Detecting incomparable equalities.** Symmetric equalities such as
//!   commutativity `a + b = b + a` are KBO-incomparable; they are stored as
//!   facts but install no auto-rule, so `simplify` cannot loop on them.
//!   These will be handled later by AC recognition (milestone 5).
//! - **Termination of `simplify`.** Each rewrite step is `lhs → rhs` with
//!   `lhs > rhs` in KBO; literal arithmetic also strictly decreases weight.
//!   Together, the fixed-point loop in `simplify` is guaranteed to halt.
//!
//! Parameters chosen here: every symbol — variables, numeric literals, and
//! application heads — has weight 1. Precedence on App heads ranks the four
//! built-ins as `= < + < · < ^` (mirroring surface precedence); other heads
//! fall back to byte-wise string order and rank above the built-ins.

use std::collections::HashSet;

use crate::kernel::term::{Symbol, Term};

/// Result of a KBO comparison. `Incomparable` means neither term dominates the
/// other under the order — the equality cannot be auto-oriented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KboOrd {
    Lt,
    Eq,
    Gt,
    Incomparable,
}

/// Compare two kernel terms under the Knuth-Bendix order with default weight 1
/// for every symbol, every variable, and every numeric literal.
pub fn kbo(s: &Term, t: &Term) -> KboOrd {
    if s == t {
        return KboOrd::Eq;
    }
    let s_gt = kbo_gt(s, t);
    let t_gt = kbo_gt(t, s);
    match (s_gt, t_gt) {
        (true, _) => KboOrd::Gt,
        (false, true) => KboOrd::Lt,
        (false, false) => KboOrd::Incomparable,
    }
}

fn kbo_gt(s: &Term, t: &Term) -> bool {
    let mut t_vars = HashSet::new();
    collect_vars(t, &mut t_vars);
    for v in &t_vars {
        if var_count(s, v) < var_count(t, v) {
            return false;
        }
    }
    let ws = weight(s);
    let wt = weight(t);
    if ws > wt {
        return true;
    }
    if ws < wt {
        return false;
    }
    match (s, t) {
        (Term::Var(_), _) | (_, Term::Var(_)) => false,
        (Term::Nat(a), Term::Nat(b)) => a > b,
        (Term::App(f, sa), Term::App(g, ta)) if f == g && sa.len() == ta.len() => {
            for (si, ti) in sa.iter().zip(ta.iter()) {
                if si == ti {
                    continue;
                }
                return kbo_gt(si, ti);
            }
            false
        }
        (Term::App(f, _), Term::App(g, _)) => prec_gt(f, g),
        (Term::App(_, _), Term::Nat(_)) => true,
        (Term::Nat(_), Term::App(_, _)) => false,
    }
}

fn weight(t: &Term) -> u64 {
    match t {
        Term::Nat(_) | Term::Var(_) => 1,
        Term::App(_, args) => 1 + args.iter().map(weight).sum::<u64>(),
    }
}

fn var_count(t: &Term, x: &Symbol) -> u64 {
    match t {
        Term::Nat(_) => 0,
        Term::Var(s) => {
            if s == x {
                1
            } else {
                0
            }
        }
        Term::App(_, args) => args.iter().map(|a| var_count(a, x)).sum(),
    }
}

fn collect_vars(t: &Term, out: &mut HashSet<Symbol>) {
    match t {
        Term::Nat(_) => {}
        Term::Var(s) => {
            out.insert(s.clone());
        }
        Term::App(_, args) => {
            for a in args {
                collect_vars(a, out);
            }
        }
    }
}

/// Strict precedence on App heads. The four built-in arithmetic/equality
/// operators are ordered `= < + < · < ^`, mirroring surface precedence.
/// Unknown heads fall back to byte-wise string comparison and are placed
/// above the built-ins so that user-introduced operators do not perturb
/// the orientation of arithmetic facts.
fn prec_gt(f: &Symbol, g: &Symbol) -> bool {
    let pf = builtin_prec(f);
    let pg = builtin_prec(g);
    match (pf, pg) {
        (Some(a), Some(b)) => a > b,
        (Some(_), None) => false,
        (None, Some(_)) => true,
        (None, None) => f.as_ref() > g.as_ref(),
    }
}

fn builtin_prec(s: &Symbol) -> Option<u8> {
    match s.as_ref() {
        "=" => Some(0),
        "+" => Some(1),
        "·" => Some(2),
        "^" => Some(3),
        _ => None,
    }
}
