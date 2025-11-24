mod lox;
mod interpreter;
mod parser;
mod expr;
mod token;
mod scanner;
mod error;
mod environment;
mod callable;
use std::env;
use std::process;
use crate::lox::Lox;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut lox = Lox::new();

    if args.len() > 1 {
        eprintln!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 1 {
        if let Err(e) = lox.run_file(&args[0]) {
            eprintln!("I/O error: {e}");
            process::exit(74); // UNIX 的な「I/O エラー」コード
        }
    } else {
        if let Err(e) = lox.run_prompt() {
            eprintln!("I/O error in prompt: {e}");
            process::exit(1);
        }
    }
}
