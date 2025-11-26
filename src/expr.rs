use crate::token::{Token, LiteralType};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: Token,
    pub initializer: Option<Expr>,
}

impl VarDecl {
    pub fn new(name: Token, initializer: Option<Expr>) -> Self {
        Self { name, initializer }
    }
}

pub trait StmtVisitor<R> {
    fn visit_print_stmt(&mut self, stmt: &Print) -> R;
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> R;
    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> R;
    fn visit_block_stmt(&mut self, block: &Block) -> R;
    fn visit_if_stmt(&mut self, if_stmt: &IfStatement) -> R;
    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> R;
    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) -> R;
    fn visit_break_stmt(&mut self, _break_stmt: &BreakStmt) -> R;
    fn visit_continue_stmt(&mut self, _continue_stmt: &ContinueStmt) -> R;
    fn visit_function_stmt(&mut self, function_stmt: &FunctionStmt) -> R;
    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> R;
    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> R;    
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Print),
    Expression(Expression),
    IfStatement(IfStatement),
    VarDeclaration(VarDecl),
    Block(Block),
    WhileStmt(WhileStmt),
    ForStmt(ForStmt),
    BreakStmt(BreakStmt),
    ContinueStmt(ContinueStmt),
    FunctionStmt(FunctionStmt),
    ReturnStmt(ReturnStmt),
    ClassDecl(ClassDecl),
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn StmtVisitor<R>) -> R {
        match self {
            Stmt::Print(stmt) => visitor.visit_print_stmt(stmt),
            Stmt::Expression(stmt) => visitor.visit_expression_stmt(stmt),
            Stmt::VarDeclaration(var_decl) => visitor.visit_var_decl(var_decl),
            Stmt::Block(block) => visitor.visit_block_stmt(block),
            Stmt::IfStatement(if_stmt) => visitor.visit_if_stmt(if_stmt),
            Stmt::WhileStmt(while_stmt) => visitor.visit_while_stmt(while_stmt),
            Stmt::ForStmt(for_stmt) => visitor.visit_for_stmt(for_stmt),
            Stmt::BreakStmt(break_stmt) => visitor.visit_break_stmt(break_stmt),
            Stmt::ContinueStmt(continue_stmt) => visitor.visit_continue_stmt(continue_stmt),
            Stmt::FunctionStmt(function_stmt) => visitor.visit_function_stmt(function_stmt),
            Stmt::ReturnStmt(return_stmt) => visitor.visit_return_stmt(return_stmt),
            Stmt::ClassDecl(class_decl) => visitor.visit_class_decl(class_decl),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: Token,
    pub methods: Vec<FunctionStmt>,
}
impl ClassDecl {
    pub fn new(name: Token, methods: Vec<FunctionStmt>) -> Self {
        Self { name, methods }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Box<Expr>>,
}
impl ReturnStmt {
    pub fn new(keyword: Token, value: Option<Box<Expr>>) -> Self {
        Self { keyword, value }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Box<Vec<Stmt>>,
}
impl FunctionStmt {
    pub fn new(name: Token, params: Vec<Token>, body: Box<Vec<Stmt>>) -> Self {
        Self { name, params, body }
    }
}
#[derive(Debug, Clone)]
pub struct BreakStmt;
impl BreakStmt {
    pub fn new() -> Self {
        Self {}
    }
} 
#[derive(Debug, Clone)]
pub struct ContinueStmt;
impl ContinueStmt {
    pub fn new() -> Self {      
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub initializer: Option<Box<Stmt>>,
    pub condition: Option<Box<Expr>>,
    pub increment: Option<Box<Expr>>,
    pub body: Box<Stmt>,
}
impl ForStmt {
    pub fn new(initializer: Option<Box<Stmt>>, condition: Option<Box<Expr>>, increment: Option<Box<Expr>>, body: Box<Stmt>) -> Self {
        Self { initializer, condition, increment, body }
    }
}
#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Box<Expr>,
    pub body: Box<Stmt>,
}
impl WhileStmt {
    pub fn new(condition: Box<Expr>, body: Box<Stmt>) -> Self {
        Self { condition, body }
    }
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<Expr>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}
impl IfStatement {
    pub fn new(condition: Box<Expr>, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>) -> Self {
        Self { condition, then_branch, else_branch }
    }
    
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}
impl Block {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, Clone)]
pub struct Print {
    pub expression: Box<Expr>,
}

impl Print {
    pub fn new(expression: Box<Expr>) -> Self {
        Self { expression }
    }
}
#[derive(Debug, Clone)]
pub struct Expression {
    pub expression: Box<Expr>,
}
impl Expression {
    pub fn new(expression: Box<Expr>) -> Self {
        Self { expression }
    }
}

// Visitor trait for traversing expressions
pub trait ExprVisitor<R> {
    fn visit_binary_expr(&mut self, expr: &Binary) -> R;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> R;
    fn visit_literal_expr(&mut self, expr: &Literal) -> R;
    fn visit_unary_expr(&mut self, expr: &Unary) -> R;
    fn visit_variable_expr(&mut self, expr: &Variable) -> R;
    fn visit_assignment_expr(&mut self, expr: &Assignment) -> R;
    fn visit_or_expr(&mut self, expr: &OR) -> R;
    fn visit_and_expr(&mut self, expr: &AND) -> R;
    fn visit_call_expr(&mut self, expr: &Call) -> R;
    fn visit_get_expr(&mut self, expr: &Get) -> R;
    fn visit_set_expr(&mut self, expr: &Set) -> R;
    fn visit_this_expr(&mut self, expr: &This) -> R;
}


// Main Expr enum containing all expression types
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assignment(Assignment),
    Get(Get),
    Set(Set),
    This(This),
    Call(Call),
    OR(OR),
    AND(AND),
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Expr::Binary(e) => {
                0u8.hash(state);
                e.hash(state);
            }
            Expr::Grouping(e) => {
                1u8.hash(state);
                e.hash(state);
            }
            Expr::Literal(e) => {
                2u8.hash(state);
                e.hash(state);
            }
            Expr::Unary(e) => {
                3u8.hash(state);
                e.hash(state);
            }
            Expr::Variable(e) => {
                4u8.hash(state);
                e.hash(state);
            }
            Expr::Assignment(e) => {
                5u8.hash(state);
                e.hash(state);
            }
            Expr::Call(e) => {
                6u8.hash(state);
                e.hash(state);
            }
            Expr::OR(e) => {
                7u8.hash(state);
                e.hash(state);
            }
            Expr::AND(e) => {
                8u8.hash(state);
                e.hash(state);
            }
            Expr::Set(e) => {
                9u8.hash(state);
                e.hash(state);
            }
            Expr::Get(e) => {
                10u8.hash(state);
                e.hash(state);
            }
            Expr::This(e) => {
                11u8.hash(state);
                e.hash(state);
            }
        }
    }
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn ExprVisitor<R>) -> R {
        match self {
            Expr::Binary(expr) => visitor.visit_binary_expr(expr),
            Expr::Grouping(expr) => visitor.visit_grouping_expr(expr),
            Expr::Literal(expr) => visitor.visit_literal_expr(expr),
            Expr::Unary(expr) => visitor.visit_unary_expr(expr),
            Expr::Variable(expr) => visitor.visit_variable_expr(expr),
            Expr::Assignment(expr) => visitor.visit_assignment_expr(expr),
            Expr::OR(expr) => visitor.visit_or_expr(expr),
            Expr::AND(expr) => visitor.visit_and_expr(expr),
            Expr::Call(expr) => visitor.visit_call_expr(expr),
            Expr::Get(expr) => visitor.visit_get_expr(expr),
            Expr::Set(expr) => visitor.visit_set_expr(expr),
            Expr::This(expr) => visitor.visit_this_expr(expr) 
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct This {
    pub keyword: Token,
}
impl This {
    pub fn new(keyword: Token) -> Self {
        Self { keyword }
    }
    
}
impl Eq for This {}

impl Hash for This {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.keyword.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set{
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}
impl Set {
    pub fn new(object: Box<Expr>, name: Token, value: Box<Expr>) -> Self {
        Self { object, name, value }
    }
}   
impl Eq for Set {}
impl Hash for Set {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object.hash(state);
        self.name.hash(state);
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Get{
    pub object: Box<Expr>,
    pub name: Token,
}
impl Get {
    pub fn new(object: Box<Expr>, name: Token) -> Self {
        Self { object, name }
    }
}

impl Eq for Get {}

impl Hash for Get {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object.hash(state);
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call{
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

impl Eq for Call {}

impl Hash for Call {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.callee.hash(state);
        self.paren.hash(state);
        self.arguments.hash(state);
    }
}

impl Call {
    pub fn new(callee: Box<Expr>, paren: Token, arguments: Vec<Expr>) -> Self {
        Self { callee, paren, arguments }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OR {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Eq for OR {}

impl Hash for OR {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.operator.hash(state);
        self.right.hash(state);
    }
}

impl OR {
    pub fn new(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
        Self { left, operator, right }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AND {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Eq for AND {}

impl Hash for AND {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.operator.hash(state);
        self.right.hash(state);
    }
}

impl AND {
    pub fn new(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
        Self { left, operator, right }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Eq for Assignment {}

impl Hash for Assignment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.value.hash(state);
    }
}

impl Assignment {
    pub fn new(name: Token, value: Box<Expr>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: Token,
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Variable {
    pub fn new(name: Token) -> Self {
        Self { name }
    }
}

// Binary expression: left operator right
#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Eq for Binary {}

impl Hash for Binary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.operator.hash(state);
        self.right.hash(state);
    }
}

impl Binary {
    pub fn new(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
        Self { left, operator, right }
    }
}

// Grouping expression: ( expression )
#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

impl Eq for Grouping {}

impl Hash for Grouping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.expression.hash(state);
    }
}

impl Grouping {
    pub fn new(expression: Box<Expr>) -> Self {
        Self { expression }
    }
}

// Literal expression: numbers, strings, booleans, nil
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub value: LiteralType,
}

impl Eq for Literal {}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Literal {
    pub fn new(value: LiteralType) -> Self {
        Self { value }
    }
}

// Unary expression: operator right
#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Eq for Unary {}

impl Hash for Unary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operator.hash(state);
        self.right.hash(state);
    }
}

impl Unary {
    pub fn new(operator: Token, right: Box<Expr>) -> Self {
        Self { operator, right }
    }
}