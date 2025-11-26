use std::{rc::Rc};
use crate::token::Token;
use crate::expr::{Assignment, Expr, ExprVisitor, Expression, FunctionStmt, Stmt, StmtVisitor, VarDecl, Variable
, ClassDecl};

use crate::{expr::Block, interpreter::Interpreter};
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(PartialEq, Eq, Clone, Copy)]
enum FunctionType{
    None,
    Function,
    Method,
    Initializer,
}
#[derive(PartialEq, Eq, Clone, Copy)]
enum ClassType{
    None,
    Class,
}

pub struct Resolver{
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        let scope = HashMap::new();
        Self { interpreter, scopes: vec![scope], current_function: FunctionType::None, current_class: ClassType::None }
    }
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn resolve_statements(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }
    fn resolve_statement(&mut self, statement: &Stmt) {
        statement.accept(self);
    }
    fn resolve_expression(&mut self, expr: &Expr) {
        expr.accept(self);
    }   
    fn declare(&mut self, name: &Token){
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            // Variable with this name already declared in this scope
            panic!("Variable with this name already declared in this scope.");
        }
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token){
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true);
    }
    fn resolve_local(&mut self, expr: &Expr, name: &Token){
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.borrow_mut().resolve(expr.clone(),  i);
                return;
            }
        }
    }
    fn resolve_function(&mut self, function: &FunctionStmt, type_: FunctionType){
        let enclosing_function = self.current_function;
        self.current_function = type_;
        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_statements(&function.body);
        self.end_scope();
        self.current_function = enclosing_function;
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_block_stmt(&mut self, block: &Block) {
        self.begin_scope();
        self.resolve_statements(&block.statements);
        self.end_scope();
    }
    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> () {
        self.declare(&var_decl.name);
        if let Some(initializer) = &var_decl.initializer {
            self.resolve_expression(initializer);
        }
        self.define(&var_decl.name);
    }
    fn visit_function_stmt(&mut self, function_stmt: &FunctionStmt) -> () {
        self.declare(&function_stmt.name);
        self.define(&function_stmt.name);
        self.resolve_function(function_stmt, FunctionType::Function);
        return ();
    }
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> () {
        self.resolve_expression(&stmt.expression);
        return ();
    }
    fn visit_if_stmt(&mut self, if_stmt: &crate::expr::IfStatement) -> () {
        self.resolve_expression(&if_stmt.condition);
        self.resolve_statement(&if_stmt.then_branch);
        if let Some(else_branch) = &if_stmt.else_branch {
            self.resolve_statement(else_branch);
        }
        return ();
    }
    fn visit_print_stmt(&mut self, stmt: &crate::expr::Print) -> () {
        self.resolve_expression(&stmt.expression);
        return ();
    }
    fn visit_return_stmt(&mut self, return_stmt: &crate::expr::ReturnStmt) -> () {
        if self.current_function == FunctionType::None {
            panic!("Cannot return from top-level code.");
        }
        if let Some(value) = &return_stmt.value {
            if self.current_function == FunctionType::Initializer {
                panic!("Cannot return a value from an initializer.");
            }
            self.resolve_expression(value);
        }
        return ();
    }
    fn visit_while_stmt(&mut self, while_stmt: &crate::expr::WhileStmt) -> () {
        self.resolve_expression(&while_stmt.condition);
        self.resolve_statement(&while_stmt.body);
        return ();
    }
    fn visit_break_stmt(&mut self, _break_stmt: &crate::expr::BreakStmt) -> () {
        return ();
    }
    fn visit_continue_stmt(&mut self, _continue_stmt: &crate::expr::ContinueStmt) -> () {
        return ();
    }
    fn visit_for_stmt(&mut self, for_stmt: &crate::expr::ForStmt) -> () {
        self.begin_scope();
        if let Some(initializer) = &for_stmt.initializer {
            self.resolve_statement(initializer);
        }
        if let Some(condition) = &for_stmt.condition {
            self.resolve_expression(condition);
        }
        self.resolve_statement(&for_stmt.body);
        self.end_scope();
    }
    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> () {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;
        self.declare(&class_decl.name);
        self.begin_scope();
        self.scopes.last_mut().unwrap().insert("this".to_string(), true);
        for method in &class_decl.methods {
            let declaration = FunctionType::Method;
            self.resolve_function(method, declaration);
        }
        self.end_scope();
        self.define(&class_decl.name);
        self.current_class = enclosing_class;
        return ();
    }
}
impl ExprVisitor<()> for Resolver {
    fn visit_variable_expr(&mut self, expr: &Variable) -> () {
        if !self.scopes.is_empty() {
            if let Some(is_defined) = self.scopes.last().unwrap().get(&expr.name.lexeme) {
                if !*is_defined {
                    // Variable is declared but not defined
                    panic!("Cannot read local variable in its own initializer.");
                }
            }
        }
        self.resolve_local(&Expr::Variable(expr.clone()), &expr.name);
        return ();
    }
    fn visit_assignment_expr(&mut self, expr: &Assignment) -> () {
        self.resolve_expression(&expr.value);
        self.resolve_local(&Expr::Assignment(expr.clone()), &expr.name);
        return ();
    }
    fn visit_binary_expr(&mut self, expr: &crate::expr::Binary) -> () {
        self.resolve_expression(&expr.left);
        self.resolve_expression(&expr.right);
        return ();
    }
    fn visit_unary_expr(&mut self, expr: &crate::expr::Unary) -> () {
        self.resolve_expression(&expr.right);
        return ();
    }
    fn visit_call_expr(&mut self, expr: &crate::expr::Call) -> () {
        self.resolve_expression(&expr.callee);
        for argument in &expr.arguments {
            self.resolve_expression(argument);
        }
        return ();
    }
    fn visit_grouping_expr(&mut self, expr: &crate::expr::Grouping) -> () {
        self.resolve_expression(&expr.expression);
        return ();
    }
    fn visit_literal_expr(&mut self, expr: &crate::expr::Literal) -> () {
        return ();
    }
    fn visit_and_expr(&mut self, expr: &crate::expr::AND) -> () {
        self.resolve_expression(&expr.left);
        self.resolve_expression(&expr.right);
        return ();
    }
    fn visit_or_expr(&mut self, expr: &crate::expr::OR) -> () {
        self.resolve_expression(&expr.left);
        self.resolve_expression(&expr.right);
        return ();
    }
    fn visit_get_expr(&mut self, expr: &crate::expr::Get) -> () {
        self.resolve_expression(&expr.object);
        return ();
    }
    fn visit_set_expr(&mut self, expr: &crate::expr::Set) -> () {
        self.resolve_expression(&expr.value);
        self.resolve_expression(&expr.object);
        return ();
    }
    fn visit_this_expr(&mut self, expr: &crate::expr::This) -> () {
        if self.current_class == ClassType::None {
            panic!("Cannot use 'this' outside of a class.");
        }
        self.resolve_local(&Expr::This(expr.clone()), &expr.keyword);
        return ();
    }
}