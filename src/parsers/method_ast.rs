use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;
use crate::environment::ValueType;

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
        let mut is_mut = false;
        if arguments.len() > 0 {
            match arguments.first() {
                Some(ASTNode::Variable { name, value_type }) => {
                    if name != "self" {
                        panic!("first argument must be self");
                    }
                    match value_type {
                        Some(value_type) => {
                            is_mut = *value_type != ValueType::SelfType;
                        },
                        _ => {},
                    }
                },
                _ => panic!("first argument must be self"),
            }
        }
        let return_type = self.parse_return_type();
        let body = self.parse_block();
        self.leave_scope();
        let method = ASTNode::Method {
            name: name.clone(),
            arguments,
            body: Box::new(body),
            return_type,
            is_mut,
        };
        self.register_method(self.get_current_scope(), self.current_struct.clone().unwrap(), method.clone());
        method
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
