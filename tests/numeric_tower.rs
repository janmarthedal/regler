use std::collections::HashMap;

use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::subst::subst;
use regler::kernel::term::Symbol;
use regler::parser::parse_expr;
use regler::printer::print_expr;

fn eval_str(src: &str) -> String {
    let e = parse_expr(src).expect("parse");
    let t = lower(&e).expect("lower");
    let t = subst(&t, &HashMap::<Symbol, _>::new());
    let t = evaluate(&t).expect("evaluate");
    let surface = to_surface(&t).expect("to_surface");
    print_expr(&surface)
}

// ℤ: subtraction and negative results

#[test]
fn subtraction_nat_result() {
    assert_eq!(eval_str("5 - 3"), "2");
}

#[test]
fn subtraction_zero_result() {
    assert_eq!(eval_str("4 - 4"), "0");
}

#[test]
fn subtraction_negative_result() {
    assert_eq!(eval_str("3 - 5"), "-2");
}

#[test]
fn negative_literal_parses_and_evaluates() {
    assert_eq!(eval_str("-7"), "-7");
    assert_eq!(eval_str("-7 + 4"), "-3");
    assert_eq!(eval_str("-3 - -2"), "-1");
}

#[test]
fn subtraction_promoting_through_int() {
    // (3 - 5) + 10 = 8: Int result promoted back to Nat after addition
    assert_eq!(eval_str("(3 - 5) + 10"), "8");
}

// ℚ: division and rational results

#[test]
fn division_exact() {
    assert_eq!(eval_str("6 / 3"), "2");
    assert_eq!(eval_str("10 / 2"), "5");
}

#[test]
fn division_rational_result() {
    assert_eq!(eval_str("1 / 3"), "1 / 3");
    assert_eq!(eval_str("2 / 4"), "1 / 2");
}

#[test]
fn division_negative_rational() {
    assert_eq!(eval_str("(0 - 1) / 3"), "-1 / 3");
}

// Mixed promotions

#[test]
fn rational_addition() {
    assert_eq!(eval_str("1 / 2 + 1 / 3"), "5 / 6");
    assert_eq!(eval_str("1 / 3 + 1 / 3 + 1 / 3"), "1");
}

#[test]
fn rational_subtraction() {
    assert_eq!(eval_str("3 / 4 - 1 / 4"), "1 / 2");
}

#[test]
fn rational_multiplication() {
    assert_eq!(eval_str("2 / 3 · 3 / 4"), "1 / 2");
}

#[test]
fn int_division_to_rational() {
    assert_eq!(eval_str("1 / 2 + 1"), "3 / 2");
}

#[test]
fn nat_times_rational() {
    assert_eq!(eval_str("2 · (1 / 3)"), "2 / 3");
}

// Existing ℕ arithmetic unchanged

#[test]
fn nat_arithmetic_unaffected() {
    assert_eq!(eval_str("2 + 3"), "5");
    assert_eq!(eval_str("2 · 3"), "6");
    assert_eq!(eval_str("2 ^ 10"), "1024");
}
