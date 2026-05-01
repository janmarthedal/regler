use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};

use regler::ast::{Command, Expr, Op};
use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{apply_eq_conditional, simplify};
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::kernel::theory::{FactEffect, Theory};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut bindings: HashMap<String, Expr> = HashMap::new();
    let mut kernel_bindings: HashMap<Symbol, Term> = HashMap::new();
    let mut theory = Theory::new();

    if let Some(path) = env::args().nth(1) {
        let file = File::open(&path).map_err(|e| io::Error::new(e.kind(), format!("{path}: {e}")))?;
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_command(trimmed) {
                Ok(Some(cmd)) => dispatch(cmd, &mut bindings, &mut kernel_bindings, &mut theory),
                Ok(None) => {}
                Err(err) => println!("parse error: {}", err.0),
            }
        }
    }

    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        line.clear();
        let n = stdin.lock().read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match parse_command(trimmed) {
            Ok(Some(cmd)) => dispatch(cmd, &mut bindings, &mut kernel_bindings, &mut theory),
            Ok(None) => {}
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    Ok(())
}

fn dispatch(
    cmd: Command,
    bindings: &mut HashMap<String, Expr>,
    kernel_bindings: &mut HashMap<Symbol, Term>,
    theory: &mut Theory,
) {
    match cmd {
        Command::Let(name, ty, rhs) => {
            println!("{}", print_command(&Command::Let(name.clone(), ty.clone(), rhs.clone())));
            handle_let(name, ty, rhs, bindings, kernel_bindings, theory);
        }
        Command::Fact(name, e, cond) => {
            println!("{}", print_command(&Command::Fact(name.clone(), e.clone(), cond.clone())));
            install_fact(name, &e, cond.as_ref(), theory);
        }
        Command::Print(e) => {
            let resolved = match &e {
                Expr::Ident(name) => bindings.get(name).cloned().unwrap_or(e.clone()),
                _ => e.clone(),
            };
            println!("{}", print_expr(&resolved));
        }
        Command::Evaluate(e) => match run_evaluate(&e, kernel_bindings) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
        Command::Simplify(e) => match run_simplify(&e, kernel_bindings, theory) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
        Command::Apply(name, e) => match run_apply(&name, &e, false, kernel_bindings, theory) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
        Command::ApplyRev(name, e) => match run_apply(&name, &e, true, kernel_bindings, theory) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
    }
}

fn handle_let(
    name: String,
    ty: Option<Expr>,
    rhs: Option<Expr>,
    bindings: &mut HashMap<String, Expr>,
    kernel_bindings: &mut HashMap<Symbol, Term>,
    theory: &mut Theory,
) {
    match (ty.as_ref(), rhs.as_ref()) {
        // `let Name : Set` — opaque set declaration
        (Some(Expr::Ident(t)), None) if t == "Set" => {}

        // `let Name : Set = {x ∈ S | P}` — predicate set definition
        (Some(Expr::Ident(t)), Some(Expr::SetBuilder(var, domain, pred))) if t == "Set" => {
            match (lower(domain), lower(pred)) {
                (Ok(dom_term), Ok(pred_term)) => {
                    theory.add_predicate_set(sym(&name), sym(var), dom_term, pred_term);
                }
                _ => println!("error: cannot lower set-builder definition for `{name}`"),
            }
        }

        // `let name : ty` — opaque declaration with type annotation (e.g. function signature)
        (Some(_ty), None) => {}

        // `let name [: ty] = rhs` — value definition
        (_, Some(rhs_expr)) => {
            match lower(rhs_expr) {
                Ok(t) => {
                    kernel_bindings.insert(sym(&name), t);
                    bindings.insert(name, rhs_expr.clone());
                }
                Err(err) => println!("error: {}", err.0),
            }
        }

        (None, None) => {}
    }
}

fn run_evaluate(e: &Expr, bindings: &HashMap<Symbol, Term>) -> Result<String, String> {
    let t = lower(e).map_err(|err| err.0)?;
    let t = subst(&t, bindings);
    let t = evaluate(&t).map_err(|err| err.0)?;
    let surface = to_surface(&t).map_err(|err| err.0)?;
    Ok(print_expr(&surface))
}

fn run_simplify(
    e: &Expr,
    bindings: &HashMap<Symbol, Term>,
    theory: &Theory,
) -> Result<String, String> {
    let t = lower(e).map_err(|err| err.0)?;
    let t = subst(&t, bindings);
    let t = simplify(&t, theory);
    let surface = to_surface(&t).map_err(|err| err.0)?;
    Ok(print_expr(&surface))
}

