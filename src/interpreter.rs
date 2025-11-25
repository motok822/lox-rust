use std::cell::RefCell;
use std::rc::Rc;
use crate::expr::{Binary, Expr, ExprVisitor, Grouping, Literal, StmtVisitor,
    Unary, IfStatement, BreakStmt, ContinueStmt, Call};
use crate::error::{Result, ReturnError, RuntimeError, Error};
use crate::environment::Environment;
use crate::token::{LiteralType, Token, TokenType, Value};
use crate::expr::{Stmt, FunctionStmt};
use crate::callable::{LoxFunction, create_clock_function};
use std::collections::HashMap;

/// Interpreter that evaluates expressions using the Visitor pattern
#[derive(Clone)]
pub struct Interpreter{
    pub environment: Rc<Environment>,
    pub locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Rc::new(Environment::new(None));

        // Register native functions
        environment.define(
            "clock".to_string(),
            Value::Callable(create_clock_function())
        );

        Self {
            environment,
            locals: HashMap::new(),
        }
    }

    /// Main entry point for interpreting an expression
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr.accept(self)
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }
    pub fn execute(&mut self, stmt: &Stmt) -> Result<Value> {
        stmt.accept(self)
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, env: Rc<Environment>) -> Result<()> {
        let previous = std::mem::replace(&mut self.environment, env);

        // ブロック内のステートメントを実行
        let result = (|| {
            for stmt in statements {
                let res = self.execute(stmt);
                if let Err(err) = res{
                    return Err(err);
                }
            }
            Ok(())
        })();

        // 元のenvironmentに戻す
        self.environment = previous;
        result
    }

    pub fn stringify(&self, value: &Value) -> String {
        match value {
            Value::Nil => "nil".to_string(),
            Value::Number(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text.truncate(text.len() - 2);
                }
                text
            },
            _ => format!("{}", value),
        }
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    pub fn look_up_variable(&self, name: &String, expr: &Expr) -> Result<Value> {
        if let Some(distance) = self.locals.get(expr) {
            self.get_at(*distance, &name)
        } else {
            self.environment.get(&name)
        }
    }

    fn get_at(&self, distance: usize, name: &str) -> Result<Value> {
        self.ancestor(distance).get(name)
    }

    fn ancestor(&self, distance: usize) -> Rc<Environment> {
        let mut environment = Rc::clone(&self.environment);
        for _ in 0..distance {
            let enclosing = environment.enclosing.as_ref().unwrap();
            environment = Rc::clone(enclosing);
        }
        environment
    }

    fn assign_at(&mut self, distance: usize, name: &Token, value: Value) -> Result<()> {
        let env = self.ancestor(distance);
        env.put(&name.lexeme, value)
    }


    /// Helper: Check if a value is truthy (Lox semantics: nil and false are falsey)
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    /// Helper: Check if two values are equal
    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        left == right
    }

    /// Helper: Convert LiteralType to Value
    fn literal_to_value(&self, literal: &LiteralType) -> Value {
        match literal {
            LiteralType::String(s) => Value::String(s.clone()),
            LiteralType::Number(n) => Value::Number(*n),
            LiteralType::Bool(b) => Value::Bool(*b),
            LiteralType::Nil => Value::Nil,
        }
    }
}

