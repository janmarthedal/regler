mod builtin;
mod main_expr;
mod parse;
mod pexpr;
mod symbols;

use crate::main_expr::MainExpr;
use crate::parse::parse;
use crate::symbols::{FuncAttr, Symbols};
use std::error::Error;

fn init_symbols() -> Symbols {
    let mut symbols = Symbols::new();
    symbols.add_function(builtin::ADD.to_string(), FuncAttr::new(true, true));
    symbols.add_function(builtin::MUL.to_string(), FuncAttr::new(true, true));
    symbols.add_function(builtin::SUB.to_string(), FuncAttr::new(false, false));
    symbols.add_function(builtin::DIV.to_string(), FuncAttr::new(false, false));
    symbols.add_function(builtin::POW.to_string(), FuncAttr::new(false, false));
    symbols.add_function(builtin::NEG.to_string(), FuncAttr::new(false, false));
    symbols
}

fn main() -> Result<(), Box<dyn Error>> {
    let (_, expr) = parse("1 + 2 + 3 * 4 * 5 + 6")?;
    println!("{:?}", expr);
    let symbols = init_symbols();
    let mut mexpr = MainExpr::from_pexpr(&expr, &symbols);
    mexpr.print_expr();
    mexpr.normalize();
    mexpr.print_expr();
    Ok(())
}
