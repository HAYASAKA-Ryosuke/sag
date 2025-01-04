mod builtin;
mod environment;
mod parser;
mod tokenizer;
mod wasm;
mod evals;
mod ast;
mod value;
mod token;

pub use wasm::evaluate;
