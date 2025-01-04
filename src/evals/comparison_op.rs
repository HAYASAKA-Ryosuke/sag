use crate::ast::ASTNode;
use crate::value::Value;
use crate::token::Token;
use crate::environment::Env;
use crate::evals::eval;

pub fn comparison_op_node(op: Token, left: Box<ASTNode>, right: Box<ASTNode>, env: &mut Env) -> Value {
    let left_value = eval(*left, env);
    let right_value = eval(*right, env);
    match (left_value, right_value, op) {
        (Value::Number(l), Value::Number(r), Token::Eq) => Value::Bool(l == r),
        (Value::Number(l), Value::Number(r), Token::Gte) => Value::Bool(l >= r),
        (Value::Number(l), Value::Number(r), Token::Gt) => Value::Bool(l > r),
        (Value::Number(l), Value::Number(r), Token::Lte) => Value::Bool(l <= r),
        (Value::Number(l), Value::Number(r), Token::Lt) => Value::Bool(l < r),
        _ => panic!("Unsupported operation"),
    }
}
