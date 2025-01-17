use crate::ast::ASTNode;
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::environment::ValueType;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_return(&mut self) -> Result<ASTNode, ParseError> {
        self.pos += 1;
        let value = self.parse_expression(0)?;
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Return{expr: Box::new(value), line, column})
    }

    pub fn parse_return_type(&mut self) -> ValueType {
        match self.get_current_token() {
            Some(Token{kind: TokenKind::Colon, ..}) => {
                self.consume_token();
                if let Some(Token{kind: TokenKind::Identifier(type_name), ..}) = self.consume_token() {
                    return self.string_to_value_type(type_name);
                }
            },
            _ => {}
        };
        ValueType::Void
    }
}
