use crate::ast::ASTNode;
use crate::value::Value;
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

#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;
    use crate::environment::ValueType;

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
        assert_eq!(Value::Void, eval(ast, &mut env));
    }

    #[test]
    fn test_if_return() {
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
        assert_eq!(Value::Void, eval(ast, &mut env));
    }
}
