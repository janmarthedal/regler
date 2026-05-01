use crate::ast::{Command, Expr, Op, UnaryOp};
use crate::lexer::{tokenize, Token};

#[derive(Debug)]
pub struct ParseError(pub String);

/// Parse a single REPL command (e.g. `let x = 1 + 2`) from source text.
/// Returns `None` for blank/comment-only input.
pub fn parse_command(src: &str) -> Result<Option<Command>, ParseError> {
    let tokens = tokenize(src).map_err(|e| ParseError(e.0))?;
    if tokens.is_empty() {
        return Ok(None);
    }
    let mut p = Parser { tokens, pos: 0 };
    let cmd = p.parse_command()?;
    p.expect_eof()?;
    Ok(Some(cmd))
}

/// Parse a standalone expression from source text (no surrounding command keyword).
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

    fn peek2(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
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
                // Distinguish `fact name : prop` from `fact prop`:
                // lookahead for Ident followed by Colon.
                let name = if matches!(self.peek(), Some(Token::Ident(_)))
                    && matches!(self.peek2(), Some(Token::Colon))
                {
                    let name = match self.advance() {
                        Some(Token::Ident(s)) => s,
                        _ => unreachable!(),
                    };
                    self.advance(); // consume ':'
                    Some(name)
                } else {
                    None
                };
                let prop = self.parse_expr(0)?;
                let cond = if matches!(self.peek(), Some(Token::If)) {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                Ok(Command::Fact(name, prop, cond))
            }
            Some(Token::Print) => {
                self.advance();
                let e = self.parse_expr(0)?;
                Ok(Command::Print(e))
            }
            Some(Token::Evaluate) => {
                self.advance();
                let e = self.parse_expr(0)?;
                Ok(Command::Evaluate(e))
            }
            Some(Token::Simplify) => {
                self.advance();
                let e = self.parse_expr(0)?;
                Ok(Command::Simplify(e))
            }
            Some(Token::Apply) => {
                self.advance();
                let reverse = if matches!(self.peek(), Some(Token::LeftArrow)) {
                    self.advance();
                    true
                } else {
                    false
                };
                let name = match self.advance() {
                    Some(Token::Ident(s)) => s,
                    other => {
                        return Err(ParseError(format!(
                            "expected fact name after `apply`, got {other:?}"
                        )))
                    }
                };
                match self.advance() {
                    Some(Token::To) => {}
                    other => {
                        return Err(ParseError(format!(
                            "expected `to` in apply command, got {other:?}"
                        )))
                    }
                }
                let e = self.parse_expr(0)?;
                if reverse {
                    Ok(Command::ApplyRev(name, e))
                } else {
                    Ok(Command::Apply(name, e))
                }
            }
            other => Err(ParseError(format!(
                "expected command (let/fact/print/evaluate/simplify/apply), got {other:?}"
            ))),
        }
    }

    /// If the next token is one of the recognized infix operators, return it
    /// without consuming.
    fn peek_binop(&self) -> Option<Op> {
        match self.peek()? {
            Token::Implies => Some(Op::Implies),
            Token::Or => Some(Op::Or),
            Token::And => Some(Op::And),
            Token::Plus => Some(Op::Add),
            Token::Minus => Some(Op::Sub),
            Token::Dot => Some(Op::Mul),
            Token::Slash => Some(Op::Div),
            Token::Caret => Some(Op::Pow),
            Token::Equals => Some(Op::Eq),
            Token::NotEquals => Some(Op::Ne),
            _ => None,
        }
    }

    /// Precedence-climbing expression parser.
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

    /// Parse a primary expression: identifier, integer literal,
    /// parenthesized subexpression, or `∀` binder.
    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        // Unary minus: `-3` folds into a negative literal; `-expr` is UnaryOp::Neg.
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            if matches!(self.peek(), Some(Token::Int(_))) {
                return match self.advance() {
                    Some(Token::Int(n)) => Ok(Expr::Int(-n)),
                    _ => unreachable!(),
                };
            }
            let operand = self.parse_atom()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(operand)));
        }
        // `∀ var, var, … ∈ domain. body`
        if matches!(self.peek(), Some(Token::ForAll)) {
            self.advance();
            let mut vars = vec![];
            loop {
                match self.advance() {
                    Some(Token::Ident(s)) => vars.push(s),
                    other => return Err(ParseError(format!(
                        "expected variable name in ∀ binding, got {other:?}"
                    ))),
                }
                if !matches!(self.peek(), Some(Token::Comma)) {
                    break;
                }
                self.advance(); // consume ','
            }
            match self.advance() {
                Some(Token::In) => {}
                other => return Err(ParseError(format!(
                    "expected ∈ after variables in ∀ binding, got {other:?}"
                ))),
            }
            let domain = self.parse_expr(0)?;
            match self.advance() {
                Some(Token::Period) => {}
                other => return Err(ParseError(format!(
                    "expected '.' after domain in ∀ binding, got {other:?}"
                ))),
            }
            let body = self.parse_expr(0)?;
            return Ok(Expr::Forall(vars, Box::new(domain), Box::new(body)));
        }
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
