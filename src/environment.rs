use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::token::{LiteralType, Token, TokenType};
use crate::error::{RuntimeError, Result, Error};
use crate::token::Value;

#[derive(Debug, Clone)]
pub struct Environment{
    pub values: RefCell<HashMap<String, Value>>,
    pub enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<Environment>>) -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            enclosing,
        }
    }

    pub fn define(&self, name: String, value: Value){
        self.values.borrow_mut().insert(name, value);
    }

    pub fn put(&self, name: &str, value: Value) -> Result<()>{
        if self.values.borrow().contains_key(name){
            self.values.borrow_mut().insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.put(name, value);
        }
        return Err(Error::RuntimeError(RuntimeError::new(
            Token::new(TokenType::IDENTIFIER, name.to_string(), 0, None),
            format!("Undefined variable '{}'.", name),
        )));
    }

    pub fn get(&self, name: &str) -> Result<Value>{
        if let Some(value) = self.values.borrow().get(name) {
            return Ok(value.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }
        Err(Error::RuntimeError(RuntimeError::new(
            Token::new(TokenType::IDENTIFIER, name.to_string(), 0, None),
            format!("Undefined variable '{}'.", name),
        )))
    }

    fn ancestor(&self, distance: usize) -> Rc<Environment> {
        let mut environment = Rc::new(self.clone());
        for _ in 0..distance {
            if let Some(enclosing) = &environment.enclosing {
                environment = Rc::clone(enclosing);
            }
        }
        environment
    }

    pub fn assign_at(&self, distance: usize, name: &str, value: Value) -> Result<()> {
        let environment = self.ancestor(distance);
        environment.put(name, value)
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Value> {
        let environment = self.ancestor(distance);
        environment.get(name)
    }
}