use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::{EnvVariableType, ValueType};

impl Parser {
    pub fn parse_function(&mut self) -> ASTNode {
        self.pos += 1;
        let name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("undefined function name"),
        };
        let function_scope = self.get_current_scope();
        self.enter_scope(name.to_string());
        self.pos += 1;
        self.extract_token(TokenKind::LParen);

        let arguments = self.parse_function_arguments();
        let return_type = self.parse_return_type();
        self.register_functions(
            function_scope,
            &name,
            &arguments,
            &return_type,
        );
        let body = self.parse_block();

        self.leave_scope();

        let mut include_return = false;
        match body.clone() {
            ASTNode::Block(statements) => {
                for statement in statements {
                    if let ASTNode::Return(value) = statement {
                        include_return = true;
                        if let Ok(return_value_type) = self.infer_type(&value.clone()) {
                            if return_value_type != return_type {
                                panic!("Return type mismatch Expected type: {:?}, Actual type: {:?}", return_type, return_value_type);
                            }
                        }
                    }
                }
            }
            _ => (),
        };

        if !include_return && return_type != ValueType::Void {
            panic!("Missing return statement");
        }

        ASTNode::Function {
            name,
            arguments,
            body: Box::new(body),
            return_type,
        }
    }

    pub fn parse_function_call_arguments_paren(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token{kind: TokenKind::LParen, ..}) => self.consume_token(),
            _ => None,
        };
        let mut arguments = vec![];
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::Comma {
                self.pos += 1;
                continue;
            }
            if token.kind == TokenKind::RParen {
                self.pos += 1;
                break;
            }
            if token.kind == TokenKind::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            let value = self.parse_expression(0);
            arguments.push(value);
        }
        ASTNode::FunctionCallArgs(arguments)
    }

    pub fn parse_function_arguments(&mut self) -> Vec<ASTNode> {
        let scope = self.get_current_scope();
        let mut arguments = Vec::new();
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::RParen {
                break;
            }
            if let TokenKind::Identifier(name) = self.consume_token().unwrap().kind {
                let mut variable_name = name.clone();
                let current_token = self.get_current_token();
                let arg_type = if current_token.is_none() {
                    self.extract_token(TokenKind::Colon);
                    match self.consume_token() {
                        Some(Token{kind: TokenKind::Identifier(type_name), ..}) => self.string_to_value_type(type_name),
                        _ => panic!("Expected type for argument"),
                    }
                } else {
                    let current_token_kind = current_token.unwrap().kind.clone();
                    if name == "self" && ( current_token_kind == TokenKind::Comma || current_token_kind == TokenKind::RParen) {
                        ValueType::SelfType
                    } else if name == "mut" && current_token_kind == TokenKind::Identifier("self".to_string()) {
                        self.consume_token();
                        let current_token_kind = self.get_current_token().unwrap().kind.clone();
                        if current_token_kind == TokenKind::Comma || current_token_kind == TokenKind::RParen {
                            variable_name = "self".to_string();
                            ValueType::MutSelfType
                        } else {
                            panic!("Expected self after mut")
                        }
                    } else {
                        self.extract_token(TokenKind::Colon);
                        match self.consume_token() {
                            Some(Token{kind: TokenKind::Identifier(type_name), ..}) => self.string_to_value_type(type_name),
                            _ => panic!("Expected type for argument"),
                        }
                    }
                };
                self.register_variables(
                    scope.to_string(),
                    &variable_name,
                    &arg_type,
                    &EnvVariableType::Immutable,
                );
                arguments.push(ASTNode::Variable {
                    name: variable_name,
                    value_type: Some(arg_type),
                });
            }
            match self.get_current_token() {
                Some(Token{kind: TokenKind::Comma, ..}) => {
                    self.consume_token();
                },
                _ => {},
            };
        }
        self.extract_token(TokenKind::RParen);
        arguments
    }

    pub fn parse_function_call_front(&mut self, name: String, arguments: ASTNode) -> ASTNode {

        let function_call = ASTNode::FunctionCall {
            name,
            arguments: Box::new(arguments),
        };
        function_call
    }

    pub fn parse_function_call(&mut self, left: ASTNode) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("failed take function name: {:?}", self.get_current_token()),
        };

        let arguments = ASTNode::FunctionCallArgs(match left {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => vec![left],
        });

        self.consume_token();

        let function_call = ASTNode::FunctionCall {
            name,
            arguments: Box::new(arguments),
        };
        function_call
    }
}
