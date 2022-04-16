mod builtin;
mod engine;
mod pexpr;
mod parse;

use std::error::Error;
use crate::engine::Engine;
use crate::parse::parse;

fn main() -> Result<(), Box<dyn Error>> {
    let (_, expr) = parse("1 + 2 + 3 * 4 * 5 + 6")?;
    println!("{:?}", expr);
    let mut engine = Engine::new();
    engine.init();
    let tree = engine.read_expr(&expr);
    Engine::print_tree(&tree);
    let ntree = Engine::normalize(&tree);
    Engine::print_tree(&ntree);
    Ok(())
}
