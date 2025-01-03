use crate::parser::{ASTNode, Value};
use crate::tokenizer::Token;
use crate::environment::Env;
use crate::evals::eval;

pub fn prefix_op(op: Token, expr: Box<ASTNode>, env: &mut Env) -> Value {
    let value = eval(*expr, env);
    match (op.clone(), value) {
        (Token::Minus, Value::Number(v)) => Value::Number(-v),
        _ => panic!("Unexpected prefix op: {:?}", op),
    }
}
