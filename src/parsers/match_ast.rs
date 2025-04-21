use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::TokenKind;
use crate::parsers::parse_error::ParseError;
use crate::environment::ValueType;

impl Parser {

    fn get_type(&self, ast: &ASTNode) -> Option<ValueType> {
        match &ast {
            ASTNode::Literal { value, .. } => {
                Some(value.value_type())
            }
            ASTNode::Variable { value_type, .. } => {
                value_type.clone()
            }
            ASTNode::Block { nodes, .. } => {
                println!("nodes: {:?}", nodes);
                if let Some(node) = nodes.last() {
                    match node {
                        ASTNode::Literal { value, .. } => Some(value.value_type()),
                        ASTNode::Variable { value_type, .. } => value_type.clone(),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => {
                None
            }
        }
    }

    pub fn parse_match(&mut self) -> Result<ASTNode, ParseError> {
        self.consume_token();
        let expression = self.parse_expression(0)?;
        self.extract_token(TokenKind::LBrace);
        let mut cases = vec![];
        let mut case_pattern_type: Option<ValueType> = None;
        let mut case_body_type: Option<ValueType> = None;
        while self.get_current_token().is_some() && self.get_current_token().unwrap().kind != TokenKind::RBrace {
            if self.get_current_token().unwrap().kind == TokenKind::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            let pattern = self.parse_expression(0)?;
            self.extract_token(TokenKind::RRocket);
            let body = self.parse_block()?;
            cases.push((pattern.clone(), body.clone()));
            if case_pattern_type.is_none() {
                case_pattern_type = self.get_type(&pattern);
                if case_pattern_type.is_none() {
                    return Err(ParseError::new("Unsupported pattern", &self.get_current_token().unwrap()))
                }
            } else if case_pattern_type != self.get_type(&pattern) {
                return Err(ParseError::new("Pattern type mismatch", &self.get_current_token().unwrap()));
            }
            if case_body_type.is_none() {
                case_body_type = self.get_type(&body);
                if case_body_type.is_none() {
                    return Err(ParseError::new("Unsupported pattern", &self.get_current_token().unwrap()));
                }
            } else if case_body_type != self.get_type(&body) {
                return Err(ParseError::new("Pattern type mismatch", &self.get_current_token().unwrap()));
            }
        }

        self.extract_token(TokenKind::RBrace);
        let (line, column) = self.get_line_column();
        Ok(ASTNode::Match {
            expression: Box::new(expression),
            cases,
            line,
            column
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::Parser;
    use crate::token::{Token, TokenKind};
    use crate::ast::ASTNode;
    use crate::value::Value;
    use std::collections::HashMap;
    use fraction::Fraction;

    #[test]
    fn test_parse_match() {
        let mut parser = Parser::new(vec![
            Token{kind: TokenKind::Match, line: 1, column: 1},
            Token{kind: TokenKind::LParen, line: 1, column: 7},
            Token{kind: TokenKind::Identifier("a".to_string()), line: 1, column: 8},
            Token{kind: TokenKind::RParen, line: 1, column: 9},
            Token{kind: TokenKind::LBrace, line: 1, column: 11},
            Token{kind: TokenKind::Number(Fraction::from(1)), line: 1, column: 13},
            Token{kind: TokenKind::RRocket, line: 1, column: 15},
            Token{kind: TokenKind::LBrace, line: 1, column: 17},
            Token{kind: TokenKind::Number(Fraction::from(2)), line: 1, column: 18},
            Token{kind: TokenKind::RBrace, line: 1, column: 20},
            Token{kind: TokenKind::RBrace, line: 1, column: 20},
        ], HashMap::new());
        let result = parser.parse();
        assert_eq!(result.is_ok(), true);
        let ast = result.unwrap();
        match ast {
            ASTNode::Match { expression, cases, .. } => {
                assert_eq!(*expression, ASTNode::Variable{name: "a".to_string(), value_type: None, line: 0, column: 3});
                assert_eq!(cases.len(), 1);
                assert_eq!(cases[0].0, ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 1, column: 15});
                assert_eq!(cases[0].1, ASTNode::Block{nodes: vec![ASTNode::Literal{value: Value::Number(Fraction::from(2)), line: 1, column: 20}], line: 1, column: 20});
            },
            _ => panic!("unexpected ast: {:?}", ast),
        }
    }
}
