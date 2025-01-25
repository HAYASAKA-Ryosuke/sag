use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::{Token, TokenKind};
use crate::environment::{ValueType, EnvVariableType};
use crate::parsers::parse_error::ParseError;
use std::collections::HashMap;

impl Parser {
    pub fn parse_identifier(&mut self, name: String) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        let scope = self.get_current_scope().to_string();
        let variable_info = self.find_variables(scope.clone(), name.clone());
        match self.get_current_token() {
            Some(Token{kind: TokenKind::LBrace, ..}) => self.create_struct_instance(name.clone()),
            Some(Token{kind: TokenKind::LParen, ..}) => self.create_function_call(name.clone()),
            Some(Token{kind: TokenKind::Equal, ..}) => self.create_assignment(name.clone(), variable_info),
            Some(Token{kind: TokenKind::Colon, ..}) => self.create_variable_declaration(name.clone()),
            Some(Token{kind: TokenKind::Dot, ..}) => self.create_struct_field_access(name.clone()),
            _ => {
                // 代入
                let value_type = if variable_info.is_some() {
                    Some(variable_info.unwrap().0)
                } else {
                    // structに所属しているかチェック
                    match self.get_current_struct(){
                        Some(struct_name) => {
                            let struct_info = self.get_struct(scope.clone(), struct_name.clone());
                            match struct_info {
                                Some(ValueType::Struct { fields, .. }) => {
                                    let field_info = fields.get(&name);
                                    match field_info {
                                        Some(field_info) => Some(field_info.clone()),
                                        None => None
                                    }
                                },
                                _ => None
                            }
                        },
                        None => None
                    }
                };
                let line = self.line;
                let column = self.pos;
                Ok(ASTNode::Variable { name, value_type, line, column })
            }

        }
    }
    fn create_struct_instance(&mut self, name: String) -> Result<ASTNode, ParseError> {
        // 構造体のインスタンス化
        self.consume_token();
        let mut fields = HashMap::new();
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::RBrace {
                self.consume_token();
                break;
            }
            if token.kind == TokenKind::Comma {
                self.consume_token();
                continue;
            }
            if let TokenKind::Identifier(field_name) = token.kind {
                self.consume_token();
                self.extract_token(TokenKind::Colon);
                let value = self.parse_expression(0)?;
                fields.insert(field_name, value);
                continue;
            }
        }
        let (line, column) = self.get_line_column();
        Ok(ASTNode::StructInstance { name, fields, line, column })
    }

    fn create_function_call(&mut self, name: String) -> Result<ASTNode, ParseError> {
        // 関数呼び出し
        self.consume_token();
        let arguments = self.parse_function_call_arguments_paren()?;
        let function_call = self.parse_function_call_front(name, arguments)?;
        Ok(function_call)
    }

    fn create_assignment(&mut self, name: String, variable_info: Option<(ValueType, EnvVariableType)>) -> Result<ASTNode, ParseError> {
        // 再代入
        self.consume_token();
        println!("re create_assignment: {:?}", name);
        println!("re variable_info: {:?}", variable_info);
        if variable_info.is_none() {
            let current_token = self.get_current_token().unwrap();
            return Err(ParseError::new(
                format!("undefined variable: {:?}", name).as_str(),
                &current_token,
            ));
        }
        let (value_type, variable_type) = variable_info.clone().unwrap();
        if variable_type == EnvVariableType::Immutable {
            let current_token = self.get_current_token().unwrap();
            return Err(ParseError::new(
                format!("It is an immutable variable and cannot be reassigned: {:?}", name).as_str(),
                &current_token,
            ));
        }
        let value = self.parse_expression(0)?;
        let infer_type = self.infer_type(&value);
        if infer_type.is_err() {
            let current_token = self.get_current_token().unwrap();
            return Err(ParseError::new(
                format!("undefined type").as_str(),
                &current_token,
            ));
        }
        match value_type {
            ValueType::Any => {}
            ValueType::OptionType(_) => {
                match value {
                    ASTNode::OptionNone { .. } => {},
                    ASTNode::OptionSome { value: _, .. } => {
                        if value_type != infer_type.unwrap() {
                            let current_token = self.get_current_token().unwrap();
                            return Err(ParseError::new(
                                format!("type mismatch").as_str(),
                                &current_token,
                            ));
                        }
                    },
                    _ => {
                        let current_token = self.get_current_token().unwrap();
                        return Err(ParseError::new(
                            format!("type mismatch").as_str(),
                            &current_token,
                        ));
                    }
                }
            }
            _ => {
                if value_type != infer_type.unwrap() {
                    let current_token = self.get_current_token().unwrap();
                    return Err(ParseError::new(
                        format!("type mismatch").as_str(),
                        &current_token,
                    ));
                }
            }
        };
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Assign {
            name,
            value: Box::new(value),
            variable_type,
            value_type,
            is_new: false,
            line,
            column,
        })
    }
    fn create_variable_declaration(&mut self, name: String) -> Result<ASTNode, ParseError> {
        self.consume_token();
        let value_type =
            if let Some(Token{kind: TokenKind::Identifier(type_name), ..}) = self.get_current_token() {
                Some(self.string_to_value_type(type_name))
            } else {
                let current_token = self.get_current_token().unwrap();
                return Err(ParseError::new(
                    format!("undefined type").as_str(),
                    &current_token,
                ));
            };
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Variable { name, value_type, line, column })
    }

    fn create_struct_field_access(&mut self, name: String) -> Result<ASTNode, ParseError> {
        self.pos += 2;
        match self.get_current_token() {
            Some(Token{kind: TokenKind::LParen, ..}) => {
                self.pos -= 1;
                let method_name = match self.get_current_token() {
                    Some(Token{kind: TokenKind::Identifier(method_name), ..}) => method_name,
                    _ => panic!("missing method name: {:?}", self.get_current_token())
                };
                self.pos += 1;
                let arguments = self.parse_function_call_arguments_paren()?;
                let (line, column) = self.get_line_column();
                let caller_variable_ast = ASTNode::Variable {
                    name: name.clone(),
                    value_type: None,
                    line,
                    column,
                };
                return Ok(self.parse_method_call(caller_variable_ast, method_name.to_string(), arguments)?);
            }
            _ => {}
        }
        self.pos -= 2;
        
        // 構造体のフィールドアクセス
        let struct_instance_access = self.parse_struct_instance_access(name.clone())?;
        // 代入
        if let Some(Token{kind: TokenKind::Equal, ..}) = self.get_current_token() {
            self.consume_token();
            let value = self.parse_expression(0)?;
            let field_name = match struct_instance_access.clone() {
                ASTNode::StructFieldAccess { field_name, .. } => field_name,
                _ => panic!("unexpected token"),
            };
            let (line, column) = self.get_line_column();
            Ok(ASTNode::StructFieldAssign {
                instance: Box::new(struct_instance_access),
                field_name: field_name.clone(),
                value: Box::new(value),
                line,
                column,
            })
        } else if let Some(Token{kind: TokenKind::Dot, ..}) = self.get_current_token() {
            match struct_instance_access.clone() {
                ASTNode::StructFieldAccess { field_name, instance: _, .. } => {
                    self.parse_identifier(field_name)
                },
                _ => panic!("unexpected token"),
            }
        } else {
            Ok(struct_instance_access)
        }
    }
}
