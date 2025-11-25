use crate::token::Value;
use crate::error::Result;
use crate::interpreter::Interpreter;
use crate::expr::Stmt;
use crate::environment::Environment;
use crate::error::{RuntimeError, Error};
use std::cell::Ref;
use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

/// Loxの呼び出し可能オブジェクト（関数）を表すenum

#[derive(Clone)]
pub enum LoxCallable {
    /// ネイティブ関数（Rustで実装された組み込み関数）
    NativeFunction(NativeFunction),
    // 将来的にユーザー定義関数を追加
    // UserFunction { ... },
    LoxFunction(LoxFunction),
}
pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, arguments: Vec<Value>, interpreter: Option<RefCell<Interpreter>>) -> Result<Value>;
    fn name(&self) -> &str;
}

#[derive(Clone)]
pub struct NativeFunction {
    name: String,
    arity: usize,
    func: fn(Vec<Value>) -> Result<Value>,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, arguments: Vec<Value>, interpreter: Option<RefCell<Interpreter>>) -> Result<Value> {
        (self.func)(arguments)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
    closure: Rc<Environment>,
}
impl LoxFunction {
    /// 新しいLoxFunctionを作成する
    pub fn new(name: String, params: Vec<String>, body: Vec<Stmt>, closure: Rc<Environment>) -> Self {
        Self { name, params, body, closure }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, arguments: Vec<Value>, interpreter: Option<RefCell<Interpreter>>) -> Result<Value> {
        if let Some(interpreter) = interpreter {
            // 関数内部用の環境を作成（クロージャを親として使う）
            let env = Rc::new(Environment::new(Some(Rc::clone(&self.closure))));
            for (param, arg) in self.params.iter().zip(arguments.into_iter()) {
                env.define(param.clone(), arg);
            }
            let res = interpreter.borrow_mut().execute_block(&self.body, env);
            match res {
                Ok(_) => Ok(Value::Nil),
                Err(err) => match err {
                    Error::ReturnError(return_err) => {
                        if let Some(value) = return_err.value {
                            Ok(value)
                        } else {
                            Ok(Value::Nil)
                        }
                    }
                    _ => Err(err),
                },
            }
        } else {
            Err(Error::RuntimeError(RuntimeError::new(
                crate::token::Token::new(crate::token::TokenType::IDENTIFIER, self.name.clone(), 0, None),
                "Interpreter is required to call LoxFunction.".to_string(),
            )))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}


impl LoxCallable {
    /// 関数の引数の数を返す
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::NativeFunction(native) => native.arity(),
            LoxCallable::LoxFunction(lox_func) => lox_func.arity(),
        }
    }

    /// 関数を呼び出す
    pub fn call(&self, arguments: Vec<Value>, interpreter: Option<RefCell<Interpreter>>) -> Result<Value> {
        match self {
            LoxCallable::NativeFunction(native) => native.call(arguments, interpreter),
            LoxCallable::LoxFunction(lox_func) => lox_func.call(arguments, interpreter),
        }
    }

    /// 関数の名前を返す（デバッグ用）
    pub fn name(&self) -> &str {
        match self {
            LoxCallable::NativeFunction(native) => native.name(),
            LoxCallable::LoxFunction(lox_func) => &lox_func.name,
        }
    }
}

impl std::fmt::Debug for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxCallable::NativeFunction(native) => {
                write!(f, "<native fn {}({} args)>", native.name(), native.arity())
            }
            LoxCallable::LoxFunction(lox_func) => {
                write!(f, "<lox fn {}({} args)>", lox_func.name, lox_func.params.len())
            }
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                LoxCallable::NativeFunction(native1),
                LoxCallable::NativeFunction(native2),
            ) => native1.name() == native2.name(),
            _ => false,
        }
    }
}

/// Native function implementations

/// clock() - Returns the current time in seconds since UNIX epoch
pub fn native_clock(_args: Vec<Value>) -> Result<Value> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch");
    Ok(Value::Number(duration.as_secs_f64()))
}

/// Helper function to create the clock native function
pub fn create_clock_function() -> LoxCallable {
    LoxCallable::NativeFunction(NativeFunction {
        name: "clock".to_string(),
        arity: 0,
        func: native_clock,
    })
}
