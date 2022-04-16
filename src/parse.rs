use std::str::FromStr;

use nom::IResult;
use nom::branch::alt;
use nom::character::complete::{alpha1, char, digit1, space0};
use nom::combinator::{all_consuming, map};
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::builtin;
use crate::pexpr::PExpr;

pub fn parse(input: &str) -> IResult<&str, PExpr> {
    all_consuming(parse_expr)(input)
}

fn parse_expr(input: &str) -> IResult<&str, PExpr> {
    parse_term(input)
}

fn parse_term(input: &str) -> IResult<&str, PExpr> {
    let (input, expr1) = parse_factor(input)?;
    fold_many0(
        tuple((alt((char('+'), char('-'))), parse_factor)),
        move || expr1.clone(),
        create_binop
    )(input)
}

fn parse_factor(input: &str) -> IResult<&str, PExpr> {
    let (input, expr1) = parse_power(input)?;
    fold_many0(
        tuple((alt((char('*'), char('/'))), parse_power)),
        move || expr1.clone(),
        create_binop
    )(input)
}

fn parse_power(input: &str) -> IResult<&str, PExpr> {
    let (input, expr1) = parse_unary(input)?;
    let (input, exprs) = many0(preceded(char('^'), parse_unary))(input)?;
    let mut exprs = exprs;
    if let Some(expr_last) = exprs.pop() {
        exprs.insert(0, expr1);
        Ok((input, exprs.iter().cloned().rfold(expr_last, |ex1, ex2| {
            PExpr::Func(builtin::POW.to_string(), vec![ex2, ex1])
        })))
    } else {
        Ok((input, expr1))
    }
}

fn parse_unary(input: &str) -> IResult<&str, PExpr> {
    let (input, ms) = preceded(space0, many0(char('-')))(input)?;
    let (input, expr) = terminated(parse_primary, space0)(input)?;
    Ok((input, ms.iter().rfold(expr, |ex, _| {
        PExpr::Func(builtin::NEG.to_string(), vec![ex])
    })))
}

// primary → NUMBER | "(" expression ")" ;
fn parse_primary(input: &str) -> IResult<&str, PExpr> {
    alt((
        delimited(char('('), parse_expr, char(')')),
        parse_function,
        parse_num
    ))(input)
}

fn parse_function(input: &str) -> IResult<&str, PExpr> {
    let (input, name) = alpha1(input)?;
    let (input, exprs) = delimited(char('('), parse_expr_list, char(')'))(input)?;
    Ok((input, PExpr::Func(name.to_string(), exprs)))
}

fn parse_expr_list(input: &str) -> IResult<&str, Vec<PExpr>> {
    let (input, expr1) = parse_expr(input)?;
    let (input, exprs) = many0(preceded(char(','), parse_expr))(input)?;
    let mut exprs = exprs;
    exprs.insert(0, expr1);
    Ok((input, exprs))
}

fn parse_num(input: &str) -> IResult<&str, PExpr> {
    map(digit1, parse_number)(input)
}

fn parse_number(parsed_num: &str) -> PExpr {
    let num = i64::from_str(parsed_num).unwrap();
    PExpr::Num(num)
}

fn create_binop(e1: PExpr, (op, e2): (char, PExpr)) -> PExpr {
    match op {
        '+' => PExpr::Func(builtin::ADD.to_string(), vec![e1, e2]),
        '-' => PExpr::Func(builtin::SUB.to_string(), vec![e1, e2]),
        '*' => PExpr::Func(builtin::MUL.to_string(), vec![e1, e2]),
        '/' => PExpr::Func(builtin::DIV.to_string(), vec![e1, e2]),
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use super::PExpr::Num;

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