impl StmtVisitor<Result<Value>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &crate::expr::Expression) -> Result<Value> {
        self.evaluate(&stmt.expression);
        Ok(Value::Nil)
    }
    fn visit_print_stmt(&mut self, stmt: &crate::expr::Print) -> Result<Value> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", self.stringify(&value));
        Ok(Value::Nil)
    }
    fn visit_var_decl(&mut self, var_decl: &crate::expr::VarDecl) -> Result<Value> {
        let value = if let Some(initializer) = &var_decl.initializer {
            self.evaluate(initializer)?
        } else {
            Value::Nil
        };
        self.environment.define(var_decl.name.lexeme.clone(), value.clone());
        Ok(value)
    }
    fn visit_block_stmt(&mut self, block: &crate::expr::Block) -> Result<Value> {
        let new_env = Rc::new(Environment::new(Some(Rc::clone(&self.environment))));
        self.execute_block(&block.statements, new_env)?;
        Ok(Value::Nil)
    }
    fn visit_if_stmt(&mut self, if_stmt: &IfStatement) -> Result<Value> {
        let condition = self.evaluate(&if_stmt.condition)?;
        if self.is_truthy(&condition) {
            self.execute(&if_stmt.then_branch)?;
        } else if let Some(else_branch) = &if_stmt.else_branch {
            self.execute(else_branch)?;
        }
        Ok(Value::Nil)
    }
    fn visit_while_stmt(&mut self, while_stmt: &crate::expr::WhileStmt) -> Result<Value> {
        while {
            let condition = self.evaluate(&while_stmt.condition)?;
            self.is_truthy(&condition)
        } {
            let res = self.execute(&while_stmt.body);
            let mut is_break= false;
            let mut is_continue = false;
            if let Err(err) = res {
                match err {
                    Error::RuntimeError(runtime_err) => match runtime_err.token.type_ {
                        TokenType::BREAK => {
                            is_break = true;
                        },
                        TokenType::CONTINUE => {
                            is_continue = true;
                        },
                        _ => return Err(Error::RuntimeError(runtime_err)),
                    },
                    _ => return Err(err),
                }
            }
            if is_break {
                break;
            }
            if is_continue {
                continue;
            }
        }
        Ok(Value::Nil)
    }
    fn visit_for_stmt(&mut self, for_stmt: &crate::expr::ForStmt) -> Result<Value> {
        // Create a new environment for the for loop
        let loop_env = Rc::new(Environment::new(Some(Rc::clone(&self.environment))));
        let previous = std::mem::replace(&mut self.environment, loop_env);

        let result = (|| -> Result<Value> {
            // Execute the initializer if it exists
            if let Some(initializer) = &for_stmt.initializer {
                self.execute(initializer)?;
            }

            loop {
                if let Some(condition_expr) = &for_stmt.condition {
                    let condition = self.evaluate(condition_expr)?;
                    if !self.is_truthy(&condition) {
                        break;
                    }
                }

                // Execute the body
                let res = self.execute(&for_stmt.body);
                if let Err(err) = res{
                    match err {
                        Error::RuntimeError(runtime_err) => match runtime_err.token.type_ {
                            TokenType::BREAK => {
                                break;
                            },
                            TokenType::CONTINUE => {
                                continue;
                            },
                            _ => return Err(Error::RuntimeError(runtime_err)),
                        },
                        _ => return Err(err),
                    }
                }

                // Execute the increment if it exists (continue時も実行する)
                if let Some(increment_expr) = &for_stmt.increment {
                    self.evaluate(increment_expr)?;
                }

            }

            Ok(Value::Nil)
        })();

        self.environment = previous;
        result
    }
    fn visit_break_stmt(&mut self, _break_stmt: &BreakStmt) -> Result<Value> {
        Err(Error::RuntimeError(RuntimeError::new(
            Token::new(TokenType::BREAK, "break".to_string(), 0, None),
            "Break statement encountered.".to_string(),
        )))
    }
    fn visit_continue_stmt(&mut self, _continue_stmt: &ContinueStmt) -> Result<Value> {
        Err(Error::RuntimeError(RuntimeError::new(
            Token::new(TokenType::CONTINUE, "continue".to_string(), 0, None),
            "Continue statement encountered.".to_string(),
        )))
    }
    fn visit_function_stmt(&mut self, function_stmt: &FunctionStmt) -> Result<Value> {
        let func_name = function_stmt.name.lexeme.clone();
        let params = function_stmt.params.iter().map(|param| param.lexeme.clone()).collect();
        let body = (*function_stmt.body).clone();

        let lox_function = LoxFunction::new(
            func_name.clone(), params, body, Rc::clone(&self.environment)
        );
        self.environment.define(
            func_name,
            Value::Callable(crate::callable::LoxCallable::LoxFunction(lox_function)),
        );

        Ok(Value::Nil)
    }
    fn visit_return_stmt(&mut self, return_stmt: &crate::expr::ReturnStmt) -> Result<Value> {
        let value = if let Some(expr) = &return_stmt.value {
            self.evaluate(expr)?
        } else {
            Value::Nil
        };
        Err(Error::ReturnError(ReturnError::new(Some(value))))

    }

}

