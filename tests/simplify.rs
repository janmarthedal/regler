use std::collections::HashMap;

use regler::kernel::kbo::{kbo, KboOrd};
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{orient, simplify, Orient, Rule};
use regler::kernel::subst::subst;
use regler::kernel::term::{Symbol, Term};
use regler::parser::parse_expr;
use regler::printer::print_expr;

fn lower_str(src: &str) -> Term {
    lower(&parse_expr(src).expect("parse")).expect("lower")
}

fn rule_from(eq_src: &str) -> Rule {
    let t = lower_str(eq_src);
    let (l, r) = match t {
        Term::App(head, args) if head.as_ref() == "=" && args.len() == 2 => {
            let mut it = args.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        }
        _ => panic!("expected an equality, got {t:?}"),
    };
    match orient(&l, &r) {
        Orient::Rule(r) => r,
        other => panic!("expected orientable rule, got {other:?}"),
    }
}

fn simp_str(src: &str, rules: &[Rule]) -> String {
    simp_with(src, rules, &HashMap::new())
}

fn simp_with(src: &str, rules: &[Rule], bindings: &HashMap<Symbol, Term>) -> String {
    let t = lower_str(src);
    let t = subst(&t, bindings);
    let t = simplify(&t, rules);
    let surface = to_surface(&t).expect("to_surface");
    print_expr(&surface)
}

#[test]
fn kbo_orients_x_plus_zero() {
    let l = lower_str("x + 0");
    let r = lower_str("x");
    assert_eq!(kbo(&l, &r), KboOrd::Gt);
    assert_eq!(kbo(&r, &l), KboOrd::Lt);
}

#[test]
fn kbo_commutativity_is_incomparable() {
    let l = lower_str("a + b");
    let r = lower_str("b + a");
    assert_eq!(kbo(&l, &r), KboOrd::Incomparable);
}

#[test]
fn rule_orients_regardless_of_written_direction() {
    let r1 = rule_from("x + 0 = x");
    let r2 = rule_from("x = x + 0");
    assert_eq!(r1.lhs, r2.lhs);
    assert_eq!(r1.rhs, r2.rhs);
}

#[test]
fn rule_with_unbound_rhs_var_is_rejected() {
    // `x = y` — neither direction satisfies the var-count constraint, so KBO
    // reports incomparable and no rule is installed.
    let t = lower_str("x = y");
    let (l, r) = match t {
        Term::App(_, args) => {
            let mut it = args.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        }
        _ => unreachable!(),
    };
    assert!(matches!(orient(&l, &r), Orient::Incomparable));
}

#[test]
fn rule_with_trivial_equality_is_reported() {
    let t = lower_str("x = x");
    let (l, r) = match t {
        Term::App(_, args) => {
            let mut it = args.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        }
        _ => unreachable!(),
    };
    assert!(matches!(orient(&l, &r), Orient::Trivial));
}

#[test]
fn simplify_applies_x_plus_zero_rule() {
    let rules = vec![rule_from("x + 0 = x")];
    assert_eq!(simp_str("a + 0", &rules), "a");
    assert_eq!(simp_str("(a + 0) + 0", &rules), "a");
}

#[test]
fn simplify_under_context() {
    let rules = vec![rule_from("x + 0 = x")];
    assert_eq!(simp_str("(a + 0) · b", &rules), "a · b");
    assert_eq!(simp_str("(a + 0) ^ (c + 0)", &rules), "a ^ c");
}

#[test]
fn simplify_folds_arithmetic_with_rewrites() {
    let rules = vec![rule_from("x + 0 = x")];
    assert_eq!(simp_str("(1 + 2) + 0", &rules), "3");
    assert_eq!(simp_str("(a + 0) + (2 · 3)", &rules), "a + 6");
}

#[test]
fn simplify_terminates_with_incomparable_rule() {
    // Commutativity is incomparable, so it must NOT install as a rule —
    // an empty rule set should leave the term untouched (modulo arith).
    let rules: Vec<Rule> = Vec::new();
    assert_eq!(simp_str("a + b", &rules), "a + b");
    assert_eq!(simp_str("b + a", &rules), "b + a");
}

#[test]
fn simplify_resolves_let_bindings_then_rewrites() {
    let mut bindings: HashMap<Symbol, Term> = HashMap::new();
    bindings.insert(regler::kernel::term::sym("a"), lower_str("7"));
    let rules = vec![rule_from("x + 0 = x")];
    assert_eq!(simp_with("a + 0", &rules, &bindings), "7");
}

#[test]
fn simplify_pattern_var_matches_compound_subterm() {
    let rules = vec![rule_from("x + 0 = x")];
    assert_eq!(simp_str("(a · b) + 0", &rules), "a · b");
}
