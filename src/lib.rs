mod builtin;
mod environment;
mod eval;
mod parser;
mod tokenizer;
mod wasm;

pub use wasm::evaluate;
