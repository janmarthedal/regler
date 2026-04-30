use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use regler::ast::{Command, Expr};
use regler::kernel::eval::evaluate;
use regler::kernel::lower::lower;
use regler::kernel::print::to_surface;
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut bindings: HashMap<String, Expr> = HashMap::new();
    let mut kernel_bindings: HashMap<Symbol, Term> = HashMap::new();
    let mut facts: Vec<Expr> = Vec::new();

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
            },
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    let _ = facts;
    Ok(())
}

fn run_evaluate(e: &Expr, bindings: &HashMap<Symbol, Term>) -> Result<String, String> {
    let t = lower(e).map_err(|err| err.0)?;
    let t = subst(&t, bindings);
    let t = evaluate(&t).map_err(|err| err.0)?;
    let surface = to_surface(&t).map_err(|err| err.0)?;
    Ok(print_expr(&surface))
}
