use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::{Token, TokenKind};
use crate::environment::ValueType;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_if(&mut self) -> Result<ASTNode, ParseError> {
        match self.get_current_token() {
            Some(Token{kind: TokenKind::If, ..}) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let condition = match self.get_current_token() {
            Some(Token{kind: TokenKind::LParen, ..}) => self.parse_expression(0)?,
            _ => {
                let current_token = self.get_current_token().unwrap();
                return Err(ParseError::new("unexpected token missing (", &current_token))
            }
        };
        let then = self.parse_expression(0)?;
        match self.get_current_token() {
            Some(Token{kind: TokenKind::Eof, ..}) => {
                self.pos = 0;
                self.line += 1;
            },
            _ => {}
        };
        let else_ = match self.get_current_token() {
            Some(Token{kind: TokenKind::Else, ..}) => {
                self.consume_token();
                match self.get_current_token() {
                    Some(Token{kind: TokenKind::If, ..}) => Some(Box::new(self.parse_if()?)),
                    _ => Some(Box::new(self.parse_expression(0)?)),
                }
            }
            _ => None,
        };

        let value_type = {
            let mut then_type = None;
            let mut else_type = None;

            match then {
                ASTNode::Return{expr: ref value, ..} => {
                    then_type = Some(self.infer_type(&value));
                },
                ASTNode::Block{nodes: ref statements, ..} => {
                    for statement in statements {
                        if let ASTNode::Return{expr: ref value, ..} = statement {
                            then_type = Some(self.infer_type(&value));
                        }
                    }
                }
                _ => {}
            }

            if let Some(else_node) = &else_ {
                match &**else_node {
                    ASTNode::Return{expr: ref value, ..} => {
                        else_type = Some(self.infer_type(&value));
                    },
                    ASTNode::Block{nodes: ref statements, ..} => {
                        for statement in statements {
                            if let ASTNode::Return{expr: ref value, ..} = statement { 
                                else_type = Some(self.infer_type(&value));
                            }
                        }
                    }
                    _ => {}
                }
            }

            match (then_type, else_type) {
                (Some(t), Some(e)) if t == e => t,
                (Some(t), None) => t,
                (None, Some(e)) => e,
                (None, None) => Ok(ValueType::Void),
                _ => Err("Type mismatch in if statement".to_string())
            }
        };
        if value_type.is_err() {
            panic!("{}", value_type.err().unwrap());
        }

        let (line, column) = match self.get_current_token() {
            Some(token) => (token.line, token.column),
            None => (self.line, self.pos),
        };

        Ok(ASTNode::If {
            condition: Box::new(condition),
            then: Box::new(then),
            else_,
            value_type: value_type.unwrap(),
            line,
            column
        })
    }
}
