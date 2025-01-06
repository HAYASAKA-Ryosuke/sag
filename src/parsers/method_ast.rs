use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;

impl Parser {

    pub fn parse_method(&mut self) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        self.enter_scope(name.to_string());
        self.consume_token();
        self.extract_token(Token::LParen);
        let arguments = self.parse_function_arguments();
        let return_type = self.parse_return_type();
        let body = self.parse_block();
        self.leave_scope();
        ASTNode::Method {
            name,
            arguments,
            body: Box::new(body),
            return_type,
        }
    }

    pub fn parse_method_call(&mut self, caller: String, method_name: String, arguments: ASTNode) -> ASTNode {
        self.consume_token();
        ASTNode::MethodCall {
            method_name,
            caller,
            arguments: Box::new(arguments),
        }
    }
}
