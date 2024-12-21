mod tokenizer;
mod parser;
mod builtin;
mod environment;
mod eval;
mod wasm;

pub use wasm::evaluate;