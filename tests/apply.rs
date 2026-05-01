use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{apply_eq, simplify};
use regler::kernel::term::{sym, Term};
use regler::kernel::theory::Theory;
use regler::parser::{parse_command, parse_expr};
use regler::printer::{print_command, print_expr};
use regler::ast::Command;

fn lower_str(src: &str) -> Term {
    lower(&parse_expr(src).expect("parse")).expect("lower")
}

fn surface(t: &Term) -> String {
    print_expr(&to_surface(t).expect("to_surface"))
}

// ── apply_eq core ──────────────────────────────────────────────────────────

#[test]
fn apply_top_level_match() {
    // a·(b+c) = a·b + a·c applied to x·(y+2) → x·y + x·2
    let lhs = lower_str("a · (b + c)");
    let rhs = lower_str("a · b + a · c");
    let target = lower_str("x · (y + 2)");
    let result = apply_eq(&lhs, &rhs, &target).expect("should match");
    // The result is structured but may need simplify to fold; just check it's not None.
    let _ = surface(&result); // must not panic
}

#[test]
fn apply_descends_to_subterm() {
    // rule: a·(b+c) = a·b + a·c
    // target: 1 + x·(y+2) — top level is +, rule matches the second child
    let lhs = lower_str("a · (b + c)");
    let rhs = lower_str("a · b + a · c");
    let target = lower_str("1 + x · (y + 2)");
    let result = apply_eq(&lhs, &rhs, &target).expect("should find subterm match");
    let _ = surface(&result);
}

#[test]
fn apply_no_match_returns_none() {
    let lhs = lower_str("a · (b + c)");
    let rhs = lower_str("a · b + a · c");
    let target = lower_str("x + y");
    assert!(apply_eq(&lhs, &rhs, &target).is_none());
}

#[test]
fn apply_reverse_factors() {
    // apply ← distrib to x·y + x·z should give x·(y+z)
    let lhs = lower_str("a · (b + c)");
    let rhs = lower_str("a · b + a · c");
    let target = lower_str("x · y + x · z");
    // forward: a·(b+c) → a·b+a·c matches x·(y+z)? No.
    // reverse: a·b+a·c → a·(b+c) should match x·y+x·z
    let result = apply_eq(&rhs, &lhs, &target).expect("reverse should match");
    let _ = surface(&result);
}

// ── named fact round-trip (parse / print) ─────────────────────────────────

#[test]
fn named_fact_parses() {
    let cmd = parse_command("fact distrib : a · (b + c) = a · b + a · c")
        .expect("parse ok")
        .expect("some command");
    match &cmd {
        Command::Fact(Some(name), _, None) => assert_eq!(name, "distrib"),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn named_fact_round_trips() {
    let src = "fact distrib : a · (b + c) = a · b + a · c";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

#[test]
fn anonymous_fact_round_trips() {
    let src = "fact a + b = b + a";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

// ── conditional fact round-trip ────────────────────────────────────────────

#[test]
fn conditional_fact_parses() {
    let cmd = parse_command("fact x / y = x · y if y ≠ 0")
        .expect("parse ok")
        .expect("some command");
    match &cmd {
        Command::Fact(None, _, Some(_)) => {}
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn conditional_fact_round_trips() {
    let src = "fact x / y = x · y if y ≠ 0";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

#[test]
fn named_conditional_fact_round_trips() {
    let src = "fact div_rule : x / y = x · y if y ≠ 0";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

// ── apply command parse / print ────────────────────────────────────────────

#[test]
fn apply_command_parses() {
    let cmd = parse_command("apply distrib to x · (y + z)")
        .unwrap()
        .unwrap();
    match &cmd {
        Command::Apply(name, _) => assert_eq!(name, "distrib"),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn apply_rev_command_parses() {
    let cmd = parse_command("apply ← distrib to x · y + x · z")
        .unwrap()
        .unwrap();
    match &cmd {
        Command::ApplyRev(name, _) => assert_eq!(name, "distrib"),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn apply_command_round_trips() {
    let src = "apply distrib to x · (y + z)";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

#[test]
fn apply_rev_command_round_trips() {
    let src = "apply ← distrib to x · y + x · z";
    let cmd = parse_command(src).unwrap().unwrap();
    assert_eq!(print_command(&cmd), src);
}

// ── conditional rule fires only when condition is satisfied ────────────────

#[test]
fn conditional_rule_fires_for_nonzero_literal() {
    // fact x - y = x - y if y ≠ 0  (silly but testable)
    // More concretely: install a rule that only fires when b ≠ 0
    // Use: 0 - b = -(b) "if b ≠ 0" — let's do something simpler.
    //
    // Install: a / b = a if b ≠ 0  (obviously wrong math but tests the gate)
    let mut theory = Theory::new();
    let eq = lower_str("a / b = a");
    let cond = lower_str("b ≠ 0");
    theory.install_fact(&eq, None, Some(&cond));

    // With b=3 (non-zero literal), rule should fire: x/3 → x
    let target = lower_str("x / 3");
    let result = simplify(&target, &theory);
    // x is Var, 3 is Nat(3), after subst b→Nat(3) in condition: 3≠0 → true
    assert_eq!(surface(&result), "x");
}

#[test]
fn conditional_rule_does_not_fire_for_zero() {
    let mut theory = Theory::new();
    let eq = lower_str("a / b = a");
    let cond = lower_str("b ≠ 0");
    theory.install_fact(&eq, None, Some(&cond));

    // With b=0, condition fails: rule must not fire
    let target = lower_str("x / 0");
    let result = simplify(&target, &theory);
    assert_eq!(surface(&result), "x / 0");
}

#[test]
fn conditional_rule_does_not_fire_for_symbolic() {
    let mut theory = Theory::new();
    let eq = lower_str("a / b = a");
    let cond = lower_str("b ≠ 0");
    theory.install_fact(&eq, None, Some(&cond));

    // With b symbolic (unknown), conservatively don't fire
    let target = lower_str("x / y");
    let result = simplify(&target, &theory);
    assert_eq!(surface(&result), "x / y");
}

// ── named fact stored in theory ────────────────────────────────────────────

#[test]
fn named_fact_stored_in_theory() {
    let mut theory = Theory::new();
    let eq = lower_str("a · (b + c) = a · b + a · c");
    theory.install_fact(&eq, Some(sym("distrib")), None);
    assert!(theory.named.contains_key(&sym("distrib")));
}

#[test]
fn named_incomparable_fact_stored_but_no_rule() {
    // Build myf(a,b) = myf(b,a) programmatically (commutativity of custom op):
    // KBO-incomparable, so no auto-rule installed; but the name must be stored.
    use regler::kernel::term::Term;
    let a = Term::Var(sym("a"));
    let b = Term::Var(sym("b"));
    let myf_ab = Term::App(sym("myf"), vec![a.clone(), b.clone()]);
    let myf_ba = Term::App(sym("myf"), vec![b.clone(), a.clone()]);
    let eq = Term::App(sym("="), vec![myf_ab, myf_ba]);

    let mut theory = Theory::new();
    theory.install_fact(&eq, Some(sym("myf_comm")), None);

    assert!(theory.named.contains_key(&sym("myf_comm")));
    // commutativity shape is recognised before orient, so no rule installed
    assert!(theory.rules.is_empty());
}
