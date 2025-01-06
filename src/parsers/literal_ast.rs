use crate::ast::ASTNode;
use crate::value::Value;
use crate::parsers::Parser;

impl Parser {
    pub fn parse_literal(&mut self, value: Value) -> ASTNode {
        self.pos += 1;
        ASTNode::Literal(value)
    }
}
