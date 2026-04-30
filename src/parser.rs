use crate::ast::{Command, Expr, Op};
use crate::lexer::{tokenize, Token};

#[derive(Debug)]
pub struct ParseError(pub String);

pub fn parse_command(src: &str) -> Result<Command, ParseError> {
    let tokens = tokenize(src).map_err(|e| ParseError(e.0))?;
    let mut p = Parser { tokens, pos: 0 };
    let cmd = p.parse_command()?;
    p.expect_eof()?;
    Ok(cmd)
}

pub fn parse_expr(src: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(src).map_err(|e| ParseError(e.0))?;
    let mut p = Parser { tokens, pos: 0 };
    let e = p.parse_expr(0)?;
    p.expect_eof()?;
    Ok(e)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect_eof(&self) -> Result<(), ParseError> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err(ParseError(format!(
                "unexpected trailing token: {:?}",
                self.tokens[self.pos]
            )))
        }
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let name = match self.advance() {
                    Some(Token::Ident(s)) => s,
                    other => {
                        return Err(ParseError(format!(
                            "expected identifier after `let`, got {other:?}"
                        )))
                    }
                };
                match self.advance() {
                    Some(Token::Equals) => {}
                    other => {
                        return Err(ParseError(format!(
                            "expected `=` in let-binding, got {other:?}"
                        )))
                    }
                }
                let e = self.parse_expr(0)?;
                Ok(Command::Let(name, e))
            }
            Some(Token::Fact) => {
                self.advance();
                let e = self.parse_expr(0)?;
                Ok(Command::Fact(e))
            }
            Some(Token::Print) => {
                self.advance();
                let e = self.parse_expr(0)?;
                Ok(Command::Print(e))
            }
            other => Err(ParseError(format!(
                "expected command (let/fact/print), got {other:?}"
            ))),
        }
    }

    fn peek_binop(&self) -> Option<Op> {
        match self.peek()? {
            Token::Plus => Some(Op::Add),
            Token::Dot => Some(Op::Mul),
            Token::Caret => Some(Op::Pow),
            Token::Equals => Some(Op::Eq),
            _ => None,
        }
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_atom()?;
        while let Some(op) = self.peek_binop() {
            if op.prec() < min_prec {
                break;
            }
            self.advance();
            let next_min = if op.right_assoc() {
                op.prec()
            } else {
                op.prec() + 1
            };
            let rhs = self.parse_expr(next_min)?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(Expr::Ident(s)),
            Some(Token::Int(n)) => Ok(Expr::Int(n)),
            Some(Token::LParen) => {
                let e = self.parse_expr(0)?;
                match self.advance() {
                    Some(Token::RParen) => Ok(e),
                    other => Err(ParseError(format!("expected `)`, got {other:?}"))),
                }
            }
            other => Err(ParseError(format!("expected atom, got {other:?}"))),
        }
    }
}
