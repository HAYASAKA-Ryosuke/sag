use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, FunctionInfo, EnvVariableType};
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

pub fn function_call_node(name: String, arguments: Box<ASTNode>, env: &mut Env) -> Value {
    if env.get_function(name.to_string()).is_some()
        || env.get_builtin(name.to_string()).is_some()
    {
        let function = match env.get_function(name.to_string()) {
            Some(function) => function.clone(),
            None => {
                let builtin = env.get_builtin(name.to_string());
                if builtin.is_some() {
                    builtin.unwrap().clone()
                } else {
                    panic!("Function is missing: {:?}", name)
                }
            }
        };
        let mut params_vec = vec![];
        for arg in &function.arguments {
            params_vec.push(match arg {
                ASTNode::Variable { name, value_type } => (name, value_type),
                _ => panic!("illigal param: {:?}", function.arguments),
            });
        }

        let args_vec = match *arguments {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => panic!("illigal arguments: {:?}", arguments),
        };

        if let Some(func) = function.builtin {
            let result = func(args_vec.iter().map(|arg| eval(arg.clone(), env)).collect());
            return result;
        };

        if args_vec.len() != function.arguments.len() {
            panic!("does not match arguments length");
        }

        let mut local_env = env.clone();

        local_env.enter_scope(name.to_string());

        for (param, arg) in params_vec.iter().zip(&args_vec) {
            let arg_value = eval(arg.clone(), env);
            let name = param.0.to_string();
            let value_type = param.1.clone();
            let _ = local_env.set(
                name,
                arg_value,
                EnvVariableType::Immutable,
                value_type.unwrap_or(ValueType::Any),
                true,
            );
        }


        let result = eval(function.body.unwrap(), &mut local_env);
        env.update_global_env(&local_env);

        local_env.leave_scope();
        if let Value::Return(v) = result {
            *v
        } else {
            result
        }
    } else if env
        .get(name.to_string(), Some(&ValueType::Lambda))
        .is_some()
    {
        let lambda = match env.get(name.to_string(), None).unwrap().value.clone() {
            Value::Lambda {
                arguments,
                body,
                env: lambda_env,
            } => (arguments, body, lambda_env),
            _ => panic!("Unexpected value type"),
        };

        let mut params_vec = vec![];
        for arg in &lambda.0 {
            params_vec.push(match arg {
                ASTNode::Variable { name, value_type } => (name, value_type),
                _ => panic!("illigal param: {:?}", lambda.0),
            });
        }

        let args_vec = match *arguments {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => panic!("illigal arguments: {:?}", arguments),
        };

        if args_vec.len() != lambda.0.len() {
            panic!("does not match arguments length");
        }

        let mut local_env = env.clone();

        local_env.enter_scope(name.to_string());

        for (param, arg) in params_vec.iter().zip(&args_vec) {
            let arg_value = eval(arg.clone(), env);
            let name = param.0.to_string();
            let value_type = param.1.clone();
            let _ = local_env.set(
                name,
                arg_value,
                EnvVariableType::Immutable,
                value_type.unwrap_or(ValueType::Any),
                true,
            );
        }

        let result = eval(*lambda.1, &mut local_env);

        env.update_global_env(&local_env);

        env.leave_scope();
        result
    } else {
        panic!("Function is missing: {:?}", name)
    }
}
