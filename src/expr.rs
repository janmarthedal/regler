#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Func(&'static str, Vec<Expr>),
}
