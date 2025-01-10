use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;
use crate::environment::{ValueType, EnvVariableType};
use std::collections::HashMap;

impl Parser {
    pub fn parse_identifier(&mut self, name: String) -> ASTNode {
        self.pos += 1;
        let scope = self.get_current_scope().to_string();
        let variable_info = self.find_variables(scope.clone(), name.clone());
        match self.get_current_token() {
            Some(Token::LBrace) => self.create_struct_instance(name.clone()),
            Some(Token::LParen) => self.create_function_call(name.clone()),
            Some(Token::Equal) => self.create_assignment(name.clone(), variable_info),
            Some(Token::Colon) => self.create_variable_declaration(name.clone()),
            Some(Token::Dot) => self.create_struct_field_access(name.clone()),
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
                ASTNode::Variable { name, value_type }
            }

        }
    }
    fn create_struct_instance(&mut self, name: String) -> ASTNode {
        // 構造体のインスタンス化
        self.consume_token();
        let mut fields = HashMap::new();
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrace {
                self.consume_token();
                break;
            }
            if token == Token::Comma {
                self.consume_token();
                continue;
            }
            if let Token::Identifier(field_name) = token {
                self.consume_token();
                self.extract_token(Token::Colon);
                let value = self.parse_expression(0);
                fields.insert(field_name, value);
                continue;
            }
        }
        ASTNode::StructInstance { name, fields }
    }

    fn create_function_call(&mut self, name: String) -> ASTNode {
        // 関数呼び出し
        self.consume_token();
        let arguments = self.parse_function_call_arguments_paren();
        let function_call = self.parse_function_call_front(name, arguments);
        function_call
    }

    fn create_assignment(&mut self, name: String, variable_info: Option<(ValueType, EnvVariableType)>) -> ASTNode {
        // 再代入
        self.consume_token();
        if variable_info.is_none() {
            panic!("missing variable: {:?}", name);
        }
        let (value_type, variable_type) = variable_info.clone().unwrap();
        if variable_type == EnvVariableType::Immutable {
            panic!(
                "It is an immutable variable and cannot be reassigned: {:?}",
                name
            );
        }
        let value = self.parse_expression(0);
        ASTNode::Assign {
            name,
            value: Box::new(value),
            variable_type,
            value_type,
            is_new: false,
        }
    }
    fn create_variable_declaration(&mut self, name: String) -> ASTNode {
        self.consume_token();
        let value_type =
            if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                Some(self.string_to_value_type(type_name))
            } else {
                panic!("undefined type")
            };
        ASTNode::Variable { name, value_type }
    }

    fn create_struct_field_access(&mut self, name: String) -> ASTNode {
        self.pos += 2;
        if Some(Token::LParen) == self.get_current_token() {
            self.pos -= 1;
            let method_name = match self.get_current_token() {
                Some(Token::Identifier(method_name)) => method_name,
                _ => panic!("missing method name: {:?}", self.get_current_token())
            };
            self.pos += 1;
            let arguments = self.parse_function_call_arguments_paren();
            return self.parse_method_call(name.to_string(), method_name.to_string(), arguments);
        }
        self.pos -= 2;
        
        // 構造体のフィールドアクセス
        let struct_instance_access = self.parse_struct_instance_access(name.clone());
        // 代入
        if let Some(Token::Equal) = self.get_current_token() {
            self.consume_token();
            let value = self.parse_expression(0);
            let field_name = match struct_instance_access.clone() {
                ASTNode::StructFieldAccess { field_name, .. } => field_name,
                _ => panic!("unexpected token"),
            };
            ASTNode::StructFieldAssign {
                instance: Box::new(struct_instance_access),
                field_name: field_name.clone(),
                value: Box::new(value),
            }
        } else if let Some(Token::Dot) = self.get_current_token() {
            match struct_instance_access.clone() {
                ASTNode::StructFieldAccess { field_name, instance: _ } => {
                    self.parse_identifier(field_name)
                },
                _ => panic!("unexpected token"),
            }
        } else {
            struct_instance_access
        }
    }
}
