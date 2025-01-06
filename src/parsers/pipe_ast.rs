use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;

impl Parser {
    pub fn parse_function_call_arguments(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Pipe) => self.consume_token(),
            _ => None,
        };
        let mut arguments = vec![];
        while let Some(token) = self.get_current_token() {
            if token == Token::Comma {
                self.pos += 1;
                continue;
            }
            if token == Token::Pipe {
                self.pos += 1;
                break;
            }
            let value = self.parse_expression(0);
            arguments.push(value);
        }
        ASTNode::FunctionCallArgs(arguments)
    }
}
