use crate::ast::ASTNode;
use crate::environment::{EnvVariableType, ValueType};
use crate::token::{Token, TokenKind};
use crate::parsers::Parser;
use crate::parsers::parse_error::ParseError;

impl Parser {
    pub fn parse_for(&mut self) -> Result<ASTNode, ParseError> {
        let (line, column) = match self.get_current_token() {
            Some(token) => (token.line, token.column),
            None => (self.line, self.pos),
        };
        match self.get_current_token() {
            Some(Token{kind: TokenKind::For, ..}) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let variable = match self.get_current_token() {
            Some(Token{kind: TokenKind::Identifier(name), ..}) => name,
            _ => panic!("unexpected token"),
        };
        self.consume_token();
        self.extract_token(TokenKind::In);
        let iterable = self.parse_expression(0)?;
        let variable_value_type = self.infer_type(&iterable).unwrap_or(ValueType::Any);
        self.register_variables(self.get_current_scope().clone(), &variable, &variable_value_type, &EnvVariableType::Mutable);
        let body = self.parse_expression(0)?;
        Ok(ASTNode::For {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
            line,
            column,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::environment::Env;
    use crate::builtin::register_builtins;

    #[test]
    fn test_parse_for() {
        let input = "for i in range(10) { i }".to_string();
        let tokens = tokenize(&input);
        let builtin = register_builtins(&mut Env::new());
        let mut parser = Parser::new(tokens, builtin);
        let ast = parser.parse_for();
        match ast {
            Ok(ASTNode::For { variable, iterable, body, .. }) => {
                assert_eq!(variable, "i");
                match iterable.as_ref() {
                    ASTNode::FunctionCall { name, .. } => assert_eq!(name, "range"),
                    _ => panic!("unexpected ast"),
                }
                match body.as_ref() {
                    ASTNode::Block { nodes, .. } => {
                        assert_eq!(nodes.len(), 1);
                        match &nodes[0] {
                            ASTNode::Variable { name, .. } => assert_eq!(name, "i"),
                            _ => panic!("unexpected ast"),
                        }
                    },
                    _ => panic!("unexpected ast"),
                }
            },
            _ => panic!("unexpected ast"),
        }
    }
}
