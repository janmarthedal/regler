use regler::ast::{Command, Expr, Op, UnaryOp};
use regler::kernel::lower::lower;
use regler::kernel::theory::Theory;
use regler::parser::{parse_command, parse_expr};
use regler::printer::{print_command, print_expr};

fn rt_expr(src: &str) {
    let e1 = parse_expr(src).expect("first parse");
    let printed = print_expr(&e1);
    let e2 = parse_expr(&printed).expect("reparse");
    assert_eq!(e1, e2, "round-trip failed; printed = {printed:?}");
}

fn rt_cmd(src: &str) {
    let c1 = parse_command(src).expect("parse").expect("command");
    let printed = print_command(&c1);
    let c2 = parse_command(&printed).expect("reparse").expect("command");
    assert_eq!(c1, c2, "round-trip failed; printed = {printed:?}");
}

fn install_and_simplify(facts: &[&str], expr: &str) -> String {
    let mut theory = Theory::new();
    for f in facts {
        let cmd = parse_command(f).expect("parse fact").expect("command");
        if let Command::Fact(name, prop, cond) = cmd {
            let t = lower(&prop).expect("lower");
            let cond_t = cond.map(|c| lower(&c).expect("lower cond"));
            let sym_name = name.as_deref().map(regler::kernel::term::sym);
            theory.install_fact(&t, sym_name, cond_t.as_ref());
        }
    }
    let t = lower(&parse_expr(expr).expect("parse")).expect("lower");
    let result = regler::kernel::rewrite::simplify(&t, &theory);
    let surface = regler::kernel::print::to_surface(&result).expect("to_surface");
    print_expr(&surface)
}

// ── Parsing ──────────────────────────────────────────────────────────────────

#[test]
fn forall_single_var_parses() {
    let e = parse_expr("∀ a ∈ ℕ. a + 0 = a").expect("parse");
    match &e {
        Expr::Forall(vars, domain, body) => {
            assert_eq!(vars, &["a"]);
            assert_eq!(**domain, Expr::Ident("ℕ".into()));
            assert!(matches!(**body, Expr::BinOp(Op::Eq, _, _)));
        }
        _ => panic!("expected Forall, got {e:?}"),
    }
}

#[test]
fn forall_multi_var_parses() {
    let e = parse_expr("∀ a, b ∈ ℕ. a + b = b + a").expect("parse");
    match &e {
        Expr::Forall(vars, _, _) => assert_eq!(vars, &["a", "b"]),
        _ => panic!("expected Forall"),
    }
}

#[test]
fn forall_three_vars_parses() {
    let e = parse_expr("∀ a, b, c ∈ ℕ. a + b + c = a + (b + c)").expect("parse");
    match &e {
        Expr::Forall(vars, _, _) => assert_eq!(vars, &["a", "b", "c"]),
        _ => panic!("expected Forall"),
    }
}

// ── Round-trip ────────────────────────────────────────────────────────────────

#[test]
fn forall_single_var_round_trips() {
    rt_expr("∀ a ∈ ℕ. a + 0 = a");
}

#[test]
fn forall_multi_var_round_trips() {
    rt_expr("∀ a, b ∈ ℕ. a + b = b + a");
}

#[test]
fn forall_fact_command_round_trips() {
    rt_cmd("fact ∀ a ∈ ℕ. a + 0 = a");
    rt_cmd("fact ∀ a, b ∈ ℕ. a + b = b + a");
    rt_cmd("fact ∀ a, b, c ∈ ℕ. a · (b + c) = a · b + a · c");
}

// ── Logical operators ─────────────────────────────────────────────────────────

#[test]
fn implies_parses_and_round_trips() {
    rt_expr("a · b = 0 ⇒ a = 0 ∨ b = 0");
}

#[test]
fn conjunction_parses_and_round_trips() {
    rt_expr("a = 0 ∧ b = 0");
}

#[test]
fn implication_right_associative() {
    let e = parse_expr("a ⇒ b ⇒ c").expect("parse");
    let printed = print_expr(&e);
    // right-assoc: a ⇒ (b ⇒ c), inner parens not needed
    assert_eq!(printed, "a ⇒ b ⇒ c");
    let e2 = parse_expr("(a ⇒ b) ⇒ c").expect("parse");
    let printed2 = print_expr(&e2);
    assert_eq!(printed2, "(a ⇒ b) ⇒ c");
}

#[test]
fn forall_with_implies_round_trips() {
    rt_cmd("fact ∀ a, b ∈ ℤ. a · b = 0 ⇒ a = 0 ∨ b = 0");
}

// ── Unary minus on identifiers ─────────────────────────────────────────────────

#[test]
fn unary_minus_on_ident_parses() {
    let e = parse_expr("-a").expect("parse");
    assert!(matches!(e, Expr::UnaryOp(UnaryOp::Neg, _)));
}

// ── Lowering strips ∀ binder ──────────────────────────────────────────────────

#[test]
fn forall_lowers_to_body() {
    let with_forall = parse_expr("∀ a ∈ ℕ. a + 0 = a").expect("parse");
    let without = parse_expr("a + 0 = a").expect("parse");
    let t1 = lower(&with_forall).expect("lower forall");
    let t2 = lower(&without).expect("lower plain");
    assert_eq!(t1, t2);
}

// ── End-to-end: nat.rgl facts rewrite correctly ───────────────────────────────

#[test]
fn nat_identity_fires_after_forall_fact() {
    let result = install_and_simplify(&["fact ∀ a ∈ ℕ. a + 0 = a"], "x + 0");
    assert_eq!(result, "x");
}

#[test]
fn nat_ac_fires_after_forall_facts() {
    let result = install_and_simplify(
        &[
            "fact ∀ a, b ∈ ℕ. a + b = b + a",
            "fact ∀ a, b, c ∈ ℕ. (a + b) + c = a + (b + c)",
        ],
        "c + b + a",
    );
    // AC normalization sorts: a + b + c
    assert_eq!(result, "a + b + c");
}
