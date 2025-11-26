use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use crate::callable::{LoxCallable, LoxInstance, LoxClass};
use std::rc::Rc;
use crate::lox::Lox;

/// Runtime value type for the interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(LoxCallable),
    Instance(Rc<RefCell<LoxInstance>>),
    Class(LoxClass),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(callable) => write!(f, "<fn {}>", callable.name()),
            Value::Instance(instance) => write!(f, "<instance of {}>", instance.borrow().class.name),
            Value::Class(class) => write!(f, "<class {}>", class.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

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

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType{
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Eq for LiteralType {}

impl Hash for LiteralType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LiteralType::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            LiteralType::Number(n) => {
                1u8.hash(state);
                n.to_bits().hash(state);
            }
            LiteralType::Bool(b) => {
                2u8.hash(state);
                b.hash(state);
            }
            LiteralType::Nil => {
                3u8.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object;

#[derive(Debug, Clone, PartialEq)]
pub struct Token{
    pub type_: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<LiteralType>,
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_.hash(state);
        self.lexeme.hash(state);
        self.line.hash(state);
        self.literal.hash(state);
    }
}

impl Token {
    pub fn new(type_: TokenType, lexeme: String, line: usize, literal: Option<LiteralType>) -> Self{
        Self {type_, lexeme, line, literal}
    }

    pub fn to_string(&self) -> String{
        format!("{:?} {} {:?}", self.type_, self.lexeme, self.literal)
    }
}
