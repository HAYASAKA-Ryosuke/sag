use crate::tokenizer::Token;
use crate::environment::{EnvVariableType, ValueType};

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
    Variable {
        name: String,
        value_type: Option<ValueType>
    },
    // -5, !trueなどの一つのオペランドを持つ演算子
    PrefixOp {op: Token, expr: Box<ASTNode>},
    // 1 + 2のような二項演算子
    BinaryOp {left: Box<ASTNode>, op: Token, right: Box<ASTNode>},
    // 変数の代入
    Assign {name: String, value: Box<ASTNode>, variable_type: EnvVariableType, value_type: ValueType},
}

pub struct Parser {
    tokens: Vec<Vec<Token>>,
    pos: usize,
    line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        fn split_lines(tokens: Vec<Token>) -> Vec<Vec<Token>> {
            let mut lines = Vec::new();
            let mut current_line = Vec::new();
            for token in tokens {
                if token == Token::Eof {
                    if !current_line.is_empty() {
                        current_line.push(Token::Eof);
                        lines.push(current_line);
                        current_line = Vec::new();
                    }
                } else {
                    current_line.push(token);
                }
            }
            lines
        }
        let lines = split_lines(tokens);
        Parser{tokens: lines.clone(), pos: 0, line: 0}
    }


    pub fn get_current_token(&self) -> Option<Token> {
        if self.pos >= self.tokens[self.line].len() {
            None
        } else {
            Some(self.tokens[self.line][self.pos].clone())
        }
    }

    fn infer_type(&self, ast: &ASTNode) -> Result<ValueType, String> {
        match ast {
            ASTNode::Literal(ref v) => match v {
                Value::Number(_) => Ok(ValueType::Number),
                Value::Str(_) => Ok(ValueType::Str),
                Value::Bool(_) => Ok(ValueType::Bool),
                _ => Ok(ValueType::Any)
            },
            ASTNode::PrefixOp { op: _, expr } => {
                let value_type = self.infer_type(&expr)?;
                Ok(value_type)
            }
            ASTNode::BinaryOp { left, op, right } => {
                let left_type = self.infer_type(&left)?;
                let right_type = self.infer_type(&right)?;

                match (&left_type, &right_type) {
                    (ValueType::Number, ValueType::Number) => Ok(ValueType::Number),
                    (ValueType::Number, ValueType::Str) => Ok(ValueType::Str),
                    (ValueType::Str, ValueType::Number) => Ok(ValueType::Str),
                    (ValueType::Bool, ValueType::Bool) => Ok(ValueType::Bool),
                    _ => Err(format!("type mismatch: {:?} {:?} {:?}", left_type, op, right_type).into())
                }
            }
            _ => Ok(ValueType::Any)
        }
    }

    fn parse_primary(&mut self) -> ASTNode {
        let token = match self.get_current_token() {
            Some(token) => token,
            _ => panic!("token not found!")
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
                        let value_type = match self.infer_type(&value) {
                            Ok(value_type) => value_type,
                            Err(e) => panic!("{}", e)
                        };
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type: if token == Token::Mutable {EnvVariableType::Mutable} else {EnvVariableType::Immutable},
                            value_type
                        }
                    },
                    Some(Token::Colon) => {
                        self.pos += 1;
                        let value_type = match self.get_current_token() {
                            Some(token) => match token {
                                Token::Identifier(value_type) => {
                                    match value_type.as_str() {
                                        "number" => ValueType::Number,
                                        "str" => ValueType::Str,
                                        "bool" => ValueType::Bool,
                                        _ => panic!("undefined type: {:?}", value_type)
                                    }
                                },
                                _ => panic!("undefined type")
                            },
                            _ => panic!("undefined type")
                        };
                        self.pos += 1;
                        match self.get_current_token() {
                            Some(Token::Equal) => {
                                self.pos += 1;
                                let value = self.parse_expression(0);
                                ASTNode::Assign {
                                    name,
                                    value: Box::new(value),
                                    variable_type: if token == Token::Mutable {EnvVariableType::Mutable} else {EnvVariableType::Immutable},
                                    value_type
                                }
                            },
                            _ => panic!("No valid statement found on the right-hand side")
                        }
                    }
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
                ASTNode::Variable{name, value_type: None}
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

    pub fn parse_lines(&mut self) -> Vec<ASTNode> {
        let mut ast_nodes = vec![];
        for _ in 0..self.tokens.len() {
            println!("line: {:?}", self.tokens[self.line]);
            ast_nodes.push(self.parse());
            self.line += 1;
            self.pos = 0;
        }
        ast_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_basic_arithmetic_operations() {
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
    }

    #[test]
    fn test_type_specified() {
        let mut parser = Parser::new(vec![Token::Mutable, Token::Identifier("x".into()), Token::Colon, Token::Identifier("number".into()), Token::Equal, Token::Num(1), Token::Eof]);
        assert_eq!(parser.parse(), ASTNode::Assign {
            name: "x".into(),
            value: Box::new(ASTNode::Literal(Value::Number(1.0))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        });
    }

    #[test]
    fn test_type_estimate() {
        let mut parser = Parser::new(vec![Token::Mutable, Token::Identifier("x".into()), Token::Equal, Token::Num(1), Token::Eof]);
        assert_eq!(parser.parse(), ASTNode::Assign {
            name: "x".into(),
            value: Box::new(ASTNode::Literal(Value::Number(1.0))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number
        });
    }
}
