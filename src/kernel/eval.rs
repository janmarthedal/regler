use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};

use crate::kernel::term::{sym, Term};

#[derive(Debug)]
pub struct EvalError(pub String);

/// Reduce `t` to a normal form by recursively evaluating arguments and then
/// folding literal arithmetic on ℕ, ℤ, and ℚ via `reduce`. Non-numeric
/// applications and free variables are returned unchanged.
pub fn evaluate(t: &Term) -> Result<Term, EvalError> {
    match t {
        Term::Nat(_) | Term::Var(_) | Term::Int(_) | Term::Rat(_) => Ok(t.clone()),
        Term::App(head, args) => {
            let args: Vec<Term> = args.iter().map(evaluate).collect::<Result<_, _>>()?;
            reduce(head, args)
        }
    }
}

/// Apply built-in literal arithmetic. For `+`, `-`, `·`, `/`: when both
/// arguments are numeric, fold them — promoting to the widest type needed.
/// For `^`: fold only when base and exponent are both `Nat`.
fn reduce(head: &str, args: Vec<Term>) -> Result<Term, EvalError> {
    if args.len() == 1 && head == "-" {
        if let Some(a) = term_to_rat(&args[0]) {
            return Ok(rat_to_term(-a));
        }
    }
    if args.len() == 2 {
        match head {
            "+" | "-" | "·" | "/" => {
                if let (Some(a), Some(b)) = (term_to_rat(&args[0]), term_to_rat(&args[1])) {
                    let result = match head {
                        "+" => a + b,
                        "-" => a - b,
                        "·" => a * b,
                        "/" => {
                            if b.is_zero() {
                                return Err(EvalError("division by zero".into()));
                            }
                            a / b
                        }
                        _ => unreachable!(),
                    };
                    return Ok(rat_to_term(result));
                }
            }
            "^" => {
                if let (Term::Nat(a), Term::Nat(b)) = (&args[0], &args[1]) {
                    return match b.to_u32() {
                        Some(e) => Ok(Term::Nat(a.pow(e))),
                        None => Err(EvalError(format!(
                            "exponent {b} too large to evaluate (must fit in u32)"
                        ))),
                    };
                }
            }
            _ => {}
        }
    }
    Ok(Term::App(sym(head), args))
}

/// Convert a numeric `Term` to `BigRational`. Returns `None` for non-numeric terms.
pub(crate) fn term_to_rat(t: &Term) -> Option<BigRational> {
    match t {
        Term::Nat(n) => Some(BigRational::from(BigInt::from(n.clone()))),
        Term::Int(n) => Some(BigRational::from(n.clone())),
        Term::Rat(r) => Some(r.clone()),
        _ => None,
    }
}

/// Convert a `BigRational` back to the most specific numeric `Term`:
/// integer-valued rationals become `Nat` (if non-negative) or `Int`;
/// non-integer rationals stay as `Rat`.
pub(crate) fn rat_to_term(r: BigRational) -> Term {
    if r.denom() == &BigInt::one() {
        let (sign, mag) = r.numer().clone().into_parts();
        if sign == Sign::Minus {
            Term::Int(BigInt::from_biguint(Sign::Minus, mag))
        } else {
            Term::Nat(mag)
        }
    } else {
        Term::Rat(r)
    }
}
