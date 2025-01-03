use crate::parser::{ASTNode, Value};
use crate::environment::Env;
use crate::evals::eval;

pub fn if_node(condition: Box<ASTNode>, then: Box<ASTNode>, else_: Option<Box<ASTNode>>, env: &mut Env) -> Value {
    let condition = eval(*condition, env);
    match condition {
        Value::Bool(true) => eval(*then, env),
        Value::Bool(false) => {
            if let Some(else_) = else_{
                eval(*else_, env)
            } else {
                Value::Void
            }
        }
        _ => panic!("Unexpected condition: {:?}", condition),
    }
}
