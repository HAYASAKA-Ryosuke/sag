use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::Env;
use crate::token::TokenKind;
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn binary_op(op: TokenKind, left: Box<ASTNode>, right: Box<ASTNode>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let left_val = eval(*left, env)?;
    let right_val = eval(*right, env)?;

    match (left_val.clone(), right_val.clone(), op.clone()) {
        (Value::String(l), Value::String(r), TokenKind::Plus) => Ok(Value::String(l + &r)),
        (Value::Number(l), Value::Number(r), TokenKind::Plus) => Ok(Value::Number(l + r)),
        (Value::Number(l), Value::Number(r), TokenKind::Minus) => Ok(Value::Number(l - r)),
        (Value::Number(l), Value::Number(r), TokenKind::Mul) => Ok(Value::Number(l * r)),
        (Value::Number(l), Value::Number(r), TokenKind::Div) => Ok(Value::Number(l / r)),
        (Value::Number(l), Value::Number(r), TokenKind::Mod) => Ok(Value::Number(l % r)),
        _ => Err(RuntimeError::new(format!("Unsupported operation: {:?} {:?} {:?}", left_val.clone(), op, right_val.clone()).as_str(), line, column)),
    }
}
