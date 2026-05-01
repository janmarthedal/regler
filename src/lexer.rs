use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Int(BigInt),
    Plus,
    Dot,    // ·
    Caret,  // ^
    Equals, // =
    LParen,
    RParen,
    Let,
    Fact,
    Print,
    Evaluate,
}

#[derive(Debug)]
pub struct LexError(pub String);

/// Split `src` into the token stream consumed by the parser. Whitespace is
/// skipped; identifiers, integer literals, punctuation, and the reserved
/// command keywords (`let`, `fact`, `print`, `evaluate`) are recognized.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let mut chars = src.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '+' {
            chars.next();
            tokens.push(Token::Plus);
        } else if c == '·' {
            chars.next();
            tokens.push(Token::Dot);
        } else if c == '^' {
            chars.next();
            tokens.push(Token::Caret);
        } else if c == '=' {
            chars.next();
            tokens.push(Token::Equals);
        } else if c == '(' {
            chars.next();
            tokens.push(Token::LParen);
        } else if c == ')' {
            chars.next();
            tokens.push(Token::RParen);
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    s.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            let n: BigInt = s.parse().map_err(|e| LexError(format!("bad int: {e}")))?;
            tokens.push(Token::Int(n));
        } else if is_ident_start(c) {
            let mut s = String::new();
            while let Some(&d) = chars.peek() {
                if is_ident_continue(d) {
                    s.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(match s.as_str() {
                "let" => Token::Let,
                "fact" => Token::Fact,
                "print" => Token::Print,
                "evaluate" => Token::Evaluate,
                _ => Token::Ident(s),
            });
        } else {
            return Err(LexError(format!("unexpected character: {c:?}")));
        }
    }

    Ok(tokens)
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
