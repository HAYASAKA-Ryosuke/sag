mod builtin;
mod environment;
mod tokenizer;
mod wasm;
mod evals;
mod parsers;
mod ast;
mod value;
mod token;

use crate::builtin::register_builtins;
use crate::environment::Env;
use crate::evals::{eval, evals};
use crate::parsers::Parser;
use crate::tokenizer::tokenize;
use std::env;

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Env::new();
    let builtins = register_builtins(&mut env);
    for line in std::io::stdin().lines() {
        let line = line?;
        let tokens = tokenize(&line);
        println!("{:?}", tokens);
        let mut parser = Parser::new(tokens.to_vec(), builtins.clone());
        let ast_node = parser.parse();
        if let Err(e) = ast_node {
            e.display_with_source(&line);
            continue;
        }
        println!("ast: {:?}", ast_node);
        let result = eval(ast_node.unwrap(), &mut env);
        println!("---------");
        println!("res: {:?}", result);
    }
    Ok(())
}

fn run_file(file_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(file_path)?;

    let tokens = tokenize(&file);
    println!("tokens: {:?}", tokens);
    let mut env = Env::new();
    let builtins = register_builtins(&mut env);
    let mut parser = Parser::new(tokens.to_vec(), builtins.clone());
    let ast_nodes = parser.parse_lines();
    if let Err(e) = ast_nodes {
        e.display_with_source(&file);
        return Ok(());
    }
    println!("ast: {:?}", ast_nodes);
    let result = evals(ast_nodes.unwrap(), &mut env);
    println!("result: {:?}", result);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        //println!("args: {:?}", args);
        let file_path = args[1].clone();
        if let Err(e) = run_file(file_path) {
            eprintln!("Error: {}", e);
        }
    } else {
        if let Err(e) = run_repl() {
            eprintln!("Error: {}", e);
        }
    }
}
