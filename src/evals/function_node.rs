use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, FunctionInfo, EnvVariableType};
use crate::evals::eval;

pub fn function_node(name: String, arguments: Vec<ASTNode>, body: Box<ASTNode>, return_type: ValueType, env: Rc<RefCell<Env>>) -> Value {
    let function_info = FunctionInfo {
        arguments,
        body: Some(*body),
        return_type,
        builtin: None,
    };
    env.borrow_mut().register_function(name, function_info);
    Value::Function
}

pub fn block_node(statements: Vec<ASTNode>, env: Rc<RefCell<Env>>) -> Value {
    for statement in statements {
        if let Value::Return(v) = eval(statement, env.clone()) {
            return *v;
        }
    }
    Value::Void
}

pub fn function_call_node(name: String, arguments: Box<ASTNode>, env: Rc<RefCell<Env>>) -> Value {
    if env.borrow().get_function(name.to_string()).is_some()
        || env.borrow().get_builtin(name.to_string()).is_some()
    {
        let function = {
            let env_borrow = env.borrow();
            if let Some(function) = env_borrow.get_function(name.to_string()) {
                function.clone()
            } else {
                env_borrow
                    .get_builtin(name.to_string())
                    .expect("Function is missing")
                    .clone()
            }
        };

        let params_vec: Vec<_> = function
            .arguments
            .iter()
            .map(|arg| match arg {
                ASTNode::Variable { name, value_type } => (name.clone(), value_type.clone()),
                _ => panic!("Illegal param: {:?}", function.arguments),
            })
            .collect();

        let args_vec = match *arguments {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => panic!("Illegal arguments: {:?}", arguments),
        };

        if let Some(func) = function.builtin {
            return func(
                args_vec
                    .iter()
                    .map(|arg| eval(arg.clone(), env.clone()))
                    .collect(),
            );
        }

        if args_vec.len() != function.arguments.len() {
            panic!("Arguments length mismatch");
        }

        let mut env_mut = env.borrow_mut().new_child();
        env_mut.enter_scope(name.to_string());

        for (param, arg) in params_vec.iter().zip(&args_vec) {
            let arg_value = eval(arg.clone(), env.clone());
            let name = param.0.to_string();
            let value_type = param.1.clone();
            {
                let aa = env_mut.set(
                    name,
                    arg_value,
                    EnvVariableType::Immutable,
                    value_type.unwrap_or(ValueType::Any),
                    true,
                );
            }
        }

        let result = {
            let body = function.body.expect("Function body is missing");
            eval(body, Rc::new(RefCell::new(env_mut.clone())))
        };

        env_mut.leave_scope();

        if let Value::Return(v) = result {
            *v
        } else {
            result
        }
    } else if let Some(lambda_value) = env
        .borrow()
        .get(name.to_string(), Some(&ValueType::Lambda))
        .map(|v| v.value.clone())
    {
        let lambda = match lambda_value {
            Value::Lambda {
                arguments,
                body,
                env: lambda_env,
            } => (arguments, body, lambda_env),
            _ => panic!("Unexpected value type"),
        };

        let params_vec: Vec<_> = lambda
            .0
            .iter()
            .map(|arg| match arg {
                ASTNode::Variable { name, value_type } => (name.clone(), value_type.clone()),
                _ => panic!("Illegal param: {:?}", lambda.0),
            })
            .collect();

        let args_vec = match *arguments {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => panic!("Illegal arguments: {:?}", arguments),
        };

        if args_vec.len() != lambda.0.len() {
            panic!("Arguments length mismatch");
        }

        {
            let mut env_mut = env.borrow_mut();
            env_mut.enter_scope(name.to_string());
        }

        for (param, arg) in params_vec.iter().zip(&args_vec) {
            let arg_value = eval(arg.clone(), env.clone());
            let name = param.0.to_string();
            let value_type = param.1.clone();
            {
                let mut env_mut = env.borrow_mut();
                env_mut.set(
                    name,
                    arg_value,
                    EnvVariableType::Immutable,
                    value_type.unwrap_or(ValueType::Any),
                    true,
                );
            }
        }

        let result = {
            let body = *lambda.1;
            eval(body, env.clone())
        };

        {
            let mut env_mut = env.borrow_mut();
            env_mut.leave_scope();
        }

        result
    } else {
        panic!("Function is missing: {:?}", name)
    }
}
