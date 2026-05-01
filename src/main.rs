use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use regler::ast::{Command, Expr};
use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{orient, simplify, Orient, Rule};
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};

/// REPL entry point: reads commands line by line and dispatches them to the
/// surface-level binding store, fact list, and kernel.
fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut bindings: HashMap<String, Expr> = HashMap::new();
    let mut kernel_bindings: HashMap<Symbol, Term> = HashMap::new();
    let mut facts: Vec<Expr> = Vec::new();
    let mut rules: Vec<Rule> = Vec::new();

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
            Ok(cmd) => match cmd {
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
                    install_fact(&e, &mut rules);
                    facts.push(e);
                }
                Command::Print(e) => {
                    let resolved = match &e {
                        Expr::Ident(name) => bindings.get(name).cloned().unwrap_or(e.clone()),
                        _ => e.clone(),
                    };
                    println!("{}", print_expr(&resolved));
                }
                Command::Evaluate(e) => match run_evaluate(&e, &kernel_bindings) {
                    Ok(out) => println!("{}", out),
                    Err(msg) => println!("error: {}", msg),
                },
                Command::Simplify(e) => match run_simplify(&e, &kernel_bindings, &rules) {
                    Ok(out) => println!("{}", out),
                    Err(msg) => println!("error: {}", msg),
                },
            },
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    Ok(())
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
    rules: &[Rule],
) -> Result<String, String> {
    let t = lower(e).map_err(|err| err.0)?;
    let t = subst(&t, bindings);
    let t = simplify(&t, rules);
    let surface = to_surface(&t).map_err(|err| err.0)?;
    Ok(print_expr(&surface))
}

/// Try to install a fact as an auto-oriented rewrite rule. If the fact is an
/// equality whose two sides are KBO-comparable, the larger side becomes the
/// rule's lhs. Equalities whose sides are KBO-incomparable, or facts that are
/// not equalities, are stored without producing a rule.
///
/// Note: `Var`s in a fact are treated as pattern variables, NOT resolved
/// against `let` bindings — `fact x + 0 = x` orients with `x` as a pattern
/// variable that matches anything.
fn install_fact(e: &Expr, rules: &mut Vec<Rule>) {
    let t = match lower(e) {
        Ok(t) => t,
        Err(err) => {
            println!("note: fact not installed as rule: {}", err.0);
            return;
        }
    };
    let (l, r) = match &t {
        Term::App(head, args) if head.as_ref() == "=" && args.len() == 2 => (&args[0], &args[1]),
        _ => return,
    };
    match orient(l, r) {
        Orient::Rule(rule) => rules.push(rule),
        Orient::Trivial => println!("note: trivial equality, no rule installed"),
        Orient::Incomparable => {
            println!("note: equality is KBO-incomparable, no rule installed")
        }
    }
}
