use crate::ast::ASTNode;
use crate::value::Value;
use crate::parsers::Parser;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_literal(&mut self, value: Value) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        Ok(ASTNode::Literal(value))
    }
}
