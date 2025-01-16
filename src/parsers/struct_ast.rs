use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::ValueType;
use std::collections::HashMap;

impl Parser {
    pub fn parse_struct(&mut self) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };
        self.enter_struct(name.clone());
        if name[0..1] != name[0..1].to_uppercase() {
            panic!("struct name must start with a capital letter");
        }
        self.consume_token();
        self.extract_token(TokenKind::LBrace);
        let mut fields = HashMap::new();
        let mut field_is_public = false;
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::RBrace {
                self.consume_token();
                break;
            }
            if token.kind == TokenKind::Comma {
                self.consume_token();
                continue;
            }
            if token.kind == TokenKind::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            if token.kind == TokenKind::Pub {
                field_is_public = true;
                self.consume_token();
                continue;
            }
    
            if let Token{kind: TokenKind::Identifier(name), ..} = token {
                self.consume_token();
                self.extract_token(TokenKind::Colon);
                let value_type = match self.get_current_token() {
                    Some(Token{kind: TokenKind::Identifier(type_name), ..}) => self.string_to_value_type(type_name),
                    _ => panic!("undefined type"),
                };
                fields.insert(name, ASTNode::StructField {
                    value_type,
                    is_public: field_is_public,
                });
                self.consume_token();
                field_is_public = false;
                continue;
            }
        }
        let result = ASTNode::Struct { name, fields };
        let scope = self.get_current_scope().clone();
        self.register_struct(scope, result.clone());
        self.leave_struct();
        result
    }

    pub fn parse_struct_instance_access(&mut self, name: String) -> ASTNode {
        self.consume_token();
        let field_name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };
        self.consume_token();
        let scope = self.get_current_scope().clone();
        if name == "self" {
            if self.current_struct.is_none() {
                panic!("undefined struct for self");
            }
            let current_struct = self.current_struct.clone().unwrap();
            let struct_type = self
                .get_struct(scope.clone(), current_struct.to_string())
                .expect("undefined struct for self");

            return ASTNode::StructFieldAccess {
                instance: Box::new(ASTNode::Variable {
                    name: "self".to_string(),
                    value_type: Some(struct_type.clone()),
                }),
                field_name,
            };
        }

        match self.find_variables(scope.clone(), name.clone()) {
            Some((ValueType::StructInstance { name: instance_name, ref fields }, _)) => {
                ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable { name: name.clone(), value_type: Some(ValueType::StructInstance {name: instance_name, fields: fields.clone()}) }),
                    field_name,
                }
            }
            _ => panic!("undefined struct: {:?}", name),
        }
    }

    pub fn parse_impl(&mut self) -> ASTNode {
        self.consume_token();
        let scope = self.get_current_scope().clone();
        let struct_name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };

        self.enter_struct(struct_name.clone());

        let base_struct = self.get_struct(scope.clone(),struct_name.to_string()).expect("undefined struct");
        self.current_struct = Some(struct_name.clone());
        self.consume_token();
        self.extract_token(TokenKind::LBrace);
        let mut methods = Vec::new();
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::RBrace {
                self.consume_token();
                break;
            }
            if token.kind == TokenKind::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            if token.kind == TokenKind::Comma {
                self.consume_token();
                continue;
            }
            if token.kind == TokenKind::Function {
                let method = self.parse_method();
                methods.push(method);
                continue;
            }
        }
        self.current_struct = None;
        self.leave_struct();
        ASTNode::Impl {
            base_struct: Box::new(base_struct),
            methods,
        }
    }
}
