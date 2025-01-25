use crate::ast::ASTNode;
use crate::token::{TokenKind};
use crate::parsers::Parser;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_option_some(&mut self) -> Result<ASTNode, ParseError> {
        self.consume_token();
        println!("Parsing option some");
        self.extract_token(TokenKind::LParen);
        let value = self.parse_expression(0)?;
        self.extract_token(TokenKind::RParen);
        let (line, column) = self.get_line_column();
        Ok(ASTNode::OptionSome{value: Box::new(value), line, column})
    }

    pub fn parse_option_none(&mut self) -> Result<ASTNode, ParseError> {
        self.consume_token();
        let (line, column) = self.get_line_column();
        Ok(ASTNode::OptionNone{line, column})
    }
}
