#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    IntegerLiteral(u32),
    StringLiteral(String),
    Keyword(String),
    Operator(char),
    Punctuation(char),
    Error,
}
