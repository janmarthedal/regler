use num_bigint::BigInt;

use crate::ast::{Expr, Op, UnaryOp};
use crate::kernel::term::Term;

#[derive(Debug)]
pub struct UnprintableError(pub String);

/// Lift a kernel term back into the surface AST. Binary applications whose
/// head matches a known infix operator become `BinOp`; n-ary AC applications
/// are unfolded into left-nested `BinOp`; all other applications become
/// `Expr::App` (function call notation).
pub fn to_surface(t: &Term) -> Result<Expr, UnprintableError> {
    match t {
        Term::Nat(n) => Ok(Expr::Int(BigInt::from(n.clone()))),
        Term::Int(n) => Ok(Expr::Int(n.clone())),
        Term::Rat(r) => Ok(Expr::BinOp(
            Op::Div,
            Box::new(Expr::Int(r.numer().clone())),
            Box::new(Expr::Int(r.denom().clone())),
        )),
        Term::Var(s) => Ok(Expr::Ident(s.to_string())),
        Term::Sym(s) => Ok(Expr::Ident(s.to_string())),
        Term::Lam(x, ty, body) => Ok(Expr::Lambda(
            x.to_string(),
            Box::new(to_surface(ty)?),
            Box::new(to_surface(body)?),
        )),
        Term::App(head, args) => {
            match head.as_ref() {
                Term::Sym(h) => {
                    // 0-arity: just the symbol name
                    if args.is_empty() {
                        return Ok(Expr::Ident(h.to_string()));
                    }
                    // Unary negation
                    if h.as_ref() == "-" && args.len() == 1 {
                        return Ok(Expr::UnaryOp(
                            UnaryOp::Neg,
                            Box::new(to_surface(&args[0])?),
                        ));
                    }
                    // Known infix operators
                    if let Some(op) = op_for(h) {
                        match args.len() {
                            2 => {
                                return Ok(Expr::BinOp(
                                    op,
                                    Box::new(to_surface(&args[0])?),
                                    Box::new(to_surface(&args[1])?),
                                ))
                            }
                            n if n > 2 => {
                                let mut it = args.iter();
                                let mut acc = to_surface(it.next().unwrap())?;
                                for a in it {
                                    acc = Expr::BinOp(op, Box::new(acc), Box::new(to_surface(a)?));
                                }
                                return Ok(acc);
                            }
                            _ => {}
                        }
                    }
                    // Function application
                    let surf_args: Result<Vec<_>, _> = args.iter().map(to_surface).collect();
                    Ok(Expr::App(h.to_string(), surf_args?))
                }
                Term::Var(v) => {
                    // Higher-order application with a variable head
                    let surf_args: Result<Vec<_>, _> = args.iter().map(to_surface).collect();
                    Ok(Expr::App(v.to_string(), surf_args?))
                }
                _ => Err(UnprintableError(
                    format!("cannot print application with non-symbol/non-variable head: {head:?}")
                )),
            }
        }
    }
}

fn op_for(head: &str) -> Option<Op> {
    match head {
        "+" => Some(Op::Add),
        "-" => Some(Op::Sub),
        "·" => Some(Op::Mul),
        "/" => Some(Op::Div),
        "^" => Some(Op::Pow),
        "=" => Some(Op::Eq),
        "≠" => Some(Op::Ne),
        ">" => Some(Op::Gt),
        "<" => Some(Op::Lt),
        "≥" => Some(Op::Ge),
        "≤" => Some(Op::Le),
        "⊆" => Some(Op::Subset),
        "∈" => Some(Op::In),
        "∧" => Some(Op::And),
        "∨" => Some(Op::Or),
        "⇒" => Some(Op::Implies),
        "→" => Some(Op::Arrow),
        _ => None,
    }
}
