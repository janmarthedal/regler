use num_traits::ToPrimitive;

use crate::kernel::kbo::{kbo, KboOrd};
use crate::kernel::pmatch::pmatch;
use crate::kernel::subst::subst;
use crate::kernel::term::{Symbol, Term};

/// A rewrite rule oriented by KBO: `lhs` strictly dominates `rhs`, so every
/// rewrite step strictly decreases the term in the order. Variables in `lhs`
/// act as pattern variables; KBO orientation guarantees every variable in
/// `rhs` also appears in `lhs`.
#[derive(Debug, Clone)]
pub struct Rule {
    pub lhs: Term,
    pub rhs: Term,
}

/// Outcome of trying to install an equality `l = r` as a rewrite rule.
#[derive(Debug)]
pub enum Orient {
    Rule(Rule),
    Trivial,
    Incomparable,
}

/// Attempt to orient an equality into a rewrite rule by KBO. The larger side
/// becomes the lhs. Equalities whose two sides are KBO-incomparable cannot be
/// auto-applied and are reported as such; trivial equalities (`l = l`) are
/// reported separately.
pub fn orient(l: &Term, r: &Term) -> Orient {
    match kbo(l, r) {
        KboOrd::Gt => Orient::Rule(Rule {
            lhs: l.clone(),
            rhs: r.clone(),
        }),
        KboOrd::Lt => Orient::Rule(Rule {
            lhs: r.clone(),
            rhs: l.clone(),
        }),
        KboOrd::Eq => Orient::Trivial,
        KboOrd::Incomparable => Orient::Incomparable,
    }
}

/// Reduce `t` to a normal form by repeatedly applying any of `rules` whose
/// lhs matches a subterm, and folding closed literal arithmetic on ℕ. Both
/// reductions strictly decrease the KBO weight of the term, so this loop
/// terminates.
pub fn simplify(t: &Term, rules: &[Rule]) -> Term {
    let t1 = match t {
        Term::Nat(_) | Term::Var(_) => t.clone(),
        Term::App(head, args) => {
            let new_args: Vec<Term> = args.iter().map(|a| simplify(a, rules)).collect();
            arith_fold(head, new_args)
        }
    };
    for r in rules {
        if let Some(sigma) = pmatch(&r.lhs, &t1) {
            let t2 = subst(&r.rhs, &sigma);
            return simplify(&t2, rules);
        }
    }
    t1
}

fn arith_fold(head: &Symbol, args: Vec<Term>) -> Term {
    if args.len() == 2 {
        if let (Term::Nat(a), Term::Nat(b)) = (&args[0], &args[1]) {
            match head.as_ref() {
                "+" => return Term::Nat(a + b),
                "·" => return Term::Nat(a * b),
                "^" => {
                    if let Some(e) = b.to_u32() {
                        return Term::Nat(a.pow(e));
                    }
                }
                _ => {}
            }
        }
    }
    Term::App(head.clone(), args)
}
