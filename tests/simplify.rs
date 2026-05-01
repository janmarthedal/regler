use std::collections::HashMap;

use regler::kernel::kbo::{kbo, KboOrd};
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{orient, simplify, Orient, Rule};
use regler::kernel::subst::subst;
use regler::kernel::term::{Symbol, Term};
use regler::kernel::theory::Theory;
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

fn theory_with_rules<I: IntoIterator<Item = Rule>>(rules: I) -> Theory {
    let mut t = Theory::new();
    t.rules.extend(rules);
    t
}

fn theory_from_facts(facts: &[&str]) -> Theory {
    let mut t = Theory::new();
    for f in facts {
        let term = lower_str(f);
        t.install_fact(&term);
    }
    t
}

fn simp_str(src: &str, theory: &Theory) -> String {
    simp_with(src, theory, &HashMap::new())
}

fn simp_with(src: &str, theory: &Theory, bindings: &HashMap<Symbol, Term>) -> String {
    let t = lower_str(src);
    let t = subst(&t, bindings);
    let t = simplify(&t, theory);
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
    let theory = theory_with_rules([rule_from("x + 0 = x")]);
    assert_eq!(simp_str("a + 0", &theory), "a");
    assert_eq!(simp_str("(a + 0) + 0", &theory), "a");
}

#[test]
fn simplify_under_context() {
    let theory = theory_with_rules([rule_from("x + 0 = x")]);
    assert_eq!(simp_str("(a + 0) · b", &theory), "a · b");
    assert_eq!(simp_str("(a + 0) ^ (c + 0)", &theory), "a ^ c");
}

#[test]
fn simplify_folds_arithmetic_with_rewrites() {
    let theory = theory_with_rules([rule_from("x + 0 = x")]);
    assert_eq!(simp_str("(1 + 2) + 0", &theory), "3");
    assert_eq!(simp_str("(a + 0) + (2 · 3)", &theory), "a + 6");
}

#[test]
fn simplify_terminates_with_incomparable_rule() {
    let theory = Theory::new();
    assert_eq!(simp_str("a + b", &theory), "a + b");
    assert_eq!(simp_str("b + a", &theory), "b + a");
}

#[test]
fn simplify_resolves_let_bindings_then_rewrites() {
    let mut bindings: HashMap<Symbol, Term> = HashMap::new();
    bindings.insert(regler::kernel::term::sym("a"), lower_str("7"));
    let theory = theory_with_rules([rule_from("x + 0 = x")]);
    assert_eq!(simp_with("a + 0", &theory, &bindings), "7");
}

#[test]
fn simplify_pattern_var_matches_compound_subterm() {
    let theory = theory_with_rules([rule_from("x + 0 = x")]);
    assert_eq!(simp_str("(a · b) + 0", &theory), "a · b");
}

// --- Milestone 5: AC and identity-element marking ---

#[test]
fn commutativity_alone_does_not_promote_to_ac() {
    let theory = theory_from_facts(&["a + b = b + a"]);
    // No AC yet — `b + a` stays in its written order.
    assert_eq!(simp_str("b + a", &theory), "b + a");
}

#[test]
fn associativity_alone_does_not_promote_to_ac() {
    let theory = theory_from_facts(&["(a + b) + c = a + (b + c)"]);
    // The associativity-shape fact is recognised but installs no rewrite rule
    // on its own; without commutativity, `+` is not AC, so the term is
    // untouched.
    assert_eq!(simp_str("(c + b) + a", &theory), "c + b + a");
}

#[test]
fn ac_promotion_sorts_operands() {
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
    ]);
    // After AC, both terms canonicalise to the same sorted, flattened form.
    let s1 = simp_str("(c + a) + b", &theory);
    let s2 = simp_str("b + (a + c)", &theory);
    assert_eq!(s1, s2);
    assert_eq!(s1, "a + b + c");
}

#[test]
fn ac_folds_literal_operands() {
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
    ]);
    // `2 + a + 3` should fold the literals into `5`.
    assert_eq!(simp_str("(2 + a) + 3", &theory), "a + 5");
}

#[test]
fn ac_with_identity_drops_identity_operand() {
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
        "x + 0 = x",
    ]);
    // The `0` operand is absorbed; result is the canonical sorted pair.
    assert_eq!(simp_str("(a + 0) + b", &theory), "a + b");
    assert_eq!(simp_str("0 + a + 0 + b", &theory), "a + b");
}

#[test]
fn ac_collapse_to_single_operand() {
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
        "x + 0 = x",
    ]);
    // Dropping all but one operand collapses the application.
    assert_eq!(simp_str("0 + a + 0", &theory), "a");
}

#[test]
fn ac_collapse_to_identity() {
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
        "x + 0 = x",
    ]);
    // No non-identity operands left → result is the identity element itself.
    assert_eq!(simp_str("0 + 0", &theory), "0");
}

#[test]
fn ac_for_multiplication_independent_of_addition() {
    let theory = theory_from_facts(&[
        "a · b = b · a",
        "(a · b) · c = a · (b · c)",
        "x · 1 = x",
    ]);
    assert_eq!(simp_str("(b · 1) · a", &theory), "a · b");
    assert_eq!(simp_str("(2 · a) · 3", &theory), "a · 6");
    // `+` is NOT AC here — addition operands stay as written.
    assert_eq!(simp_str("b + a", &theory), "b + a");
}

#[test]
fn left_identity_alone_drops_in_non_ac_setting() {
    // Use `^` (which we don't make AC) with a fictitious left identity. We
    // install `1 ^ x = x` as a left-identity fact.
    let theory = theory_from_facts(&["1 ^ x = x"]);
    assert_eq!(simp_str("1 ^ a", &theory), "a");
    // Right-side `e` is not registered as a right identity (operator non-AC).
    assert_eq!(simp_str("a ^ 1", &theory), "a ^ 1");
}

#[test]
fn ac_unifies_left_and_right_identity() {
    // Stating only the right-identity form once + AC → both sides absorbed.
    let theory = theory_from_facts(&[
        "a + b = b + a",
        "(a + b) + c = a + (b + c)",
        "x + 0 = x",
    ]);
    // Even though we only stated x + 0 = x (right id), 0 + a should drop too.
    assert_eq!(simp_str("0 + a", &theory), "a");
}
