use std::error::Error;

mod expr;
mod parse;
use crate::parse::parse;

fn main() -> Result<(), Box<dyn Error>> {
    // let (_, expr) = parse(" ( 1 + 2 ) * --7 - 3 + 4 ^ 6 ^ -9 *( 2 --5 )")?;
    let (_, expr) = parse("foo(1, 2)")?;
    println!("{:?}", expr);
    Ok(())
}
