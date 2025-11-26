use crate::lox::Lox;
use crate::token::LiteralType;
use crate::token::Token;
use crate::token::TokenType;
use crate::token::Object;

pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
    keywords: std::collections::HashMap<String, TokenType>,
    start : usize,
    current : usize,
    line : usize,
}
    
impl Scanner {
    pub fn new(source: &str) -> Self {
        let mut keywords = std::collections::HashMap::new();
        keywords.insert("and".to_string(), TokenType::AND);
        keywords.insert("class".to_string(), TokenType::CLASS);
        keywords.insert("else".to_string(), TokenType::ELSE);
        keywords.insert("false".to_string(), TokenType::FALSE);
        keywords.insert("for".to_string(), TokenType::FOR);
        keywords.insert("fun".to_string(), TokenType::FUN);
        keywords.insert("if".to_string(), TokenType::IF);
        keywords.insert("nil".to_string(), TokenType::NIL);
        keywords.insert("or".to_string(), TokenType::OR);
        keywords.insert("print".to_string(), TokenType::PRINT);
        keywords.insert("return".to_string(), TokenType::RETURN);
        keywords.insert("super".to_string(), TokenType::SUPER);
        keywords.insert("this".to_string(), TokenType::THIS);
        keywords.insert("true".to_string(), TokenType::TRUE);
        keywords.insert("var".to_string(), TokenType::VAR);
        keywords.insert("while".to_string(), TokenType::WHILE);
        keywords.insert("break".to_string(), TokenType::BREAK);
        keywords.insert("continue".to_string(), TokenType::CONTINUE);

        Self {
            source: source.to_string(),
            tokens: Vec::new(),
            keywords: keywords,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn advance(&mut self) -> char{
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }
    fn add_token(&mut self, type_: TokenType, literal: Option<LiteralType>){
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            type_,
            text.to_string(),
            self.line,
            literal,
        ));
    }

    fn match_char(&mut self, expected: char) -> bool{
        if self.is_at_end(){
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected{
            return false;
        }
        self.current +=1;
        true
    }
    fn peek(&self) -> char{
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn string(&mut self){
        while self.peek() != '"' && !self.is_at_end(){
            if self.peek() == '\n'{
                self.line +=1;
            }
            self.advance();
        }

        if self.is_at_end(){
            // エラー処理
            return;
        }
        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::STRING, Some(LiteralType::String(value)));
    }

    fn scan_token(&mut self){
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN, None),
            ')' => self.add_token(TokenType::RIGHT_PAREN, None),
            '{' => self.add_token(TokenType::LEFT_BRACE, None),
            '}' => self.add_token(TokenType::RIGHT_BRACE, None),
            ',' => self.add_token(TokenType::COMMA, None),
            '.' => self.add_token(TokenType::DOT, None),
            '-' => self.add_token(TokenType::MINUS, None),
            '+' => self.add_token(TokenType::PLUS, None),
            ';' => self.add_token(TokenType::SEMICOLON, None),
            '*' => self.add_token(TokenType::STAR, None),
            '!' => {
                if self.match_char('='){
                    self.add_token(TokenType::BANG_EQUAL, None);
                } else {
                    self.add_token(TokenType::BANG, None);
                }
            }
            '=' => {
                if self.match_char('='){
                    self.add_token(TokenType::EQUAL_EQUAL, None);
                } else {
                    self.add_token(TokenType::EQUAL, None);
                }
            }
            '<' => {
                if self.match_char('='){
                    self.add_token(TokenType::LESS_EQUAL, None);
                } else {
                    self.add_token(TokenType::LESS, None);
                }
            }
            '>' => {
                if self.match_char('='){
                    self.add_token(TokenType::GREATER_EQUAL, None);
                } else {
                    self.add_token(TokenType::GREATER, None);
                }
            }
            '/' => {
                if self.match_char('/'){
                    while self.peek() != '\n' && !self.is_at_end(){
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH, None);
                }
            }
            ' ' | '\r' | '\t' => {},
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),
            _ => {
                if self.is_dight(c) {
                    self.number();
                } else {
                    self.identifier();
                }
            },
        }
    }
    fn peek_next(&self) -> char{
        if self.current + 1 >= self.source.len(){
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }
    fn number(&mut self){
        while self.is_dight(self.peek()){
            self.advance();
        }
        if self.peek() == '.' && self.is_dight(self.peek_next()){
            self.advance();
            while self.is_dight(self.peek()){
                self.advance();
            }
        }
        self.add_token(
            TokenType::NUMBER,
            Some(LiteralType::Number(
                self.source[self.start..self.current]
                    .parse()
                    .unwrap(),
            )),
        );
    }
    fn is_dight(&mut self,c: char) -> bool {
        c >= '0' && c <= '9'
    }
    fn is_alpha(&mut self,c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }
    fn is_alphanumeric(&mut self,c: char) -> bool {
        self.is_alpha(c) || self.is_dight(c)
    }

    fn identifier(&mut self){
        while self.is_alphanumeric(self.peek()){
            self.advance();
        }
        let lexeme = self.source[self.start..self.current].to_string();
        let type_ = match self.keywords.get(&lexeme){
            Some(t) => t.clone(),
            None => TokenType::IDENTIFIER,
        };
        self.add_token(type_, None);
    }
    
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            self.line,
            None,
        ));
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
