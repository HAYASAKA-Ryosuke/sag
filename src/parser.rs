use crate::tokenizer::Token;
use crate::environment::EnvVariableType;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
}


#[derive(Debug, PartialEq)]
pub enum ASTNode {
    // 数値や文字列などのリテラル
    Literal(Value),
    // 変数
    Variable(String),
    // -5, !trueなどの一つのオペランドを持つ演算子
    PrefixOp {op: Token, expr: Box<ASTNode>},
    // 1 + 2のような二項演算子
    BinaryOp {left: Box<ASTNode>, op: Token, right: Box<ASTNode>},
    // 変数の代入
    Assign {name: String, value: Box<ASTNode>, variable_type: EnvVariableType},
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser{tokens, pos: 0}
    }

    pub fn get_current_token(&self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            Some(self.tokens[self.pos].clone())
        }
    }

    fn parse_primary(&mut self) -> ASTNode {
        let token = match self.get_current_token() {
            Some(token) => token,
            _ => panic!("token not found")
        };
        match token {
            Token::Minus => {
                self.pos += 1;
                let value = self.parse_expression(std::u8::MAX);
                ASTNode::PrefixOp {
                    op: Token::Minus,
                    expr: Box::new(value)
                }
            },
            Token::Num(value) => {
                self.pos += 1;
                ASTNode::Literal(Value::Number(value.into()))
            },
            Token::String(value) => {
                self.pos += 1;
                ASTNode::Literal(Value::Str(value))
            },
            Token::Mutable | Token::Immutable => {
                self.pos += 1;
                let name = match self.get_current_token() {
                    Some(Token::Identifier(name)) => name,
                    _ => panic!("unexpected token")
                };
                self.pos += 1;
                match self.get_current_token() {
                    Some(Token::Equal) => {
                        self.pos += 1;
                        let value = self.parse_expression(0);
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type: if token == Token::Mutable {EnvVariableType::Mutable} else {EnvVariableType::Immutable}
                        }
                    },
                    _ => panic!("unexpected token")
                }
            },
            Token::LParen => {
                self.pos += 1;
                let expr = self.parse_expression(0);
                self.pos += 1;
                match self.get_current_token() {
                    Some(t) => {
                        if t == Token::RParen {
                            panic!("unexpected token: {:?}", t)
                        }
                    },
                    None => panic!("unexpected token: {:?}", token)
                }
                expr
            },
            Token::Identifier(name) => {
                self.pos += 1;
                ASTNode::Variable(name)
            },
            _ => panic!("undefined token: {:?}", token)
        }
    }

    fn parse_expression(&mut self, min_priority: u8) -> ASTNode {
        let mut lhs = self.parse_primary();

        loop {
            let token = match self.get_current_token() {
                Some(token) => token,
                _ => panic!("token not found")
            };
            let priorities = self.get_priority(&token);
            if priorities.is_none() {

                break;
            }
            let (left_priority, right_priority) = priorities.unwrap();
            if left_priority < min_priority {
                println!("left < min");
                break;
            }
            self.pos += 1;

            let rhs = self.parse_expression(right_priority);
            lhs = ASTNode::BinaryOp {
                left: Box::new(lhs),
                op: token,
                right: Box::new(rhs),
            }
        }
        lhs
    }
    fn get_priority(&self, token: &Token) -> Option<(u8, u8)> {
        match token {
            Token::Plus | Token::Minus => Some((1, 2)),
            Token::Mul | Token::Div => Some((3, 4)),
            _ => None
        }
    }

    pub fn parse(&mut self) -> ASTNode {
        self.parse_expression(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new(vec![Token::Minus, Token::Num(1), Token::Plus, Token::Num(2), Token::Mul, Token::Num(3), Token::Eof]);
        assert_eq!(parser.parse(), ASTNode::BinaryOp {
            left: Box::new(ASTNode::PrefixOp{
                op: Token::Minus,
                expr: Box::new(ASTNode::Literal(Value::Number(1.0)))
            }),
            op: Token::Plus,
            right: Box::new(ASTNode::BinaryOp{
                left: Box::new(ASTNode::Literal(Value::Number(2.0))),
                op: Token::Mul,
                right: Box::new(ASTNode::Literal(Value::Number(3.0)))
            })
        });
        let mut parser = Parser::new(vec![Token::Mutable , Token::Identifier("x".into()), Token::Equal, Token::Num(1), Token::Eof]);
        assert_eq!(parser.parse(), ASTNode::Assign {
            name: "x".into(),
            value: Box::new(ASTNode::Literal(Value::Number(1.0))),
            variable_type: EnvVariableType::Mutable
        });

    }
}
