use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;
use crate::environment::ValueType;

impl Parser {
    pub fn parse_if(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::If) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let condition = match self.get_current_token() {
            Some(Token::LParen) => self.parse_expression(0),
            _ => panic!("unexpected token missing ("),
        };
        let then = self.parse_expression(0);
        if self.get_current_token() == Some(Token::Eof) {
            self.pos = 0;
            self.line += 1;
        }

        let else_ = match self.get_current_token() {
            Some(Token::Else) => {
                self.consume_token();
                if self.get_current_token() == Some(Token::If) {
                    Some(Box::new(self.parse_if()))
                } else {
                    Some(Box::new(self.parse_expression(0)))
                }
            }
            _ => None,
        };

        let value_type = {
            let mut then_type = None;
            let mut else_type = None;

            match then {
                ASTNode::Return(ref value) => {
                    then_type = Some(self.infer_type(&value));
                },
                ASTNode::Block(ref statements) => {
                    for statement in statements {
                        if let ASTNode::Return(ref value) = statement {
                            then_type = Some(self.infer_type(&value));
                        }
                    }
                }
                _ => {}
            }

            if let Some(else_node) = &else_ {
                match &**else_node {
                    ASTNode::Return(ref value) => {
                        else_type = Some(self.infer_type(&value));
                    },
                    ASTNode::Block(ref statements) => {
                        for statement in statements {
                            if let ASTNode::Return(ref value) = statement { 
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

        ASTNode::If {
            condition: Box::new(condition),
            then: Box::new(then),
            else_,
            value_type: value_type.unwrap(),
        }
    }
}
