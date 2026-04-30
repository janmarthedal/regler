use std::collections::HashMap;

use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::parser::parse_expr;
use regler::printer::print_expr;

fn eval_str(src: &str) -> String {
    eval_with(src, &HashMap::new())
}

fn eval_with(src: &str, bindings: &HashMap<Symbol, Term>) -> String {
    let e = parse_expr(src).expect("parse");
    let t = lower(&e).expect("lower");
    let t = subst(&t, bindings);
    let t = evaluate(&t).expect("evaluate");
    let surface = to_surface(&t).expect("to_surface");
    print_expr(&surface)
}

#[test]
fn arithmetic_closed() {
    assert_eq!(eval_str("2 + 3"), "5");
    assert_eq!(eval_str("2 · 3"), "6");
    assert_eq!(eval_str("2 ^ 10"), "1024");
    assert_eq!(eval_str("2 + 3 · 4"), "14");
    assert_eq!(eval_str("(2 + 3) · 4"), "20");
}

#[test]
fn arithmetic_bigint() {
    assert_eq!(
        eval_str("99999999999999999999 + 1"),
        "100000000000000000000"
    );
    assert_eq!(eval_str("2 ^ 64"), "18446744073709551616");
}

#[test]
fn symbolic_partial_reduction() {
    assert_eq!(eval_str("x + 2 · 3"), "x + 6");
    assert_eq!(eval_str("(2 + 3) · x"), "5 · x");
    assert_eq!(eval_str("x + y"), "x + y");
}

#[test]
fn evaluator_identity_on_purely_symbolic() {
    for src in ["x", "x + y", "a · (b + c)", "x ^ y", "a = b"] {
        assert_eq!(eval_str(src), print_expr(&parse_expr(src).unwrap()));
    }
}

#[test]
fn substitution_resolves_bindings() {
    let mut bindings: HashMap<Symbol, Term> = HashMap::new();
    bindings.insert(sym("x"), lower(&parse_expr("7").unwrap()).unwrap());
    assert_eq!(eval_with("x + 1", &bindings), "8");
    assert_eq!(eval_with("x · x", &bindings), "49");
}
