use crate::token::{LiteralType, Token, TokenType};
use crate::expr::{Expr, Literal, Unary, Binary, Grouping, Stmt, BreakStmt, ContinueStmt, ReturnStmt, Set, This, Print,
    Expression, VarDecl, Variable, Assignment, Block, IfStatement, OR, AND, WhileStmt, ForStmt, Call, ClassDecl};

pub struct Parser{
    tokens: Vec<Token>,
    current: usize,
}
impl Parser{
    pub fn new(tokens: Vec<Token>) -> Self{
        Self {tokens, current: 0}
    }
    pub fn parse(&mut self) -> Vec<Stmt>{
        let mut statements = Vec::new();
        while !self.is_at_end(){
            statements.push(self.declaration());
        }
        statements
    }
    fn declaration(&mut self) -> Stmt{
        if self.match_token(&[TokenType::VAR]){
            return self.var_declaration();
        }
        if self.match_token(&[TokenType::FUN]){
            return self.function_declaration();
        }
        if self.match_token(&[TokenType::CLASS]){
            return self.class_declaration();
        }
        return self.statement();
    }
    fn class_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name.").clone();
        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body.");
        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            let method = self.function_declaration();
            methods.push(method);
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.");
        return Stmt::ClassDecl(
            ClassDecl::new(
                name,
                methods
                    .into_iter()
                    .filter_map(|stmt| if let Stmt::FunctionStmt(func) = stmt { Some(func) } else { None })
                    .collect(),
            ),
        );
    }
    fn function_declaration(&mut self) -> Stmt{
        let name = self.consume(TokenType::IDENTIFIER, "Expect function name.").clone();
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after function name.");

        let mut parameters = Vec::new();
        if !self.check(&TokenType::RIGHT_PAREN){
            loop {
                if parameters.len() >= 255{
                    panic!("Can't have more than 255 parameters.");
                }
                let param = self.consume(TokenType::IDENTIFIER, "Expect parameter name.").clone();
                parameters.push(param);
                if !self.match_token(&[TokenType::COMMA]){
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.");

        self.consume(TokenType::LEFT_BRACE, "Expect '{' before function body.");
        let body = self.block_statement();

        Stmt::FunctionStmt(
            crate::expr::FunctionStmt::new(name, parameters, Box::new(vec![body]))
        )
    }
    fn var_declaration(&mut self) -> Stmt{
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.").clone();
        let mut initializer: Option<Expr> = None;
        if self.match_token(&[TokenType::EQUAL]){
            initializer = Some(self.expression());
        }
        self.consume(TokenType::SEMICOLON, "Expect ';' after variable declaration.");
        Stmt::VarDeclaration(
            VarDecl::new(name, initializer)
        )
    }
    fn statement(&mut self) -> Stmt{
        if self.match_token(&[TokenType::IF]){
            return self.if_statement()
        }
        if self.match_token(&[TokenType::PRINT]){
            return self.print_statement();
        }
        if self.match_token(&[TokenType::LEFT_BRACE]){
            return self.block_statement();
        }
        if self.match_token(&[TokenType::WHILE]){
            return self.while_statement()
        }
        if self.match_token(&[TokenType::FOR]){
            return self.for_statement()
        }
        if self.match_token(&[TokenType::RETURN]){
            return self.return_statement()
        }
        if self.match_token(&[TokenType::BREAK]){
            self.consume(TokenType::SEMICOLON, "Expect ';' after 'break'.");
            return Stmt::BreakStmt(
                BreakStmt::new()
            );
        }
        if self.match_token(&[TokenType::CONTINUE]){
            self.consume(TokenType::SEMICOLON, "Expect ';' after 'continue'.");
            return Stmt::ContinueStmt(
                ContinueStmt::new()
            );
        }
        self.expression_statement()
    }
    fn return_statement(&mut self) -> Stmt{
        let keyword = self.previous().clone();
        let mut value = None;
        if !self.check(&TokenType::SEMICOLON){
            value = Some(self.expression());
        }
        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.");
        return Stmt::ReturnStmt(
            ReturnStmt::new(keyword, value.map(Box::new))
        );
    }

    fn for_statement(&mut self) -> Stmt{
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.");

        let initializer = if self.match_token(&[TokenType::SEMICOLON]){
            None
        } else if self.match_token(&[TokenType::VAR]){
            Some(Box::new(self.var_declaration()))
        } else {
            Some(Box::new(self.expression_statement()))
        };

        let condition = if !self.check(&TokenType::SEMICOLON){
            Some(Box::new(self.expression()))
        } else {
            None
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.");

        let increment = if !self.check(&TokenType::RIGHT_PAREN){
            Some(Box::new(self.expression()))
        } else {
            None
        };
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after for clauses.");

        let body = Box::new(self.statement());

        Stmt::ForStmt(ForStmt::new(initializer, condition, increment, body))
    }
    fn while_statement(&mut self) -> Stmt{
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.");
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.");
        let body = Box::new(self.statement());
        Stmt::WhileStmt(WhileStmt::new(Box::new(condition), body))
    }

    fn if_statement(&mut self) -> Stmt{
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition.");

        let then_branch = Box::new(self.statement());

        let else_branch = if self.match_token(&[TokenType::ELSE]) {
            Some(Box::new(self.statement()))
        } else {
            None
        };

        Stmt::IfStatement(IfStatement::new(Box::new(condition), then_branch, else_branch))
    }
    fn block_statement(&mut self) -> Stmt{
        let mut statements = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end(){
            statements.push(self.declaration());
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");
        Stmt::Block(Block::new(statements))
    }

    fn print_statement(&mut self) -> Stmt{
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");
        Stmt::Print(Print::new(Box::new(value)))
    }
    fn expression_statement(&mut self) -> Stmt{
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        Stmt::Expression(Expression::new(Box::new(expr)))
    }

    pub fn expression(&mut self) -> Expr{
        self.assignment()
    }
    pub fn assignment(&mut self) -> Expr{
        let expr = self.or() ;
        if self.match_token(&[TokenType::EQUAL]){
            let value = self.assignment();
            if let Expr::Variable(var_expr) = expr{
                let name = var_expr.name.clone();
                return Expr::Assignment(Assignment::new(name, Box::new(value)));
            }else if let Expr::Get(get_expr) = expr {
                let name = get_expr.name.clone();
                return Expr::Set(Set::new(get_expr.object.clone(), name, Box::new(value)));
            }
            panic!("Invalid assignment target.");
        }
        expr
    }

    fn or(&mut self) -> Expr{
        let mut expr = self.and();
        while self.match_token(&[TokenType::OR]){
            let operator = self.previous().clone();
            let right = self.and();
            expr = Expr::OR(OR::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }

    fn and(&mut self) -> Expr{
        let mut expr = self.equality();
        while self.match_token(&[TokenType::AND]){
            let operator = self.previous().clone();
            let right = self.equality();
            expr = Expr::AND(AND::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }

    pub fn equality(&mut self) -> Expr{
        let mut expr = self.comparison();
        while self.match_token(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]){
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }
    fn comparison(&mut self) -> Expr{
        let mut expr = self.term();
        while self.match_token(&[TokenType::GREATER, TokenType::GREATER_EQUAL, TokenType::LESS, TokenType::LESS_EQUAL]){
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }
    fn term(&mut self) ->  Expr{
        let mut expr = self.factor();
        while self.match_token(&[TokenType::PLUS, TokenType::MINUS]){
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }
    fn factor(&mut self) -> Expr{
        let mut expr = self.unary();
        while self.match_token(&[TokenType::STAR, TokenType::SLASH]){
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }
    fn unary(&mut self) -> Expr{
        if self.match_token(&[TokenType::BANG, TokenType::MINUS]){
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::Unary(Unary::new(operator, Box::new(right)));
        }
        self.call()
    }
    fn call(&mut self) -> Expr{
        let mut expr = self.primary();
        loop {
            if self.match_token(&[TokenType::LEFT_PAREN]){
                expr = self.finish_call(expr);
            } else if self.match_token(&[TokenType::DOT]){
                let name = self.consume(TokenType::IDENTIFIER, "Expect property name after '.'.").clone();
                expr = Expr::Get(crate::expr::Get::new(Box::new(expr), name));
            } else {
                break;
            }
        }
        expr
    }
    fn finish_call(&mut self, callee: Expr) -> Expr{
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RIGHT_PAREN){
            loop {
                if arguments.len() >= 255{
                    panic!("Can't have more than 255 arguments.");
                }
                arguments.push(self.expression());
                if !self.match_token(&[TokenType::COMMA]){
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.").clone();
        Expr::Call(Call::new(Box::new(callee), paren, arguments))
    }
    fn primary(&mut self) -> Expr{
        if self.match_token(&[TokenType::FALSE]){
            return Expr::Literal(Literal::new(LiteralType::Bool(false)));
        }
        if self.match_token(&[TokenType::TRUE]){
            return Expr::Literal(Literal::new(LiteralType::Bool(true)));
        }
        if self.match_token(&[TokenType::NIL]){
            return Expr::Literal(Literal::new(LiteralType::Nil));
        }
        if self.match_token(&[TokenType::THIS]){
            let keyword = self.previous().clone();
            return Expr::This(This::new(keyword));
        }
        if self.match_token(&[TokenType::NUMBER]){
            let value = match &self.previous().literal{
                Some(LiteralType::Number(n)) => *n,
                _ => panic!("Expected number literal"),
            };
            return Expr::Literal(Literal::new(LiteralType::Number(value)));
        }
        if self.match_token(&[TokenType::STRING]){
            let value = match &self.previous().literal{
                Some(LiteralType::String(s)) => s.clone(),
                _ => panic!("Expected string literal"),
            };
            return Expr::Literal(Literal::new(LiteralType::String(value)));
        }
        if self.match_token(&[TokenType::LEFT_PAREN]){
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(Grouping::new(Box::new(expr)));
        }
        if self.match_token(&[TokenType::IDENTIFIER]){
            let name = self.previous().clone();
            return Expr::Variable(Variable::new(name));
        }
        panic!("Expected expression.");
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool{
        for type_ in types{
            if self.check(type_){
                self.advance();
                return true;
            }
        }
        false
    }
    fn check(&self, type_: &TokenType) -> bool{
        if self.is_at_end(){
            return false;
        }
        &self.peek().type_ == type_
    }
    fn advance(&mut self) -> &Token{
        if !self.is_at_end(){
            self.current +=1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool{
        self.peek().type_ == TokenType::EOF
    }
    fn peek(&self) -> &Token{
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    fn consume(&mut self, type_: TokenType, message: &str) -> &Token{
        if self.check(&type_){
            return self.advance();
        }
        panic!("{}", message);
    }
}
