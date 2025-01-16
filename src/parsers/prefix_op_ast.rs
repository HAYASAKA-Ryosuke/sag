use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::TokenKind;

impl Parser {
    pub fn parse_prefix_op(&mut self, op: TokenKind) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(std::u8::MAX);
        ASTNode::PrefixOp {
            op,
            expr: Box::new(value),
        }
    }

}
