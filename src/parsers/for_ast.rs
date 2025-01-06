use crate::ast::ASTNode;
use crate::token::Token;
use crate::parsers::Parser;
use crate::environment::{EnvVariableType, ValueType};

impl Parser {
    pub fn parse_for(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::For) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let variable = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        self.consume_token();
        self.extract_token(Token::In);
        let iterable = self.parse_expression(0);
        let body = self.parse_expression(0);
        ASTNode::For {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
        }
    }
}
