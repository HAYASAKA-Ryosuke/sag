use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::{Token, TokenKind};
use crate::environment::ValueType;
use crate::value::Value;
use crate::parsers::parse_error::ParseError;

impl Parser {

    pub fn parse_method(&mut self) -> Result<ASTNode, ParseError> {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };
        self.enter_scope(name.to_string());
        self.consume_token();
        self.extract_token(TokenKind::LParen);
        let arguments = self.parse_function_arguments()?;
        let mut is_mut = false;
        if arguments.len() > 0 {
            match arguments.first() {
                Some(ASTNode::Variable { name, value_type, .. }) => {
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
        let body = self.parse_block()?;
        self.leave_scope();
        let (line, column) = self.get_line_column();
        let method = ASTNode::Method {
            name: name.clone(),
            arguments,
            body: Box::new(body),
            return_type,
            is_mut,
            line,
            column
        };
        self.register_method(self.get_current_scope(), self.current_struct.clone().unwrap(), method.clone());
        Ok(method)
    }

    fn is_builtin_method(&self, caller: &ASTNode) -> bool {
        let builtin = match caller {
            ASTNode::Literal{value: Value::Number(_), ..} => true,
            ASTNode::Literal{value: Value::String(_), ..} => true,
            ASTNode::Literal{value: Value::Bool(_), ..} => true,
            ASTNode::Literal{value: Value::Void, ..} => true,
            ASTNode::Literal{value: Value::List(_), ..} => true,
            ASTNode::Variable { name, value_type, .. } => {
                if value_type.is_none() {
                    let variable = self.find_variables(self.get_current_scope(), name.clone());
                    match variable {
                        Some((value_type, _)) => {
                            match value_type {
                                ValueType::Number => true,
                                ValueType::String => true,
                                ValueType::Bool => true,
                                ValueType::Void => true,
                                ValueType::List(_) => true,
                                _ => false,
                            }
                        },
                        _ => false,
                    }
                } else {
                    match value_type {
                        Some(value_type) => {
                            match value_type {
                                ValueType::Number => true,
                                ValueType::String => true,
                                ValueType::Bool => true,
                                ValueType::Void => true,
                                ValueType::List(_) => true,
                                _ => false,
                            }
                        },
                        _ => false,
                    }
                }
            },
            ASTNode::MethodCall { caller, .. } => {
                match self.infer_type(&caller) {
                    Ok(ValueType::Number) => true,
                    Ok(ValueType::String) => true,
                    Ok(ValueType::Bool) => true,
                    Ok(ValueType::Void) => true,
                    Ok(ValueType::List(_)) => true,
                    _ => false,
                }
            },
            _ => false,
        };
        builtin
    }

    pub fn parse_method_call(&mut self, caller: ASTNode, method_name: String, arguments: ASTNode) -> Result<ASTNode, ParseError> {
        let builtin = self.is_builtin_method(&caller);
        let (line, column) = self.get_line_column();
        Ok(ASTNode::MethodCall {
            method_name,
            caller: Box::new(caller),
            arguments: Box::new(arguments),
            builtin,
            line,
            column
        })
    }
}
