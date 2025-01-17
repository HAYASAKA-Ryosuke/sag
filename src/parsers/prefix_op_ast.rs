use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::TokenKind;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_prefix_op(&mut self, op: TokenKind) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        let value = self.parse_expression(std::u8::MAX)?;
        Ok(ASTNode::PrefixOp {
            op,
            expr: Box::new(value),
        })
    }

}
