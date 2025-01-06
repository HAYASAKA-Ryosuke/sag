use crate::ast::ASTNode;
use crate::token::Token;
use crate::parsers::Parser;
use crate::environment::ValueType;

impl Parser {
    pub fn parse_return(&mut self) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(0);
        ASTNode::Return(Box::new(value))
    }

    pub fn parse_return_type(&mut self) -> ValueType {
        if self.get_current_token() == Some(Token::Colon) {
            self.consume_token();
            if let Some(Token::Identifier(type_name)) = self.consume_token() {
                return self.string_to_value_type(type_name);
            }
        }
        ValueType::Void
    }
}
