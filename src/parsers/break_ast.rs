use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_break(&mut self) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Break{line, column})
    }
}
