use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;
use crate::value::Value;

impl Parser {
    pub fn parse_list(&mut self) -> ASTNode {
        self.consume_token();
        let mut list = vec![];
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrancket {
                self.consume_token();
                break;
            }
            if token == Token::Comma {
                self.consume_token();
                continue;
            }
            let value = match token {
                Token::Number(value) => Value::Number(value),
                Token::String(value) => Value::String(value),
                _ => panic!("unexpected token: {:?}", token),
            };
            list.push(ASTNode::Literal(value));
            self.consume_token();
        }
        ASTNode::Literal(Value::List(
            list.iter()
                .map(|x| match x {
                    ASTNode::Literal(value) => value.clone(),
                    _ => panic!("unexpected node"),
                })
                .collect(),
        ))
    }
}
