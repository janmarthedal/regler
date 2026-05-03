use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use regler::ast::{Command, Expr, Op};
use regler::kernel::eval::evaluate;
use regler::kernel::lower::{lower, lower_fact_body};
use regler::kernel::print::to_surface;
use regler::kernel::rewrite::{apply_eq_conditional, simplify};
use regler::kernel::subst::subst;
use regler::kernel::term::{sym, Symbol, Term};
use regler::kernel::theory::{FactEffect, Theory};
use regler::parser::parse_command;
use regler::printer::{print_command, print_expr};
use regler::runner::collect_statements;

struct Session {
    bindings: HashMap<String, Expr>,
    kernel_bindings: HashMap<Symbol, Term>,
    theory: Theory,
    let_declared: HashSet<String>,
}

impl Session {
    fn new() -> Self {
        Session {
            bindings: HashMap::new(),
            kernel_bindings: HashMap::new(),
            theory: Theory::new(),
            let_declared: HashSet::new(),
        }
    }
}

struct ImportCtx {
    /// Files currently being processed — used for cycle detection.
    in_progress: HashSet<PathBuf>,
    /// Files already fully imported — used for idempotency.
    imported: HashSet<PathBuf>,
}

impl ImportCtx {
    fn new() -> Self {
        ImportCtx {
            in_progress: HashSet::new(),
            imported: HashSet::new(),
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut session = Session::new();
    let mut import_ctx = ImportCtx::new();

    if let Some(path) = env::args().nth(1) {
        let path = PathBuf::from(&path);
        if let Err(e) = run_file(&path, &mut session, &mut import_ctx) {
            println!("error: {e}");
        }
    }

    let repl_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
            Ok(Some(cmd)) => dispatch(cmd, &mut session, &mut import_ctx, &repl_dir),
            Ok(None) => {}
            Err(err) => println!("parse error: {}", err.0),
        }
    }
    Ok(())
}

fn run_file(path: &Path, session: &mut Session, ctx: &mut ImportCtx) -> io::Result<()> {
    let canonical = path.canonicalize().map_err(|e| {
        io::Error::new(e.kind(), format!("{}: {e}", path.display()))
    })?;

    if ctx.imported.contains(&canonical) {
        return Ok(());
    }
    if ctx.in_progress.contains(&canonical) {
        println!("error: import cycle detected: {}", path.display());
        return Ok(());
    }

    ctx.in_progress.insert(canonical.clone());

    let file = File::open(path).map_err(|e| {
        io::Error::new(e.kind(), format!("{}: {e}", path.display()))
    })?;
    let raw: Result<Vec<_>, _> = io::BufReader::new(file).lines().collect();
    let stmts = collect_statements(raw?);

    let current_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();

    for stmt in stmts {
        match parse_command(&stmt) {
            Ok(Some(cmd)) => dispatch(cmd, session, ctx, &current_dir),
            Ok(None) => {}
            Err(err) => println!("parse error: {}", err.0),
        }
    }

    ctx.in_progress.remove(&canonical);
    ctx.imported.insert(canonical);
    Ok(())
}

fn dispatch(cmd: Command, session: &mut Session, ctx: &mut ImportCtx, current_dir: &Path) {
    match cmd {
        Command::Let(name, ty, rhs) => {
            println!("{}", print_command(&Command::Let(name.clone(), ty.clone(), rhs.clone())));
            handle_let(name, ty, rhs, session);
        }
        Command::Fact(name, e, cond) => {
            println!("{}", print_command(&Command::Fact(name.clone(), e.clone(), cond.clone())));
            install_fact(name, &e, cond.as_ref(), &mut session.theory, &session.let_declared);
        }
        Command::Print(e) => {
            let resolved = match &e {
                Expr::Ident(name) => session.bindings.get(name).cloned().unwrap_or(e.clone()),
                _ => e.clone(),
            };
            println!("{}", print_expr(&resolved));
        }
        Command::Evaluate(e) => match run_evaluate(&e, &session.kernel_bindings) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
        Command::Simplify(e) => match run_simplify(&e, &session.kernel_bindings, &session.theory) {
            Ok(out) => println!("{}", out),
            Err(msg) => println!("error: {}", msg),
        },
        Command::Apply(name, e) => {
            match run_apply(&name, &e, false, &session.kernel_bindings, &session.theory) {
                Ok(out) => println!("{}", out),
                Err(msg) => println!("error: {}", msg),
            }
        }
        Command::ApplyRev(name, e) => {
            match run_apply(&name, &e, true, &session.kernel_bindings, &session.theory) {
                Ok(out) => println!("{}", out),
                Err(msg) => println!("error: {}", msg),
            }
        }
        Command::Import(path_str) => {
            println!("import \"{path_str}\"");
            let path = current_dir.join(&path_str);
            if let Err(e) = run_file(&path, session, ctx) {
                println!("error: {e}");
            }
        }
    }
}

