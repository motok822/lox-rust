use std::fmt;
use crate::callable::LoxCallable;

/// Runtime value type for the interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(LoxCallable),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(callable) => write!(f, "<fn {}>", callable.name()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]

pub enum TokenType {
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    // ... other token types ...
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    // literal
    IDENTIFIER,
    STRING,
    NUMBER,
    // keywords
    BREAK,
    CONTINUE,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
}

#[derive(Debug, Clone)]
pub enum LiteralType{
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub struct Object;

#[derive(Debug, Clone)]
pub struct Token{
    pub type_: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<LiteralType>,
}

impl Token {
    pub fn new(type_: TokenType, lexeme: String, line: usize, literal: Option<LiteralType>) -> Self{
        Self {type_, lexeme, line, literal}
    }

    pub fn to_string(&self) -> String{
        format!("{:?} {} {:?}", self.type_, self.lexeme, self.literal)
    }
}