/// Implement the ExprVisitor trait for the Interpreter
impl ExprVisitor<Result<Value>> for Interpreter {

    fn visit_literal_expr(&mut self, expr: &Literal) -> Result<Value> {
        Ok(self.literal_to_value(&expr.value))
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Result<Value> {
        self.evaluate(&expr.expression)
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> Result<Value> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TokenType::MINUS => {
                match right {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operand must be a number.".to_string(),
                    ))),
                }
            }
            TokenType::BANG => {
                Ok(Value::Bool(!self.is_truthy(&right)))
            }
            _ => Err(Error::RuntimeError(RuntimeError::new(
                expr.operator.clone(),
                format!("Unknown unary operator: {:?}", expr.operator.type_),
            ))),
        }
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> Result<Value> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            // Arithmetic operators
            TokenType::MINUS => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::SLASH => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        if r == 0.0 {
                            Err(Error::RuntimeError(RuntimeError::new(
                                expr.operator.clone(),
                                "Division by zero.".to_string(),
                            )))
                        } else {
                            Ok(Value::Number(l / r))
                        }
                    }
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::STAR => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::PLUS => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::String(l), Value::Number(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::Number(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be two numbers or two strings.".to_string(),
                    ))),
                }
            }

            // Comparison operators
            TokenType::GREATER => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::GREATER_EQUAL => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::LESS => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }
            TokenType::LESS_EQUAL => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                    _ => Err(Error::RuntimeError(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.".to_string(),
                    ))),
                }
            }

            // Equality operators
            TokenType::BANG_EQUAL => {
                Ok(Value::Bool(!self.is_equal(&left, &right)))
            }
            TokenType::EQUAL_EQUAL => {
                Ok(Value::Bool(self.is_equal(&left, &right)))
            }

            _ => Err(Error::RuntimeError(RuntimeError::new(
                expr.operator.clone(),
                format!("Unknown binary operator: {:?}", expr.operator.type_),
            ))),
        }
    }
    fn visit_variable_expr(&mut self, expr: &crate::expr::Variable) -> Result<Value> {
        let name = &expr.name.lexeme;
        return self.look_up_variable(name, &Expr::Variable(expr.clone()));
    }
    fn visit_assignment_expr(&mut self, expr: &crate::expr::Assignment) -> Result<Value> {
        let value = self.evaluate(&expr.value)?;
        let distance = self.locals.get(&Expr::Assignment(expr.clone()));
        if let Some(distance) = distance {
            self.assign_at(*distance, &expr.name, value.clone())?;
        } else {
            self.environment.put(&expr.name.lexeme, value.clone())?;
        }
        Ok(value)  
    }
    fn visit_or_expr(&mut self, expr: &crate::expr::OR) -> Result<Value> {
        let left = self.evaluate(&expr.left)?;
        if self.is_truthy(&left) {
            return Ok(left);
        }
        self.evaluate(&expr.right)
    }
    fn visit_and_expr(&mut self, expr: &crate::expr::AND) -> Result<Value> {
        let left = self.evaluate(&expr.left)?;
        if !self.is_truthy(&left) {
            return Ok(left);
        }
        self.evaluate(&expr.right)
    }
    fn visit_call_expr(&mut self, expr: &Call) -> Result<Value> {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments = Vec::new();
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }

        // calleeがCallableかチェック
        match callee {
            Value::Callable(function) => {
                // 引数の数をチェック
                if arguments.len() != function.arity() {
                    return Err(Error::RuntimeError(RuntimeError::new(
                        expr.paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            function.arity(),
                            arguments.len()
                        ),
                    )));
                }

                // 関数を呼び出す
                function.call(arguments, Some(RefCell::new(self.clone())))
            }
            _ => Err(Error::RuntimeError(RuntimeError::new(
                expr.paren.clone(),
                "Can only call functions and classes.".to_string(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_literal_number() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Literal(Literal::new(LiteralType::Number(42.0)));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_literal_string() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Literal(Literal::new(LiteralType::String("hello".to_string())));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_unary_negation() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Unary(Unary::new(
            Token::new(TokenType::MINUS, "-".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(10.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(-10.0));
    }

    #[test]
    fn test_binary_addition() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(1.0)))),
            Token::new(TokenType::PLUS, "+".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(2.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_binary_subtraction() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(10.0)))),
            Token::new(TokenType::MINUS, "-".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(3.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_binary_multiplication() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(4.0)))),
            Token::new(TokenType::STAR, "*".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_binary_division() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(20.0)))),
            Token::new(TokenType::SLASH, "/".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(4.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_string_concatenation() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::String("Hello, ".to_string())))),
            Token::new(TokenType::PLUS, "+".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::String("World!".to_string())))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_comparison_greater() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(10.0)))),
            Token::new(TokenType::GREATER, ">".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_comparison_less() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(3.0)))),
            Token::new(TokenType::LESS, "<".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(8.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_equality() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
            Token::new(TokenType::EQUAL_EQUAL, "==".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_inequality() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
            Token::new(TokenType::BANG_EQUAL, "!=".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(3.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_unary_bang() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Unary(Unary::new(
            Token::new(TokenType::BANG, "!".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Bool(true)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_grouping() {
        let mut interpreter = Interpreter::new();
        // (2 + 3) * 4 = 20
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Grouping(Grouping::new(
                Box::new(Expr::Binary(Binary::new(
                    Box::new(Expr::Literal(Literal::new(LiteralType::Number(2.0)))),
                    Token::new(TokenType::PLUS, "+".to_string(), 1, None),
                    Box::new(Expr::Literal(Literal::new(LiteralType::Number(3.0)))),
                )))
            ))),
            Token::new(TokenType::STAR, "*".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(4.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_truthy_values() {
        let interpreter = Interpreter::new();

        // nil is falsey
        assert_eq!(interpreter.is_truthy(&Value::Nil), false);

        // false is falsey
        assert_eq!(interpreter.is_truthy(&Value::Bool(false)), false);

        // true is truthy
        assert_eq!(interpreter.is_truthy(&Value::Bool(true)), true);

        // numbers are truthy
        assert_eq!(interpreter.is_truthy(&Value::Number(0.0)), true);
        assert_eq!(interpreter.is_truthy(&Value::Number(42.0)), true);

        // strings are truthy
        assert_eq!(interpreter.is_truthy(&Value::String("".to_string())), true);
        assert_eq!(interpreter.is_truthy(&Value::String("hello".to_string())), true);
    }

    #[test]
    fn test_stringify() {
        let interpreter = Interpreter::new();

        assert_eq!(interpreter.stringify(&Value::Nil), "nil");
        assert_eq!(interpreter.stringify(&Value::Bool(true)), "true");
        assert_eq!(interpreter.stringify(&Value::Bool(false)), "false");
        assert_eq!(interpreter.stringify(&Value::Number(42.0)), "42");
        assert_eq!(interpreter.stringify(&Value::Number(3.14)), "3.14");
        assert_eq!(interpreter.stringify(&Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_division_by_zero() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(10.0)))),
            Token::new(TokenType::SLASH, "/".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(0.0)))),
        ));
        let result = interpreter.evaluate(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error_unary_minus() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Unary(Unary::new(
            Token::new(TokenType::MINUS, "-".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::String("hello".to_string())))),
        ));
        let result = interpreter.evaluate(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error_addition() {
        let mut interpreter = Interpreter::new();
        // number + boolean should fail
        let expr = Expr::Binary(Binary::new(
            Box::new(Expr::Literal(Literal::new(LiteralType::Number(5.0)))),
            Token::new(TokenType::PLUS, "+".to_string(), 1, None),
            Box::new(Expr::Literal(Literal::new(LiteralType::Bool(true)))),
        ));
        let result = interpreter.evaluate(&expr);
        assert!(result.is_err());
    }
}