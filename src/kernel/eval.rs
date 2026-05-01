use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::kernel::term::Term;

#[derive(Debug)]
pub struct EvalError(pub String);

/// Reduce `t` to a normal form by recursively evaluating arguments and then
/// folding literal arithmetic on ℕ via `reduce`. Non-numeric applications and
/// free variables are returned unchanged.
pub fn evaluate(t: &Term) -> Result<Term, EvalError> {
    match t {
        Term::Nat(_) | Term::Var(_) => Ok(t.clone()),
        Term::App(head, args) => {
            let args: Vec<Term> = args
                .iter()
                .map(evaluate)
                .collect::<Result<_, _>>()?;
            Ok(reduce(head, args)?)
        }
    }
}

/// Apply built-in literal arithmetic: when both arguments are `Nat`, fold
/// `+`, `·`, and `^` into a single `Nat`. All other shapes are rebuilt as an
/// application unchanged.
fn reduce(head: &str, args: Vec<Term>) -> Result<Term, EvalError> {
    if args.len() == 2 {
        if let (Term::Nat(a), Term::Nat(b)) = (&args[0], &args[1]) {
            match head {
                "+" => return Ok(Term::Nat(a + b)),
                "·" => return Ok(Term::Nat(a * b)),
                "^" => return Ok(Term::Nat(pow_nat(a, b)?)),
                _ => {}
            }
        }
    }
    Ok(Term::App(crate::kernel::term::sym(head), args))
}

fn pow_nat(base: &BigUint, exp: &BigUint) -> Result<BigUint, EvalError> {
    match exp.to_u32() {
        Some(e) => Ok(base.pow(e)),
        None => Err(EvalError(format!(
            "exponent {exp} too large to evaluate (must fit in u32)"
        ))),
    }
}
