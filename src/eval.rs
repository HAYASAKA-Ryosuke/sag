use crate::parser::{ASTNode, Value};
use crate::environment::Env;
use crate::tokenizer::Token;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::VariableType;

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
            variable_type: VariableType::Mutable
        };
        assert_eq!(Value::Number(5.0), eval(ast, &mut env));
        assert_eq!(Value::Number(5.0), env.get("x".to_string()).unwrap().value);
        assert_eq!(VariableType::Mutable, env.get("x".to_string()).unwrap().variable_type);
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(5.0))),
            variable_type: VariableType::Immutable
        };
        assert_eq!(Value::Number(5.0), eval(ast, &mut env));
        assert_eq!(Value::Number(5.0), env.get("x".to_string()).unwrap().value);
        assert_eq!(VariableType::Immutable, env.get("x".to_string()).unwrap().variable_type);
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
            variable_type: VariableType::Mutable,
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
            variable_type: VariableType::Mutable,
        };
        eval(ast1, &mut env);

        // 再代入
        let ast2 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(100.0))),
            variable_type: VariableType::Mutable,
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
            variable_type: VariableType::Immutable,
        };
        eval(ast1, &mut env);

        // 再代入しようとしてエラー
        let ast2 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(300.0))),
            variable_type: VariableType::Immutable,
        };
        eval(ast2, &mut env);
    }
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
        ASTNode::Assign { name, value, variable_type } => {
            let value = eval(*value, env);
            let result = env.set(name.to_string(), value.clone(), variable_type);
            if result.is_err() {
                panic!("{}", result.unwrap_err());
            }
            value
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
        }
    }
}
