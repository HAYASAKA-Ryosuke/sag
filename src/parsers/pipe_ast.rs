use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::{Token, TokenKind};

impl Parser {
    pub fn parse_function_call_arguments(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token{kind: TokenKind::Pipe, ..}) => self.consume_token(),
            _ => None,
        };
        let mut arguments = vec![];
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::Comma {
                self.pos += 1;
                continue;
            }
            if token.kind == TokenKind::Pipe {
                self.pos += 1;
                break;
            }
            let value = self.parse_expression(0);
            arguments.push(value);
        }
        ASTNode::FunctionCallArgs(arguments)
    }
}
