use crate::ast::ASTNode;
use crate::value::Value;
use crate::token::TokenKind;
use crate::environment::Env;
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn prefix_op(op: TokenKind, expr: Box<ASTNode>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let value = eval(*expr, env)?;
    match (op.clone(), value) {
        (TokenKind::Minus, Value::Number(v)) => Ok(Value::Number(-v)),
        _ => Err(RuntimeError::new(format!("Unexpected prefix op: {:?}", op).as_str(), line, column)),
    }
}
