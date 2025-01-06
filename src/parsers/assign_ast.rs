use crate::ast::ASTNode;
use crate::token::Token;
use crate::parsers::Parser;
use crate::environment::{EnvVariableType, ValueType};

impl Parser {

    pub fn parse_assign(&mut self) -> ASTNode {
        let scope = self.get_current_scope();
        let mutable_or_immutable = self.consume_token().unwrap();
        let name = match self.consume_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        match self.consume_token() {
            Some(Token::Equal) => {
                let value = self.parse_expression(0);
                let value_type = match self.infer_type(&value) {
                    Ok(value_type) => value_type,
                    Err(e) => panic!("{}", e),
                };
                let variable_type = if mutable_or_immutable == Token::Mutable {
                    EnvVariableType::Mutable
                } else {
                    EnvVariableType::Immutable
                };

                self.register_variables(scope.clone(), &name, &value_type, &variable_type);
                ASTNode::Assign {
                    name,
                    value: Box::new(value),
                    variable_type,
                    value_type,
                    is_new: true,
                }
            }
            Some(Token::Colon) => {
                let value_type = match self.consume_token() {
                    Some(token) => match token {
                        Token::Identifier(value_type) => match value_type.as_str() {
                            "number" => ValueType::Number,
                            "str" => ValueType::String,
                            "bool" => ValueType::Bool,
                            "void" => ValueType::Void,
                            _ => {
                                self.string_to_value_type(value_type)
                            }
                        },
                        _ => panic!("undefined type"),
                    },
                    _ => panic!("undefined type"),
                };
                match self.consume_token() {
                    Some(Token::Equal) => {
                        let value = self.parse_expression(0);
                        let variable_type = if mutable_or_immutable == Token::Mutable {
                            EnvVariableType::Mutable
                        } else {
                            EnvVariableType::Immutable
                        };
                        self.register_variables(scope, &name, &value_type, &variable_type);
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type,
                            value_type,
                            is_new: true,
                        }
                    }
                    _ => panic!("No valid statement found on the right-hand side"),
                }
            }
            _ => panic!("unexpected token"),
        }
    }
}
