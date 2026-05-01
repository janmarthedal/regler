use crate::ast::{Command, Expr, Op};

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
    Top,
}

/// Render an expression to surface syntax.
pub fn print_expr(e: &Expr) -> String {
    let mut out = String::new();
    fmt_expr(e, 0, Side::Top, &mut out);
    out
}

pub fn print_command(c: &Command) -> String {
    match c {
        Command::Let(name, ty, rhs) => {
            let mut s = format!("let {name}");
            if let Some(t) = ty {
                s.push_str(" : ");
                s.push_str(&print_expr(t));
            }
            if let Some(r) = rhs {
                s.push_str(" = ");
                s.push_str(&print_expr(r));
            }
            s
        }
        Command::Fact(name, e, cond) => {
            let mut s = String::from("fact ");
            if let Some(n) = name {
                s.push_str(n);
                s.push_str(" : ");
            }
            s.push_str(&print_expr(e));
            if let Some(c) = cond {
                s.push_str(" if ");
                s.push_str(&print_expr(c));
            }
            s
        }
        Command::Print(e) => format!("print {}", print_expr(e)),
        Command::Evaluate(e) => format!("evaluate {}", print_expr(e)),
        Command::Simplify(e) => format!("simplify {}", print_expr(e)),
        Command::Apply(name, e) => format!("apply {} to {}", name, print_expr(e)),
        Command::ApplyRev(name, e) => format!("apply ← {} to {}", name, print_expr(e)),
    }
}

fn fmt_expr(e: &Expr, parent: u8, side: Side, out: &mut String) {
    match e {
        Expr::Ident(s) => out.push_str(s),
        Expr::Int(n) => out.push_str(&n.to_string()),
        Expr::App(f, args) => {
            out.push_str(f);
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                fmt_expr(a, 0, Side::Top, out);
            }
            out.push(')');
        }
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
        Expr::UnaryOp(op, operand) => {
            out.push_str(op.symbol());
            let needs = matches!(**operand, Expr::BinOp(_, _, _) | Expr::Forall(_, _, _));
            if needs {
                out.push('(');
            }
            fmt_expr(operand, 0, Side::Top, out);
            if needs {
                out.push(')');
            }
        }
        Expr::Forall(vars, domain, body) => {
            let needs = parent > 0;
            if needs {
                out.push('(');
            }
            out.push_str("∀ ");
            out.push_str(&vars.join(", "));
            out.push_str(" ∈ ");
            fmt_expr(domain, 0, Side::Top, out);
            out.push_str(". ");
            fmt_expr(body, 0, Side::Top, out);
            if needs {
                out.push(')');
            }
        }
        Expr::SetBuilder(var, domain, pred) => {
            out.push('{');
            out.push_str(var);
            out.push_str(" ∈ ");
            fmt_expr(domain, 0, Side::Top, out);
            out.push_str(" | ");
            fmt_expr(pred, 0, Side::Top, out);
            out.push('}');
        }
    }
}

fn wrong_side(op: Op, side: Side) -> bool {
    match (op, side) {
        (_, Side::Top) => false,
        // Right-associative: left operand at same prec needs parens
        (Op::Pow | Op::Implies | Op::Arrow, Side::Left) => true,
        (Op::Pow | Op::Implies | Op::Arrow, Side::Right) => false,
        // Non-associative: both sides need parens at same level
        (Op::Eq | Op::Ne | Op::Subset | Op::In | Op::Lt | Op::Gt | Op::Le | Op::Ge, _) => true,
        (_, Side::Right) => true,
        (_, Side::Left) => false,
    }
}
