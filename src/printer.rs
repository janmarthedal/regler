use crate::ast::{Command, Expr, Op};

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
    Top,
}

/// Render an expression to surface syntax, inserting parentheses only where
/// required by the operator precedence and associativity rules.
pub fn print_expr(e: &Expr) -> String {
    let mut out = String::new();
    fmt_expr(e, 0, Side::Top, &mut out);
    out
}

pub fn print_command(c: &Command) -> String {
    match c {
        Command::Let(name, e) => format!("let {} = {}", name, print_expr(e)),
        Command::Fact(e) => format!("fact {}", print_expr(e)),
        Command::Print(e) => format!("print {}", print_expr(e)),
        Command::Evaluate(e) => format!("evaluate {}", print_expr(e)),
        Command::Simplify(e) => format!("simplify {}", print_expr(e)),
    }
}

/// Recursive worker for `print_expr`. `parent` is the precedence of the
/// enclosing operator (0 at the top), and `side` records whether this node
/// sits on the left/right of that parent so we can decide when same-precedence
/// nesting needs parentheses.
fn fmt_expr(e: &Expr, parent: u8, side: Side, out: &mut String) {
    match e {
        Expr::Ident(s) => out.push_str(s),
        Expr::Int(n) => out.push_str(&n.to_string()),
        Expr::BinOp(op, l, r) => {
            let p = op.prec();
            let needs = p < parent || (p == parent && wrong_side(*op, side));
            if needs {
                out.push('(');
            }
            fmt_expr(l, p, Side::Left, out);
            out.push(' ');
            out.push_str(op.symbol());
            out.push(' ');
            fmt_expr(r, p, Side::Right, out);
            if needs {
                out.push(')');
            }
        }
    }
}

fn wrong_side(op: Op, side: Side) -> bool {
    // Same-precedence collisions: parenthesize the side that fights associativity.
    // Left-assoc (+, ·, =-treated-as-non-assoc): right child needs parens.
    // Right-assoc (^): left child needs parens.
    // Non-assoc (=): both sides need parens to disambiguate.
    match (op, side) {
        (_, Side::Top) => false,
        (Op::Pow, Side::Left) => true,
        (Op::Pow, Side::Right) => false,
        (Op::Eq, _) => true,
        (_, Side::Right) => true,
        (_, Side::Left) => false,
    }
}
