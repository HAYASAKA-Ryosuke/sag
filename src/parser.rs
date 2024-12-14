use crate::tokenizer::Token;
use crate::environment::{EnvVariableType, ValueType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Function,
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
    Block(Vec<ASTNode>),
    // -5, !trueなどの一つのオペランドを持つ演算子
    PrefixOp {op: Token, expr: Box<ASTNode>},
    // 1 + 2のような二項演算子
    BinaryOp {left: Box<ASTNode>, op: Token, right: Box<ASTNode>},
    // 変数の代入
    Assign {name: String, value: Box<ASTNode>, variable_type: EnvVariableType, value_type: ValueType},
    Function {name: String, arguments: Vec<ASTNode>, body: Box<ASTNode>, return_type: ValueType},
    FunctionCall {name: String, arguments: Vec<ASTNode>},
    Return(Box<ASTNode>),
}

pub struct Parser {
    tokens: Vec<Vec<Token>>,
    pos: usize,
    line: usize,
    scopes: Vec<String>,
    variables: HashMap<(String, String), ValueType> // key: (scope, name), value: value_type
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let lines = Self::split_lines(tokens);
        Parser{tokens: lines.clone(), pos: 0, line: 0, scopes: vec!["global".into()], variables: HashMap::new()}
    }

    fn enter_scope(&mut self, scope_name: String) {
        self.scopes.push(scope_name);
    }
    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_current_scope(&mut self) -> String {
        self.scopes.last().unwrap().to_string()
    }

    fn register_variables(&mut self, scope: String, name: &String, value_type: &ValueType) {
        self.variables.insert((scope, name.to_string()), value_type.clone());
    }

    fn find_variables(&mut self, scope: String, name: String) -> Option<ValueType> {
        match self.variables.get(&(scope.to_string(), name.to_string())) {
            Some(value_type) => match value_type {
                &ValueType::Number => Some(ValueType::Number),
                &ValueType::Str => Some(ValueType::Str),
                &ValueType::Bool => Some(ValueType::Bool),
                &ValueType::Function => Some(ValueType::Function),
                _ => None
            },
            None => None
        }
    }

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
        if !current_line.is_empty() {
            current_line.push(Token::Eof);
            lines.push(current_line);
        }
        lines
    }

    pub fn get_current_token(&self) -> Option<Token> {
        if self.pos >= self.tokens[self.line].len() {
            None
        } else {
            Some(self.tokens[self.line][self.pos].clone())
        }
    }

    pub fn consume_token(&mut self) -> Option<Token> {
        let token = self.get_current_token()?.clone();
        self.pos += 1;
        Some(token)
    }

    pub fn extract_token(&mut self, token: Token) -> Token {
        match self.get_current_token() {
            Some(current_token) if current_token == token => {
                self.pos += 1;
                current_token
            },
            _ => panic!("unexpected token")
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

    fn parse_prefix_op(&mut self, op: Token) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(std::u8::MAX);
        ASTNode::PrefixOp {
            op,
            expr: Box::new(value)
        }
    }

    fn parse_return(&mut self) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(0);
        ASTNode::Return(Box::new(value))
    }

    fn parse_literal(&mut self, value: Value) -> ASTNode {
        self.pos += 1;
        ASTNode::Literal(value)
    }

    fn parse_function(&mut self) -> ASTNode {
        self.pos += 1;
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("undefined function name")
        };
        self.enter_scope(name.to_string());
        self.pos += 1;
        self.extract_token(Token::Equal);
        self.extract_token(Token::LParen);

        let arguments = self.parse_function_arguments();
        let return_type = self.parse_return_type();
        let body = self.parse_block();

        self.leave_scope();

        ASTNode::Function {
            name,
            arguments,
            body: Box::new(body),
            return_type
        }

    }
    fn string_to_value_type(&mut self, type_name: String) -> ValueType {
        match type_name.as_str() {
            "number" => ValueType::Number,
            "str" => ValueType::Str,
            "bool" => ValueType::Bool,
            _ => panic!("undefined type")
        }
    }
    fn parse_function_arguments(&mut self) -> Vec<ASTNode> {
        let scope = self.get_current_scope();
        let mut arguments = Vec::new();
        while let Some(token) = self.get_current_token() {
            if token == Token::RParen {
                break;
            }
            if let Token::Identifier(name) = self.consume_token().unwrap() {
                self.extract_token(Token::Colon);
                let arg_type = match self.consume_token() {
                    Some(Token::Identifier(type_name)) => self.string_to_value_type(type_name),
                    _ => panic!("Expected type for argument"),
                };
                self.register_variables(scope.to_string(), &name, &arg_type);
                arguments.push(ASTNode::Variable {
                    name,
                    value_type: Some(arg_type),
                });
            }
            if self.get_current_token() == Some(Token::Comma) {
                self.consume_token();
            }
        }
        self.extract_token(Token::RParen);
        arguments
    }

    fn parse_return_type(&mut self) -> ValueType {
        if self.get_current_token() == Some(Token::Colon) {
            self.consume_token();
            if let Some(Token::Identifier(type_name)) = self.consume_token() {
                return self.string_to_value_type(type_name);
            }
        }
        ValueType::Void
    }

    fn parse_assign(&mut self) -> ASTNode {
        let scope = self.get_current_scope();
        let mutable_or_immutable = self.consume_token().unwrap();
        let name = match self.consume_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token")
        };
        match self.consume_token() {
            Some(Token::Equal) => {
                let value = self.parse_expression(0);
                let value_type = match self.infer_type(&value) {
                    Ok(value_type) => value_type,
                    Err(e) => panic!("{}", e)
                };
                self.register_variables(scope, &name, &value_type);
                ASTNode::Assign {
                    name,
                    value: Box::new(value),
                    variable_type: if mutable_or_immutable == Token::Mutable {EnvVariableType::Mutable} else {EnvVariableType::Immutable},
                    value_type
                }
            },
            Some(Token::Colon) => {
                let value_type = match self.consume_token() {
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
                match self.consume_token() {
                    Some(Token::Equal) => {
                        let value = self.parse_expression(0);
                        self.register_variables(scope, &name, &value_type);
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type: if mutable_or_immutable == Token::Mutable {EnvVariableType::Mutable} else {EnvVariableType::Immutable},
                            value_type
                        }
                    },
                    _ => panic!("No valid statement found on the right-hand side")
                }
            }
            _ => panic!("unexpected token")
        }
    }

    fn parse_primary(&mut self) -> ASTNode {
        let token = match self.get_current_token() {
            Some(token) => token,
            _ => panic!("token not found!")
        };
        match token {
            Token::Minus => self.parse_prefix_op(Token::Minus),
            Token::Return => self.parse_return(),
            Token::Num(value) => self.parse_literal(Value::Number(value.into())),
            Token::String(value) => self.parse_literal(Value::Str(value.into())),
            Token::Function => self.parse_function(),
            Token::Mutable | Token::Immutable => self.parse_assign(),
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
                let scope = self.get_current_scope().to_string();
                let value_type = self.find_variables(scope, name.clone());
                ASTNode::Variable{name, value_type}
            },
            _ => panic!("undefined token: {:?}", token)
        }
    }

    fn parse_block(&mut self) -> ASTNode {
        self.consume_token();
        let mut statements = Vec::new();

        println!("parse block");
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrace {
                self.pos += 1;
                break;
            }
            let statement = self.parse_expression(0);
            statements.push(statement);
            self.pos += 1;
            match self.get_current_token() {
                Some(Token::Eof) => {
                    self.pos += 1;
                }
                _ => {}
            };
        }
        ASTNode::Block(statements)
    }

    fn parse_expression(&mut self, min_priority: u8) -> ASTNode {
        let mut lhs = self.parse_primary();

        loop {
            let token = match self.get_current_token() {
                Some(token) => token,
                //_ => panic!("token not found")
                _ => {break;}
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
            println!("ast_nodes: {:?}", ast_nodes);
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

    #[test]
    fn test_function2() {
        let mut parser = Parser::new(vec![Token::Function, Token::Identifier("foo".into()), Token::Equal, Token::LParen, Token::Identifier("x".into()), Token::Colon, Token::Identifier("number".into()), Token::Comma, Token::Identifier("y".into()), Token::Colon, Token::Identifier("number".into()), Token::RParen, Token::Colon, Token::Identifier("number".into()), Token::LBrace, Token::Return, Token::Identifier("x".into()), Token::Plus, Token::Identifier("y".into()), Token::RBrace, Token::Eof]);
        assert_eq!(parser.parse(), ASTNode::Function {
            name: "foo".into(),
            arguments: vec![ASTNode::Variable{name: "x".into(), value_type: Some(ValueType::Number)}, ASTNode::Variable{name: "y".into(), value_type: Some(ValueType::Number)}],
            body: Box::new(
                ASTNode::Block(vec![
                    ASTNode::Return(Box::new(
                        ASTNode::BinaryOp{
                            left: Box::new(ASTNode::Variable{name: "x".into(), value_type: Some(ValueType::Number)}),
                            op: Token::Plus,
                            right: Box::new(ASTNode::Variable{name: "y".into(), value_type: Some(ValueType::Number)}),
                        }
                    ))
                ])
            ),
            return_type: ValueType::Number
        })
    }
}
