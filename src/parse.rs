use std::str::FromStr;

use nom::IResult;
use nom::character::complete::{digit1, space0};
use nom::combinator::map;
use nom::sequence::delimited;

use crate::expr::Expr;

pub fn parse(input: &str) -> IResult<&str, Expr> {
    parse_expr(input)
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    parse_num(input)
}

fn parse_num(input: &str) -> IResult<&str, Expr> {
    map(delimited(space0, digit1, space0), parse_number)(input)
}

fn parse_number(parsed_num: &str) -> Expr {
    let num = i64::from_str(parsed_num).unwrap();
    Expr::Num(num)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use super::Expr::Num;

    #[test]
    fn parse_num() {
        assert_eq!(
            parse("1234"),
            Ok(("", Num(1234)))
        );
        assert_eq!(
            parse("  0"),
            Ok(("", Num(0)))
        );
        assert_eq!(
            parse("  0123  "),
            Ok(("", Num(123)))
        );
    }

}