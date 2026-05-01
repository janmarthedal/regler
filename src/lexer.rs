use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Int(BigInt),
    Plus,
    Minus,      // -
    Dot,        // ·
    Slash,      // /
    Caret,      // ^
    Equals,     // =
    NotEquals,  // ≠
    Implies,    // ⇒
    And,        // ∧
    Or,         // ∨
    Colon,      // :
    LeftArrow,  // ←
    Arrow,      // →
    ForAll,     // ∀
    In,         // ∈
    Subset,     // ⊆
    Period,     // .
    Comma,      // ,
    LParen,
    RParen,
    LBrace,     // {
    RBrace,     // }
    Bar,        // |
    Lt,         // <
    Gt,         // >
    Le,         // ≤
    Ge,         // ≥
    Let,
    Fact,
    Print,
    Evaluate,
    Simplify,
    Apply,
    To,
    If,
}

#[derive(Debug)]
pub struct LexError(pub String);

/// Split `src` into the token stream consumed by the parser. Whitespace is
/// skipped; identifiers, integer literals, punctuation, and reserved keywords
/// are recognized.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let mut chars = src.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '+' {
            chars.next();
            tokens.push(Token::Plus);
        } else if c == '-' {
            chars.next();
            tokens.push(Token::Minus);
        } else if c == '·' {
            chars.next();
            tokens.push(Token::Dot);
        } else if c == '/' {
            chars.next();
            tokens.push(Token::Slash);
        } else if c == '^' {
            chars.next();
            tokens.push(Token::Caret);
        } else if c == '=' {
            chars.next();
            tokens.push(Token::Equals);
        } else if c == '≠' {
            chars.next();
            tokens.push(Token::NotEquals);
        } else if c == '⇒' {
            chars.next();
            tokens.push(Token::Implies);
        } else if c == '∧' {
            chars.next();
            tokens.push(Token::And);
        } else if c == '∨' {
            chars.next();
            tokens.push(Token::Or);
        } else if c == '∀' {
            chars.next();
            tokens.push(Token::ForAll);
        } else if c == '∈' {
            chars.next();
            tokens.push(Token::In);
        } else if c == '.' {
            chars.next();
            tokens.push(Token::Period);
        } else if c == ',' {
            chars.next();
            tokens.push(Token::Comma);
        } else if c == ':' {
            chars.next();
            tokens.push(Token::Colon);
        } else if c == '←' {
            chars.next();
            tokens.push(Token::LeftArrow);
        } else if c == '→' {
            chars.next();
            tokens.push(Token::Arrow);
        } else if c == '⊆' {
            chars.next();
            tokens.push(Token::Subset);
        } else if c == '≤' {
            chars.next();
            tokens.push(Token::Le);
        } else if c == '≥' {
            chars.next();
            tokens.push(Token::Ge);
        } else if c == '<' {
            chars.next();
            tokens.push(Token::Lt);
        } else if c == '>' {
            chars.next();
            tokens.push(Token::Gt);
        } else if c == '{' {
            chars.next();
            tokens.push(Token::LBrace);
        } else if c == '}' {
            chars.next();
            tokens.push(Token::RBrace);
        } else if c == '|' {
            chars.next();
            tokens.push(Token::Bar);
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
                "simplify" => Token::Simplify,
                "apply" => Token::Apply,
                "to" => Token::To,
                "if" => Token::If,
                _ => Token::Ident(s),
            });
        } else if c == '#' {
            // line comment — discard the rest of the input
            break;
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
