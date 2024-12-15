use crate::parser::{ASTNode, Value};
use crate::environment::{ValueType, Env, FunctionInfo, EnvVariableType};
use crate::tokenizer::Token;


pub fn evals(asts: Vec<ASTNode>, env: &mut Env) -> Vec<Value> {
    let mut values = vec![];
    for ast in asts {
        values.push(eval(ast, env));
    }
    values
}

pub fn eval(ast: ASTNode, env: &mut Env) -> Value {
    match ast {
        ASTNode::Literal(value) => value.clone(),
        ASTNode::PrefixOp { op, expr } => {
            let value = eval(*expr, env);
            match (op.clone(), value) {
                (Token::Minus, Value::Number(v)) => Value::Number(-v),
                _ => panic!("Unexpected prefix op: {:?}", op)
            }
        },
        ASTNode::Function { name, arguments, body, return_type } => {
            let function_info = FunctionInfo {
                arguments,
                body: *body,
                return_type,
            };
            env.register_function(name, function_info);
            Value::Function
        },
        ASTNode::Block(statements) => {
            let mut value = Value::Number(1.0);
            for statement in statements {
                value = eval(statement, env);
            }
            value
        },
        ASTNode::Return(value) => {
            let result = eval(*value, env);
            println!("res: {:?}", result);
            result
        },
        ASTNode::Assign { name, value, variable_type, value_type: _ } => {
            let value = eval(*value, env);
            let value_type = match value {
                Value::Number(_) => ValueType::Number,
                Value::Str(_) => ValueType::Str,
                Value::Bool(_) => ValueType::Bool,
                Value::Function => ValueType::Function,
            };
            let result = env.set(name.to_string(), value.clone(), variable_type, value_type);
            if result.is_err() {
                panic!("{}", result.unwrap_err());
            }
            value
        },
        ASTNode::FunctionCall { name, arguments } => {
            println!("name: {:?}, arguments: {:?}", name, arguments);
            let function = match env.get_function(name.to_string()) {
                Some(function) => function.clone(),
                None => panic!("Function is missing: {:?}", name)
            };
            let mut params_vec = vec![];
            for arg in &function.arguments {
                params_vec.push(
                    match arg {
                        ASTNode::Variable { name, value_type } => (name, value_type),
                        _ => panic!("illigal param: {:?}", function.arguments)
                });
            }

            let args_vec = match *arguments {
                ASTNode::FunctionCallArgs(arguments) => arguments,
                _ => panic!("illigal arguments: {:?}", arguments)
            };

            if args_vec.len() != function.arguments.len() {
                panic!("does not match arguments length");
            }

            let mut local_env = env.clone();

            local_env.enter_scope(name.to_string());


            for (param, arg) in params_vec.iter().zip(args_vec) {
                let arg_value = eval(arg, env);
                let name = param.0.to_string();
                let value_type = param.1.clone();
                local_env.set(name, arg_value, EnvVariableType::Immutable, value_type.unwrap_or(ValueType::Any));
            }

            println!("body: {:?}", function.body);
            let result = eval(function.body, &mut local_env);

            env.leave_scope();
            result
        },
        ASTNode::Variable{ name, value_type: _ } => {
            let value = env.get(name.to_string());
            if value.is_none() {
                panic!("Variable not found: {:?}", name);
            }
            value.unwrap().value.clone()
        },
        ASTNode::BinaryOp { left, op, right } => {
            let left_val = eval(*left, env);
            let right_val = eval(*right, env);

            match (left_val, right_val, op) {
                (Value::Str(l), Value::Str(r), Token::Plus) => Value::Str(l + &r),
                (Value::Number(l), Value::Number(r), Token::Plus) => Value::Number(l + r),
                (Value::Number(l), Value::Number(r), Token::Mul) => Value::Number(l * r),
                (Value::Number(l), Value::Number(r), Token::Div) => Value::Number(l / r),
                _ => panic!("Unsupported operation"),
            }
        },
        _ => panic!("Unsupported ast node: {:?}", ast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::EnvVariableType;

    #[test]
    fn test_four_basic_arithmetic_operations() {
        let mut env = Env::new();
        let ast = 
            ASTNode::BinaryOp {
                left: Box::new(ASTNode::PrefixOp{
                    op: Token::Minus,
                    expr: Box::new(ASTNode::Literal(Value::Number(1.0)))
                }),
                op: Token::Plus,
                right: Box::new(ASTNode::BinaryOp{
                    left: Box::new(ASTNode::Literal(Value::Number(2.0))),
                    op: Token::Mul,
                    right: Box::new(ASTNode::Literal(Value::Number(3.0)))
                })
            };
        assert_eq!(Value::Number(5.0), eval(ast, &mut env));
    }
    #[test]
    fn test_assign() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(5.0))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        };
        assert_eq!(Value::Number(5.0), eval(ast, &mut env));
        assert_eq!(Value::Number(5.0), env.get("x".to_string()).unwrap().value);
        assert_eq!(EnvVariableType::Mutable, env.get("x".to_string()).unwrap().variable_type);
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(5.0))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number
        };
        assert_eq!(Value::Number(5.0), eval(ast, &mut env));
        assert_eq!(Value::Number(5.0), env.get("x".to_string()).unwrap().value);
        assert_eq!(EnvVariableType::Immutable, env.get("x".to_string()).unwrap().variable_type);
    }
    #[test]
    fn test_assign_expression_value() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "y".to_string(),
            value: Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(10.0))),
                op: Token::Plus,
                right: Box::new(ASTNode::Literal(Value::Number(20.0))),
            }),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        };
        assert_eq!(Value::Number(30.0), eval(ast, &mut env));
        assert_eq!(env.get("y".to_string()).unwrap().value, Value::Number(30.0));
    }
    #[test]
    fn test_assign_overwrite_mutable_variable() {
        let mut env = Env::new();

        let ast1 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(50.0))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        };
        eval(ast1, &mut env);

        // 再代入
        let ast2 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(100.0))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        };

        // 環境に新しい値が登録されていること
        assert_eq!(eval(ast2, &mut env), Value::Number(100.0));
        assert_eq!(env.get("z".to_string()).unwrap().value, Value::Number(100.0));
    }
    #[test]
    #[should_panic(expected = "Cannot reassign to immutable variable")]
    fn test_assign_to_immutable_variable() {
        let mut env = Env::new();

        // Immutable 変数の初期値を設定
        let ast1 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(200.0))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number
        };
        eval(ast1, &mut env);

        // 再代入しようとしてエラー
        let ast2 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(300.0))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number
        };
        eval(ast2, &mut env);
    }
    #[test]
    fn test_register_function_and_function_call() {
        let mut env = Env::new();
        let ast = ASTNode::Function {
            name: "foo".into(),
            arguments: vec![ASTNode::Variable{name: "x".into(), value_type: Some(ValueType::Number)}, ASTNode::Variable{name: "y".into(), value_type: Some(ValueType::Number)}],
            body: Box::new(
                ASTNode::Block(vec![
                    ASTNode::Return(Box::new(
                        ASTNode::BinaryOp{
                            left: Box::new(ASTNode::Variable{name: "x".into(), value_type: Some(ValueType::Number)}),
                            op: Token::Plus,
                            right: Box::new(ASTNode::Variable{name: "y".into(), value_type: Some(ValueType::Number)}),
                        }
                    ))
                ])
            ),
            return_type: ValueType::Number
        };
        eval(ast, &mut env);
        let ast = ASTNode::FunctionCall {
            name: "foo".into(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                    ASTNode::Literal(Value::Number(1.0)),
                    ASTNode::Literal(Value::Number(2.0))
            ])),
        };
        let result = eval(ast, &mut env);
        assert_eq!(result, Value::Number(3.0));
    }
}
