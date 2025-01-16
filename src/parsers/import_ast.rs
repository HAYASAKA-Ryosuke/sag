use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::token::{Token, TokenKind};


impl Parser {
    pub fn parse_import(&mut self) -> ASTNode {
        self.extract_token(TokenKind::Import);
        let mut symbols = vec![];
        while let Some(token) = self.get_current_token() {
            if token.kind == TokenKind::Comma {
                self.consume_token();
                continue;
            }
            if token.kind == TokenKind::From {
                break;
            }
            match token.kind {
                TokenKind::Identifier(name) =>  {
                    self.consume_token();
                    symbols.push(name);
                },
                _ => panic!("Expected identifier"),
            };
        }
        self.extract_token(TokenKind::From);
        let module_name = match self.get_current_token() { Some(Token{kind: TokenKind::Identifier(module_name), ..}) => module_name.clone(),
            _ => panic!("Expected module name"),
        };
        ASTNode::Import { module_name, symbols }
    }

    pub fn parse_public(&mut self) -> ASTNode {
        self.extract_token(TokenKind::Pub);
        ASTNode::Public {node: Box::new(self.parse_expression(0))}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    
    #[test]
    fn test_parse_import() {
        let input = vec![
            Token::Import,
            Token::Identifier("foo1".into()),
            Token::Comma,
            Token::Identifier("foo2".into()),
            Token::Comma,
            Token::Identifier("foo3".into()),
            Token::From,
            Token::Identifier("Foo".into()),
            Token::Eof
        ];
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        match ast {
            ASTNode::Import { module_name, symbols } => {
                assert_eq!(module_name, "Foo");
                assert_eq!(symbols, vec!["foo1", "foo2", "foo3"]);
            }
            _ => panic!("Expected Import"),
        }
    }

    #[test]
    fn test_parse_public() {
        let input = vec![
            Token::Pub,
            Token::Immutable,
            Token::Identifier("foo".into()),
            Token::Equal,
            Token::String("hello".into()),
            Token::Eof
        ];

        let mut parser = Parser::new(input);
        let ast = parser.parse();
        match ast {
            ASTNode::Public { node } => {
                match *node {
                    ASTNode::Assign { name, value, ..} => {
                        assert_eq!(name, "foo");
                        assert_eq!(*value, ASTNode::Literal(Value::String("hello".into())));
                    }
                    _ => panic!("Expected Assignment"),
                }
            }
            _ => panic!("Expected Public"),
        }
    }
}
