use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, EnvVariableType};
use crate::evals::eval;

pub fn for_node(variable: String, iterable: Box<ASTNode>, body: Box<ASTNode>, env: &mut Env) -> Value {
    let iterable = eval(*iterable, env);
    match iterable {
        Value::List(values) => {
            for value in values {
                let scope_name = format!("for-{}-{}", variable.clone(), value.clone());
                env.enter_scope(scope_name.clone());
                env.set(variable.clone(), value.clone(), EnvVariableType::Immutable, value.value_type(), true);
                eval(*body.clone(), env);
                env.leave_scope();
            }
            Value::Void
        }
        _ => panic!("Unexpected iterable: {:?}", iterable),
    }
}
