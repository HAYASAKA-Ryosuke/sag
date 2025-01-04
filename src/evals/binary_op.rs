use std::sync::{Arc, Mutex};
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::Env;
use crate::token::Token;
use crate::evals::eval;

pub fn binary_op(op: Token, left: Box<ASTNode>, right: Box<ASTNode>, env: Arc<Mutex<Env>>) -> Value {
    let left_val = eval(*left, env.clone());
    let right_val = eval(*right, env.clone());

    match (left_val.clone(), right_val.clone(), op.clone()) {
        (Value::String(l), Value::String(r), Token::Plus) => Value::String(l + &r),
        (Value::Number(l), Value::Number(r), Token::Plus) => Value::Number(l + r),
        (Value::Number(l), Value::Number(r), Token::Minus) => Value::Number(l - r),
        (Value::Number(l), Value::Number(r), Token::Mul) => Value::Number(l * r),
        (Value::Number(l), Value::Number(r), Token::Div) => Value::Number(l / r),
        _ => panic!("Unsupported operation: {:?} {:?} {:?}", left_val.clone(), op, right_val.clone()),
    }
}
