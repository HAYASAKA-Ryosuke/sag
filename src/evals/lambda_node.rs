use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, EnvVariableType, ValueType};
use crate::evals::eval;

pub fn lambda_call_node(lambda: Box<ASTNode>, arguments: Vec<ASTNode>, env: &mut Env) -> Value {
    let mut params_vec = vec![];
    let lambda = match *lambda {
        ASTNode::Lambda { arguments, body } => (arguments, body),
        _ => panic!("Unexpected value type: {:?}", lambda),
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

#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;
    use crate::token::Token;

    #[test]
    fn test_lambda_expression() {
        let mut env = Env::new();
        let lambda_ast = ASTNode::Lambda {
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
            body: Box::new(ASTNode::BinaryOp {
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
        };
        let lambda = eval(lambda_ast.clone(), &mut env);
        assert_eq!(
            lambda,
            Value::Lambda {
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
                body: Box::new(ASTNode::BinaryOp {
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
                env: env.clone(),
            }
        );

        // ラムダの呼び出し
        let call_ast = ASTNode::LambdaCall {
            lambda: Box::new(lambda_ast),
            arguments: vec![
                ASTNode::Literal(Value::Number(Fraction::from(3))),
                ASTNode::Literal(Value::Number(Fraction::from(4))),
            ],
        };
        let result = eval(call_ast, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(7)));
    }
}
