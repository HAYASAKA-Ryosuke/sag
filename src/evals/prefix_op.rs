use crate::ast::ASTNode;
use crate::value::Value;
use crate::token::TokenKind;
use crate::environment::Env;
use crate::evals::eval;

pub fn prefix_op(op: TokenKind, expr: Box<ASTNode>, env: &mut Env) -> Value {
    let value = eval(*expr, env);
    match (op.clone(), value) {
        (TokenKind::Minus, Value::Number(v)) => Value::Number(-v),
        _ => panic!("Unexpected prefix op: {:?}", op),
    }
}