fn handle_let(name: String, ty: Option<Expr>, rhs: Option<Expr>, session: &mut Session) {
    session.let_declared.insert(name.clone());
    match (ty.as_ref(), rhs.as_ref()) {
        // `let Name : Set` — opaque set declaration
        (Some(Expr::Ident(t)), None) if t == "Set" => {
            session.kernel_bindings.insert(sym(&name), Term::App(sym(&name), vec![]));
        }

        // `let Name : Set = {x ∈ S | P}` — predicate set definition
        (Some(Expr::Ident(t)), Some(Expr::SetBuilder(var, domain, pred))) if t == "Set" => {
            session.kernel_bindings.insert(sym(&name), Term::App(sym(&name), vec![]));
            match (lower(domain), lower(pred)) {
                (Ok(dom_term), Ok(pred_term)) => {
                    session.theory.add_predicate_set(sym(&name), sym(var), dom_term, pred_term);
                }
                _ => println!("error: cannot lower set-builder definition for `{name}`"),
            }
        }

        // `let name : ty` — opaque declaration with type annotation (e.g. `let i : ℂ`)
        (Some(_ty), None) => {
            session.kernel_bindings.insert(sym(&name), Term::App(sym(&name), vec![]));
        }

        // `let name [: ty] = rhs` — value definition
        (_, Some(rhs_expr)) => {
            match lower(rhs_expr) {
                Ok(t) => {
                    session.kernel_bindings.insert(sym(&name), t);
                    session.bindings.insert(name, rhs_expr.clone());
                }
                Err(err) => println!("error: {}", err.0),
            }
        }

        (None, None) => {
            session.kernel_bindings.insert(sym(&name), Term::App(sym(&name), vec![]));
        }
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
/// automatically and merged with any explicit `if` clause. Identifiers that were
/// declared with `let` (and are not `∀`-bound) are lowered as 0-arity constants
/// rather than pattern variables.
fn install_fact(
    name: Option<String>,
    e: &Expr,
    condition: Option<&Expr>,
    theory: &mut Theory,
    let_declared: &HashSet<String>,
) {
    let (body_expr, binder_cond, pvars) = extract_binder_conditions(e, theory);

    // Merge binder-generated conditions with explicit `if` condition.
    let merged_cond: Option<Expr> = match (binder_cond, condition.cloned()) {
        (Some(bc), Some(ec)) => Some(Expr::BinOp(Op::And, Box::new(bc), Box::new(ec))),
        (Some(bc), None) => Some(bc),
        (None, Some(ec)) => Some(ec),
        (None, None) => None,
    };

    let t = match lower_fact_body(&body_expr, &pvars, let_declared) {
        Ok(t) => t,
        Err(err) => {
            println!("note: fact not installed: {}", err.0);
            return;
        }
    };
    let cond_term = match merged_cond.as_ref().map(|c| lower_fact_body(c, &pvars, let_declared)) {
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

/// If `e` is `Forall(vars, domain, body)`, strip the binder and return
/// `(body, binder_conditions, pvars)`. The bound variable names are always
/// returned as `pvars` so they remain pattern wildcards in `lower_fact_body`.
/// If `domain` is a predicate-defined set, membership conditions are generated
/// for each variable and returned; otherwise the condition is `None`.
fn extract_binder_conditions(e: &Expr, theory: &Theory) -> (Expr, Option<Expr>, HashSet<String>) {
    if let Expr::Forall(vars, domain, body) = e {
        let pvars: HashSet<String> = vars.iter().cloned().collect();
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
                return (*body.clone(), cond, pvars);
            }
        }
        // Domain is not a predicate set — strip binder, no conditions generated.
        (*body.clone(), None, pvars)
    } else {
        (e.clone(), None, HashSet::new())
    }
}
