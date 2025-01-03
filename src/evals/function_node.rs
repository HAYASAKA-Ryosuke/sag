use crate::parser::{ASTNode, Value};
use crate::environment::{Env, ValueType, FunctionInfo};
use crate::evals::eval;


pub fn function_node(name: String, arguments: Vec<ASTNode>, body: Box<ASTNode>, return_type: ValueType, env: &mut Env) -> Value {
    let function_info = FunctionInfo {
        arguments,
        body: Some(*body),
        return_type,
        builtin: None,
    };
    env.register_function(name, function_info);
    Value::Function
}

pub fn block_node(statements: Vec<ASTNode>, env: &mut Env) -> Value {
    for statement in statements {
        if let Value::Return(v) = eval(statement, env) {
            return *v;
        }
    }
    Value::Void
}
