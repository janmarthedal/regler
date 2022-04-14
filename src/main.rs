use std::error::Error;
use std::process;

mod expr;
mod parse;
use crate::parse::parse;

fn main() -> Result<(), Box<dyn Error>> {
    let (rest, expr) = parse("1234")?;
    if !rest.is_empty() {
        eprintln!("parsing error, input remaining {:?}", rest);
        process::exit(1);
    }
    println!("{:?}", expr);
    Ok(())
}