fn run_apply(
    name: &str,
    e: &Expr,
    reverse: bool,
    bindings: &HashMap<Symbol, Term>,
    theory: &Theory,
) -> Result<String, String> {
    let nf = theory
        .named
        .get(&sym(name))
        .ok_or_else(|| format!("no named fact `{name}`"))?;

    let (pat, rhs) = if reverse {
        (&nf.rhs, &nf.lhs)
    } else {
        (&nf.lhs, &nf.rhs)
    };

    let target = lower(e).map_err(|err| err.0)?;
    let target = subst(&target, bindings);

    match apply_eq_conditional(pat, rhs, nf.condition.as_ref(), &target, theory) {
        Some(result) => {
            let surface = to_surface(&result).map_err(|err| err.0)?;
            Ok(print_expr(&surface))
        }
        None => Err(format!(
            "fact `{name}` does not match any subterm of the expression"
        )),
    }
}

/// Install a fact into the theory. If the fact has a `∀ vars ∈ Domain.` prefix
/// and `Domain` is a predicate-defined set, membership conditions are generated
/// automatically and merged with any explicit `if` clause.
fn install_fact(
    name: Option<String>,
    e: &Expr,
    condition: Option<&Expr>,
    theory: &mut Theory,
) {
    let (body_expr, binder_cond) = extract_binder_conditions(e, theory);

    // Merge binder-generated conditions with explicit `if` condition.
    let merged_cond: Option<Expr> = match (binder_cond, condition.cloned()) {
        (Some(bc), Some(ec)) => Some(Expr::BinOp(Op::And, Box::new(bc), Box::new(ec))),
        (Some(bc), None) => Some(bc),
        (None, Some(ec)) => Some(ec),
        (None, None) => None,
    };

    let t = match lower(&body_expr) {
        Ok(t) => t,
        Err(err) => {
            println!("note: fact not installed: {}", err.0);
            return;
        }
    };
    let cond_term = match merged_cond.as_ref().map(lower) {
        Some(Ok(t)) => Some(t),
        Some(Err(err)) => {
            println!("note: condition not installed: {}", err.0);
            return;
        }
        None => None,
    };
    let sym_name = name.as_deref().map(sym);
    for effect in theory.install_fact(&t, sym_name, cond_term.as_ref()) {
        match effect {
            FactEffect::NotEquality => {}
            FactEffect::SubsetFact => {}
            FactEffect::RuleInstalled => {}
            FactEffect::AlreadyKnown => {}
            FactEffect::Trivial => println!("note: trivial equality, no rule installed"),
            FactEffect::Incomparable => {
                println!("note: equality is KBO-incomparable, no rule installed")
            }
            FactEffect::Commutativity(f) => {
                println!("note: recognised commutativity for `{}`", f)
            }
            FactEffect::Associativity(f) => {
                println!("note: recognised associativity for `{}`", f)
            }
            FactEffect::LeftIdentity(f, _) => {
                println!("note: registered left identity for `{}`", f)
            }
            FactEffect::RightIdentity(f, _) => {
                println!("note: registered right identity for `{}`", f)
            }
            FactEffect::AcPromoted(f) => {
                println!("note: `{}` promoted to AC", f)
            }
        }
    }
}

/// If `e` is `Forall(vars, domain, body)` and `domain` names a predicate-defined
/// set in `theory`, generate membership conditions `v ∈ domain` for each `v` and
/// return `(body, Some(conditions))`. Otherwise return `(e, None)`.
fn extract_binder_conditions(e: &Expr, theory: &Theory) -> (Expr, Option<Expr>) {
    if let Expr::Forall(vars, domain, body) = e {
        if let Expr::Ident(domain_name) = domain.as_ref() {
            if theory.predicate_sets.contains_key(&sym(domain_name)) {
                let conds: Vec<Expr> = vars
                    .iter()
                    .map(|v| {
                        Expr::BinOp(
                            Op::In,
                            Box::new(Expr::Ident(v.clone())),
                            Box::new(*domain.clone()),
                        )
                    })
                    .collect();
                let cond = conds
                    .into_iter()
                    .reduce(|a, b| Expr::BinOp(Op::And, Box::new(a), Box::new(b)));
                return (*body.clone(), cond);
            }
        }
        // Domain is not a predicate set — strip binder, no conditions generated.
        (*body.clone(), None)
    } else {
        (e.clone(), None)
    }
}
