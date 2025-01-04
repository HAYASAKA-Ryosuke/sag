use std::sync::{Arc, Mutex};
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::Env;
use crate::evals::eval;

pub fn if_node(condition: Box<ASTNode>, then: Box<ASTNode>, else_: Option<Box<ASTNode>>, env: Arc<Mutex<Env>>) -> Value {
    let condition = eval(*condition, env.clone());
    match condition {
        Value::Bool(true) => eval(*then, env.clone()),
        Value::Bool(false) => {
            if let Some(else_) = else_{
                eval(*else_, env.clone())
            } else {
                Value::Void
            }
        }
        _ => panic!("Unexpected condition: {:?}", condition),
    }
}
