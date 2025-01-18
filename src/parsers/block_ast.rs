use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_block(&mut self) -> Result<ASTNode, ParseError> {
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
            let statement = self.parse_expression(0)?;
            statements.push(statement);
        }

        let current_token = self.get_current_token();
        let (line, column) = match current_token {
            Some(token) => (token.line, token.column),
            None => (self.line, self.pos),
        };

        Ok(ASTNode::Block{nodes: statements, line, column})
    }
}
