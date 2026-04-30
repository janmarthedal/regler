use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use regler::ast::{Command, Expr};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut bindings: HashMap<String, Expr> = HashMap::new();
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
                    bindings.insert(name, e);
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
            },
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    let _ = facts; // silence unused warning until milestone 3
    Ok(())
}
