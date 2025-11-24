use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::error::RuntimeError;
use crate::token::Token;
use std::{
    env,
    fs,
    io::{self, BufRead, Write},
    process,
};

pub struct Lox{
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self{
        Self {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &str) -> io::Result<()> {
        let source = fs::read_to_string(path)?;
        self.run(&source);
        if self.had_error {
            process::exit(65);
        }
        if self.had_runtime_error {
            process::exit(70);
        }
        Ok(())
    }

    pub fn run_prompt(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        loop {
            print!("> ");
            stdout.flush()?;
            let mut source = String::new();
            let mut line = String::new();
            let n = stdin.lock().read_line(&mut line)?;
            if n == 0 {
                break;
            }
            source.push_str(&line);

            // Continue reading lines if braces are unbalanced
            while !self.is_balanced(&source) {
                print!("... ");
                stdout.flush()?;
                line.clear();
                let n = stdin.lock().read_line(&mut line)?;
                if n == 0 {
                    break;
                }
                source.push_str(&line);
            }

            self.run(&source);
            self.had_error = false;
            self.had_runtime_error = false;
        }
        Ok(())
    }

    /// Check if braces are balanced in the input
    fn is_balanced(&self, source: &str) -> bool {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut chars = source.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '"' && !in_string {
                in_string = true;
            } else if ch == '"' && in_string {
                // Check if it's not an escaped quote
                in_string = false;
            } else if !in_string {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
        }

        brace_count == 0
    }

    pub fn run(&mut self, source: &str){
        let mut scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        if self.had_error {
            return;
        }
        self.interpreter.interpret(&statements);
    }

    pub fn error(&mut self, line: usize, message: &str){
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, where_: &str, message: &str){
        eprintln!("[line {}] Error{}: {}", line, where_, message);
        self.had_error = true;
    }

    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{}", error);
        self.had_runtime_error = true;
    }
}