use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::ValueType;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_continue(&mut self) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Continue{line, column})
    }
}
