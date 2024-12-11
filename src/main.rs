mod tokenizer;
mod parser;
use crate::tokenizer::{{ tokenize, Token }};
use crate::parser::{{ ASTNode, Parser, Value }};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_basic_arithmetic_operations() {
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
        assert_eq!(Value::Number(5.0), eval(ast));
    }
}


fn eval(ast: ASTNode) -> Value {
    match ast {
        ASTNode::Literal(value) => value.clone(),
        ASTNode::PrefixOp { op, expr } => {
            let value = eval(*expr);
            match (op.clone(), value) {
                (Token::Minus, Value::Number(v)) => Value::Number(-v),
                _ => panic!("Unexpected prefix op: {:?}", op)
            }
        },
        ASTNode::BinaryOp { left, op, right } => {
            let left_val = eval(*left);
            let right_val = eval(*right);

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

fn run() -> Result<(), Box<dyn std::error::Error>> {
    for line in std::io::stdin().lines() {
        let tokens = tokenize(&line?);
        println!("{:?}", tokens);
        let mut parser = Parser::new(tokens.to_vec());
        let ast_node = parser.parse();
        println!("{:?}", ast_node);
        let result = eval(ast_node);
        println!("{:?}", result);
    }
    Ok(())
}

fn main() {
    run();
}


