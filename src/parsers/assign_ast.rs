use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::{EnvVariableType, ValueType};
use crate::parsers::parse_error::ParseError;

impl Parser {

    pub fn parse_assign(&mut self) -> Result<ASTNode, ParseError> {
        let scope = self.get_current_scope();
        let mutable_or_immutable = self.consume_token().unwrap();
        let name = match self.consume_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => {
                let current_token = self.get_current_token().unwrap();
                return Err(ParseError::new("unexpected token missing variable name", &current_token))
            }
        };
        match self.consume_token() {
            Some(Token{kind: TokenKind::Equal, ..}) => {
                let value = self.parse_expression(0)?;
                let value_type = match self.infer_type(&value) {
                    Ok(value_type) => value_type,
                    Err(e) => panic!("{}", e),
                };
                let variable_type = if mutable_or_immutable.kind == TokenKind::Mutable {
                    EnvVariableType::Mutable
                } else {
                    EnvVariableType::Immutable
                };

                self.register_variables(scope.clone(), &name, &value_type, &variable_type);
                Ok(ASTNode::Assign {
                    name,
                    value: Box::new(value),
                    variable_type,
                    value_type,
                    is_new: true,
                    line: mutable_or_immutable.line,
                    column: mutable_or_immutable.column,
                })
            }
            Some(Token{kind: TokenKind::Colon, ..}) => {
                let value_type = match self.consume_token() {
                    Some(token) => match token.kind {
                        TokenKind::Identifier(value_type) => match value_type.as_str() {
                            "number" => ValueType::Number,
                            "str" => ValueType::String,
                            "bool" => ValueType::Bool,
                            "void" => ValueType::Void,
                            _ => {
                                self.string_to_value_type(value_type)
                            }
                        },
                        TokenKind::Option => {
                            self.extract_token(TokenKind::Lt);
                            let value_type = match self.consume_token() {
                                Some(token) => match token.kind {
                                    TokenKind::Identifier(value_type) => self.string_to_value_type(value_type),
                                    _ => return Err(ParseError::new("unexpected token", &token)),
                                },
                                _ => return Err(ParseError::new("unexpected token", &token)),
                            };
                            self.extract_token(TokenKind::Gt);
                            ValueType::OptionType(Box::new(value_type))
                        },
                        _ => {
                            println!("{:?}", token);
                            panic!("undefined type")
                        },
                    },
                    _ => panic!("undefined type"),
                };
                match self.consume_token() {
                    Some(Token{kind: TokenKind::Equal, ..}) => {
                        println!("Parsing assign with type");
                        let value = self.parse_expression(0)?;
                        println!("Value: {:?}", value);
                        let variable_type = if mutable_or_immutable.kind == TokenKind::Mutable {
                            EnvVariableType::Mutable
                        } else {
                            EnvVariableType::Immutable
                        };
                        println!("ffVariable type: {:?}", variable_type);
                        self.register_variables(scope, &name, &value_type, &variable_type);
                        Ok(ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type,
                            value_type,
                            is_new: true,
                            line: mutable_or_immutable.line,
                            column: mutable_or_immutable.column,
                        })
                    }
                    _ => panic!("No valid statement found on the right-hand side"),
                }
            }
            _ => panic!("unexpected token"),
        }
    }
}
