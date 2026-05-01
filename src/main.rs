use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};

use regler::ast::{Command, Expr};
use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::simplify;
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::kernel::theory::{FactEffect, Theory};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};

/// REPL entry point: reads commands line by line and dispatches them to the
/// surface-level binding store, fact list, and kernel.
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut bindings: HashMap<String, Expr> = HashMap::new();
    let mut kernel_bindings: HashMap<Symbol, Term> = HashMap::new();
    let mut facts: Vec<Expr> = Vec::new();
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
                Ok(Some(cmd)) => dispatch(cmd, &mut bindings, &mut kernel_bindings, &mut facts, &mut theory),
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
            Ok(Some(cmd)) => dispatch(cmd, &mut bindings, &mut kernel_bindings, &mut facts, &mut theory),
            Ok(None) => {}
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    Ok(())
}

/// Dispatch a parsed command, updating all state in place and printing results.
fn dispatch(
    cmd: Command,
    bindings: &mut HashMap<String, Expr>,
    kernel_bindings: &mut HashMap<Symbol, Term>,
    facts: &mut Vec<Expr>,
    theory: &mut Theory,
) {
    match cmd {
        Command::Let(name, e) => {
            println!("{}", print_command(&Command::Let(name.clone(), e.clone())));
            match lower(&e) {
                Ok(t) => {
                    kernel_bindings.insert(sym(&name), t);
                    bindings.insert(name, e);
                }
                Err(err) => println!("error: {}", err.0),
            }
        }
        Command::Fact(e) => {
            println!("{}", print_command(&Command::Fact(e.clone())));
            install_fact(&e, theory);
            facts.push(e);
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
    }
}

/// Implement the `evaluate` REPL command: lower the surface expression into
/// a kernel term, substitute previously-`let`-bound names, reduce to a normal
/// form, and pretty-print the result back as surface syntax.
fn run_evaluate(e: &Expr, bindings: &HashMap<Symbol, Term>) -> Result<String, String> {
    let t = lower(e).map_err(|err| err.0)?;
    let t = subst(&t, bindings);
    let t = evaluate(&t).map_err(|err| err.0)?;
    let surface = to_surface(&t).map_err(|err| err.0)?;
    Ok(print_expr(&surface))
}

/// Implement the `simplify` REPL command: lower, substitute let-bindings,
/// then reduce by repeatedly applying every auto-oriented rewrite rule and
/// folding closed literal arithmetic.
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

/// Hand the fact to the theory: it tries commutativity / associativity /
/// identity-shape recognition first, then falls back to KBO orientation.
///
/// Note: `Var`s in a fact are treated as pattern variables, NOT resolved
/// against `let` bindings — `fact x + 0 = x` orients with `x` as a pattern
/// variable that matches anything.
fn install_fact(e: &Expr, theory: &mut Theory) {
    let t = match lower(e) {
        Ok(t) => t,
        Err(err) => {
            println!("note: fact not installed: {}", err.0);
            return;
        }
    };
    for effect in theory.install_fact(&t) {
        match effect {
            FactEffect::NotEquality => {}
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
