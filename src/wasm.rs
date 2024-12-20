use wasm_bindgen::prelude::*;
use crate::tokenizer::tokenize;
use crate::parser::Parser;
use crate::environment::Env;
use crate::eval::evals;

#[wasm_bindgen]
pub fn evaluate(input: &str) -> String {
    let tokens = tokenize(&input.to_string());
    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse_lines();
    let mut env = Env::new();
    let result = evals(ast_nodes, &mut env);
    format!("{:?}", result)
} 