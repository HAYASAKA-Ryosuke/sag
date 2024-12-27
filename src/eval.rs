use crate::environment::{Env, EnvVariableType, FunctionInfo, ValueType};
use crate::parser::{ASTNode, Value};
use crate::tokenizer::Token;
use fraction::Fraction;

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
                _ => panic!("Unexpected prefix op: {:?}", op),
            }
        }
        ASTNode::Function {
            name,
            arguments,
            body,
            return_type,
        } => {
            let function_info = FunctionInfo {
                arguments,
                body: Some(*body),
                return_type,
                builtin: None,
            };
            env.register_function(name, function_info);
            Value::Function
        }
        ASTNode::Lambda { arguments, body } => Value::Lambda {
            arguments,
            body: body.clone(),
            env: env.clone(),
        },
        ASTNode::Block(statements) => {
            let mut value = Value::Number(Fraction::from(1));
            for statement in statements {
                value = eval(statement, env);
            }
            value
        }
        ASTNode::Return(value) => {
            let result = eval(*value, env);
            result
        }
        ASTNode::Eq { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l == r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::If {
            condition,
            then,
            else_,
            value_type: _,
        } => {
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
                _ => panic!("Unexpected value type"),
            }
        }
        ASTNode::Assign {
            name,
            value,
            variable_type,
            value_type: _,
            is_new,
        } => {
            let value = eval(*value, env);
            let value_type = match value {
                Value::Number(_) => ValueType::Number,
                Value::String(_) => ValueType::String,
                Value::Bool(_) => ValueType::Bool,
                Value::Function => ValueType::Function,
                Value::Lambda { .. } => ValueType::Lambda,
                Value::Void => ValueType::Void,
                Value::List(ref elements) => {
                    if elements.len() == 0 {
                        ValueType::List(Box::new(ValueType::Any))
                    } else {
                        let first_element = elements.first().unwrap();
                        let value_type = first_element.value_type();
                        for e in elements {
                            if e.value_type() != value_type {
                                panic!("List value type mismatch");
                            }
                        }
                        ValueType::List(Box::new(value_type))
                    }
                }
            };
            let result = env.set(
                name.to_string(),
                value.clone(),
                variable_type,
                value_type,
                is_new,
            );
            if result.is_err() {
                panic!("{}", result.unwrap_err());
            }
            value
        }
        ASTNode::LambdaCall { lambda, arguments } => {
            println!("LambdaCall: {:?}", arguments);
            let mut params_vec = vec![];
            let lambda = match *lambda {
                ASTNode::Lambda { arguments, body } => (arguments, body),
                _ => panic!("Unexpected value type"),
            };
            for arg in &lambda.0 {
                params_vec.push(match arg {
                    ASTNode::Variable { name, value_type } => (name, value_type),
                    _ => panic!("illigal param: {:?}", lambda.0),
                });
            }

            let mut args_vec = vec![];

            for arg in arguments {
                match arg {
                    ASTNode::FunctionCallArgs(arguments) => {
                        args_vec = arguments;
                    }
                    _ => {
                        args_vec.push(arg);
                    }
                }
            }
            if args_vec.len() != lambda.0.len() {
                panic!("does not match arguments length");
            }

            let mut local_env = env.clone();

            local_env.enter_scope("lambda".to_string());

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
        }
        ASTNode::FunctionCall { name, arguments } => {
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

                env.leave_scope();
                result
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
        ASTNode::Variable {
            name,
            value_type: _,
        } => {
            let value = env.get(name.to_string(), None);
            if value.is_none() {
                panic!("Variable not found: {:?}", name);
            }
            value.unwrap().value.clone()
        }
        ASTNode::BinaryOp { left, op, right } => {
            let left_val = eval(*left, env);
            let right_val = eval(*right, env);

            match (left_val, right_val, op) {
                (Value::String(l), Value::String(r), Token::Plus) => Value::String(l + &r),
                (Value::Number(l), Value::Number(r), Token::Plus) => Value::Number(l + r),
                (Value::Number(l), Value::Number(r), Token::Mul) => Value::Number(l * r),
                (Value::Number(l), Value::Number(r), Token::Div) => Value::Number(l / r),
                _ => panic!("Unsupported operation"),
            }
        }
        _ => panic!("Unsupported ast node: {:?}", ast),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::EnvVariableType;

    #[test]
    fn test_four_basic_arithmetic_operations() {
        let mut env = Env::new();
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::PrefixOp {
                op: Token::Minus,
                expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            }),
            op: Token::Plus,
            right: Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                op: Token::Mul,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            }),
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
    }
    #[test]
    fn test_assign() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
        assert_eq!(
            Value::Number(Fraction::from(5)),
            env.get("x".to_string(), None).unwrap().value
        );
        assert_eq!(
            EnvVariableType::Mutable,
            env.get("x".to_string(), None).unwrap().variable_type
        );
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: false,
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
        assert_eq!(
            Value::Number(Fraction::from(5)),
            env.get("x".to_string(), None).unwrap().value
        );
        assert_eq!(
            EnvVariableType::Immutable,
            env.get("x".to_string(), None).unwrap().variable_type
        );
    }
    #[test]
    fn test_assign_expression_value() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "y".to_string(),
            value: Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(10)))),
                op: Token::Plus,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(20)))),
            }),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        assert_eq!(Value::Number(Fraction::from(30)), eval(ast, &mut env));
        assert_eq!(
            env.get("y".to_string(), None).unwrap().value,
            Value::Number(Fraction::from(30))
        );
    }
    #[test]
    fn test_assign_overwrite_mutable_variable() {
        let mut env = Env::new();

        let ast1 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(50)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(ast1, &mut env);

        // 再代入
        let ast2 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(100)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: false,
        };

        // 環境に新しい値が登録されていること
        assert_eq!(eval(ast2, &mut env), Value::Number(Fraction::from(100)));
        assert_eq!(
            env.get("z".to_string(), None).unwrap().value,
            Value::Number(Fraction::from(100))
        );
    }
    #[test]
    #[should_panic(expected = "Cannot reassign to immutable variable")]
    fn test_assign_to_immutable_variable() {
        let mut env = Env::new();

        // Immutable 変数の初期値を設定
        let ast1 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(200)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(ast1, &mut env);

        // 再代入しようとしてエラー
        let ast2 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(300)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: false,
        };
        eval(ast2, &mut env);
    }
    #[test]
    fn test_register_function_and_function_call() {
        let mut env = Env::new();
        let ast = ASTNode::Function {
            name: "foo".into(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(
                ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "y".into(),
                        value_type: Some(ValueType::Number),
                    }),
                },
            ))])),
            return_type: ValueType::Number,
        };
        eval(ast, &mut env);
        let ast = ASTNode::FunctionCall {
            name: "foo".into(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(1))),
                ASTNode::Literal(Value::Number(Fraction::from(2))),
            ])),
        };
        let result = eval(ast, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(3)));
    }

    #[test]
    #[should_panic(expected = "Unexpected prefix op: Plus")]
    fn test_unsupported_prefix_operation() {
        let mut env = Env::new();
        let ast = ASTNode::PrefixOp {
            op: Token::Plus,
            expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
        };
        eval(ast, &mut env);
    }

    #[test]
    #[should_panic(expected = "Unsupported operation")]
    fn test_unsupported_binary_operation() {
        let mut env = Env::new();
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::Literal(Value::String("hello".to_string()))),
            op: Token::Mul,
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
        };
        eval(ast, &mut env);
    }

    #[test]
    fn test_list() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::List(Box::new(ValueType::Number)),
            is_new: true,
        };
        assert_eq!(
            Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]),
            eval(ast, &mut env)
        );
        assert_eq!(
            Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]),
            env.get("x".to_string(), None).unwrap().value
        );
    }

    #[test]
    #[should_panic(expected = "does not match arguments length")]
    fn test_function_call_argument_mismatch() {
        let mut env = Env::new();
        let ast_function = ASTNode::Function {
            name: "bar".to_string(),
            arguments: vec![ASTNode::Variable {
                name: "x".into(),
                value_type: Some(ValueType::Number),
            }],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::Variable {
                name: "x".into(),
                value_type: Some(ValueType::Number),
            }))),
            return_type: ValueType::Number,
        };
        eval(ast_function, &mut env);

        // 引数の数が合わない関数呼び出し
        let ast_call = ASTNode::FunctionCall {
            name: "bar".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(5))),
                ASTNode::Literal(Value::Number(Fraction::from(10))), // 余分な引数
            ])),
        };
        eval(ast_call, &mut env);
    }

    #[test]
    fn test_scope_management_in_function() {
        let mut env = Env::new();

        // 関数定義
        let ast_function = ASTNode::Function {
            name: "add_and_return".to_string(),
            arguments: vec![ASTNode::Variable {
                name: "a".into(),
                value_type: Some(ValueType::Number),
            }],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "local_var".into(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(10)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: true,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "a".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "local_var".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };

        eval(ast_function, &mut env);

        // 関数呼び出し
        let ast_call = ASTNode::FunctionCall {
            name: "add_and_return".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![ASTNode::Literal(
                Value::Number(Fraction::from(5)),
            )])),
        };

        // 結果の確認
        let result = eval(ast_call, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(15)));

        // スコープ外でローカル変数が見つからないことを確認
        let local_var_check = env.get("local_var".to_string(), None);
        assert!(local_var_check.is_none());
    }

    #[test]
    fn test_scope_and_global_variable() {
        let mut env = Env::new();

        // グローバル変数 z を定義
        let global_z = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(global_z, &mut env);

        // f1 関数の定義
        let f1 = ASTNode::Function {
            name: "f1".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Assign {
                    name: "d".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,

                    is_new: true,
                },
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Assign {
                        name: "d".to_string(),
                        value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4)))),
                        variable_type: EnvVariableType::Mutable,
                        value_type: ValueType::Number,
                        is_new: false,
                    }),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number),
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Variable {
                            name: "y".into(),
                            value_type: Some(ValueType::Number),
                        }),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "z".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };
        eval(f1, &mut env);

        // f2 関数の定義
        let f2 = ASTNode::Function {
            name: "f2".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "y".into(),
                        value_type: Some(ValueType::Number),
                    }),
                }),
                op: Token::Plus,
                right: Box::new(ASTNode::Variable {
                    name: "z".into(),
                    value_type: Some(ValueType::Number),
                }),
            }))),
            return_type: ValueType::Number,
        };
        eval(f2, &mut env);

        // f3 関数の定義
        let f3 = ASTNode::Function {
            name: "f3".to_string(),
            arguments: vec![],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(
                Fraction::from(1),
            ))))),
            return_type: ValueType::Number,
        };
        eval(f3, &mut env);

        // f1 の呼び出し
        let call_f1 = ASTNode::FunctionCall {
            name: "f1".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result_f1 = eval(call_f1, &mut env);
        assert_eq!(result_f1, Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6

        // f2 の呼び出し (f1 の影響で z = 4)
        let call_f2 = ASTNode::FunctionCall {
            name: "f2".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result_f2 = eval(call_f2, &mut env);
        assert_eq!(result_f2, Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6

        // f3 の呼び出し
        let call_f3 = ASTNode::FunctionCall {
            name: "f3".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![])),
        };
        let result_f3 = eval(call_f3, &mut env);
        assert_eq!(result_f3, Value::Number(Fraction::from(1)));
    }

    #[test]
    fn test_global_variable_and_functions() {
        let mut env = Env::new();

        // グローバル変数の定義
        let global_z = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(global_z, &mut env);

        // f1関数の定義
        let f1 = ASTNode::Function {
            name: "f1".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Assign {
                    name: "d".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: true,
                },
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Assign {
                        name: "d".to_string(),
                        value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4)))),
                        variable_type: EnvVariableType::Mutable,
                        value_type: ValueType::Number,
                        is_new: false,
                    }),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number),
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Variable {
                            name: "y".into(),
                            value_type: Some(ValueType::Number),
                        }),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "z".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };
        eval(f1, &mut env);

        // f1の呼び出し
        let call_f1 = ASTNode::FunctionCall {
            name: "f1".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result = eval(call_f1, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(6))); // 2 + 0 + 4 = 6
    }

    #[test]
    fn test_if() {
        let mut env = Env::new();
        let ast = ASTNode::If {
            condition: Box::new(ASTNode::Eq {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
            }),
            then: Box::new(ASTNode::Block(vec![
                ASTNode::Literal(Value::Number(Fraction::from(1)))
            ])),
            else_: None,
            value_type: ValueType::Void
        };
        assert_eq!(Value::Number(Fraction::from(1)), eval(ast, &mut env));
    }
}
