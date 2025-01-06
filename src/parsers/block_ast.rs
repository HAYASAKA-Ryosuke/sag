use crate::ast::ASTNode;
use crate::token::Token;
use crate::parsers::Parser;

impl Parser {
    pub fn parse_block(&mut self) -> ASTNode {
        let mut statements = Vec::new();
        if self.get_current_token() != Some(Token::LBrace) {
            panic!("Expected LBrace at the start of a block");
        }
        self.consume_token();

        loop {
            let token = self.get_current_token();
            if token == Some(Token::RBrace) {
                self.consume_token();
                break;
            }
            if token == Some(Token::Eof) {
                self.pos = 0;
                self.line += 1;
                continue;
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
