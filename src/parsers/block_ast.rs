use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;

impl Parser {
    pub fn parse_block(&mut self) -> ASTNode {
        let mut statements = Vec::new();
        match self.get_current_token() {
            Some(Token{kind: TokenKind::LBrace, ..}) => {},
            _ => panic!("Expected LBrace at the start of a block")
        }
        self.consume_token();

        loop {
            let token = self.get_current_token();
            match token {
                Some(Token{kind: TokenKind::RBrace, ..}) => {
                    self.consume_token();
                    break;
                },
                _ => {}
            }
            match token {
                Some(Token{kind: TokenKind::Eof, ..}) => {
                    self.pos = 0;
                    self.line += 1;
                    continue;
                },
                _ => {}
            }
            if token == None {
                if self.line >= self.tokens.len() {
                    break;
                }
                self.pos = 0;
                self.line += 1;
                continue;
            }
            let statement = self.parse_expression(0);
            statements.push(statement);
        }

        ASTNode::Block(statements)
    }
}
