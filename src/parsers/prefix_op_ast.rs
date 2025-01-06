use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::Token;

impl Parser {
    pub fn parse_prefix_op(&mut self, op: Token) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(std::u8::MAX);
        ASTNode::PrefixOp {
            op,
            expr: Box::new(value),
        }
    }

}
