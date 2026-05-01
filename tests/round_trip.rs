use regler::parser::{parse_command, parse_expr};
use regler::printer::{print_command, print_expr};

fn rt_expr(src: &str) {
    let e1 = parse_expr(src).expect("first parse");
    let printed = print_expr(&e1);
    let e2 = parse_expr(&printed).expect("reparse");
    assert_eq!(e1, e2, "round-trip failed; printed = {printed:?}");
}

fn rt_cmd(src: &str) {
    let c1 = parse_command(src).expect("first parse").expect("expected a command");
    let printed = print_command(&c1);
    let c2 = parse_command(&printed).expect("reparse").expect("expected a command on reparse");
    assert_eq!(c1, c2, "round-trip failed; printed = {printed:?}");
}

#[test]
fn atoms() {
    rt_expr("a");
    rt_expr("foo_bar");
    rt_expr("0");
    rt_expr("123");
    rt_expr("99999999999999999999999999999999");
}

#[test]
fn flat_binops() {
    rt_expr("a + b");
    rt_expr("a · b");
    rt_expr("a ^ b");
    rt_expr("a = b");
}

#[test]
fn precedence_mix() {
    rt_expr("a + b · c");
    rt_expr("(a + b) · c");
    rt_expr("a · b + c");
    rt_expr("a · (b + c)");
    rt_expr("a · b ^ c");
    rt_expr("(a · b) ^ c");
}

#[test]
fn associativity() {
    rt_expr("a + b + c");
    rt_expr("a + (b + c)");
    rt_expr("a · b · c");
    rt_expr("a ^ b ^ c");
    rt_expr("(a ^ b) ^ c");
}

#[test]
fn equality_at_top() {
    rt_expr("a + b = b + a");
    rt_expr("a · (b + c) = a · b + a · c");
    rt_expr("x ^ 2 = x · x");
}

#[test]
fn commands() {
    rt_cmd("let x = 1 + 2");
    rt_cmd("let foo = a · b");
    rt_cmd("fact a + b = b + a");
    rt_cmd("fact a · (b + c) = a · b + a · c");
    rt_cmd("print x");
    rt_cmd("print a + b · c");
    rt_cmd("evaluate 2 + 3 · 4");
    rt_cmd("evaluate x + 2 · 3");
    rt_cmd("simplify a + 0");
    rt_cmd("simplify (x + 0) · y");
}

#[test]
fn parens_only_when_needed() {
    let e = parse_expr("((a + b))").unwrap();
    assert_eq!(print_expr(&e), "a + b");
    let e = parse_expr("a + (b · c)").unwrap();
    assert_eq!(print_expr(&e), "a + b · c");
}
