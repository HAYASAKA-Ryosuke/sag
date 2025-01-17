use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::ValueType;
use std::collections::HashMap;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_struct(&mut self) -> Result<ASTNode, ParseError> {
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
        Ok(result)
    }

    pub fn parse_struct_instance_access(&mut self, name: String) -> Result<ASTNode, ParseError> {
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

            return Ok(ASTNode::StructFieldAccess {
                instance: Box::new(ASTNode::Variable {
                    name: "self".to_string(),
                    value_type: Some(struct_type.clone()),
                }),
                field_name,
            });
        }

        match self.find_variables(scope.clone(), name.clone()) {
            Some((ValueType::StructInstance { name: instance_name, ref fields }, _)) => {
                Ok(ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable { name: name.clone(), value_type: Some(ValueType::StructInstance {name: instance_name, fields: fields.clone()}) }),
                    field_name,
                })
            }
            _ => panic!("undefined struct: {:?}", name),
        }
    }

    pub fn parse_impl(&mut self) -> Result<ASTNode, ParseError> {
        self.consume_token();
        let scope = self.get_current_scope().clone();
        let struct_name = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };

        self.enter_struct(struct_name.clone());

        let base_struct = self.get_struct(scope.clone(),struct_name.to_string());
        if base_struct.is_none() {
            return Err(ParseError::new(format!("undefined struct: {:?}", struct_name).as_str(), &self.get_current_token().unwrap()));
        }
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
                let method = self.parse_method()?;
                methods.push(method);
                continue;
            }
        }
        self.current_struct = None;
        self.leave_struct();
        Ok(ASTNode::Impl {
            base_struct: Box::new(base_struct.unwrap()),
            methods,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::environment::Env;
    use crate::builtin::register_builtins;

    #[test]
    fn test_parse_struct() {
        let input = r#"
struct Point {
  x: number,
  y: number
}

impl Point {
  fun move(mut self, dx: number, dy: number) {
      self.x = self.x + dx
      self.y = self.y + dy
  }
}

impl Point {
  fun clear(mut self) {
      self.x = 0
      self.y = 0
  }
}

val x = 8
val y = 3
val mut point = Point{x: x, y: y}
point.move(5, 2)
point.clear()
"#;
        let tokens = tokenize(&input.to_string());
        let builtin = register_builtins(&mut Env::new());
        let mut parser = Parser::new(tokens, builtin);
        let ast = parser.parse_lines();
        println!("{:?}", ast);
    }
}
