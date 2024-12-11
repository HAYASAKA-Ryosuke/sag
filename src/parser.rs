use crate::tokenizer::Token;
use std::collections::HashMap;


pub struct Env {
    variable_map: HashMap<VariableKeyInfo, VariableValueInfo>,
    scope_stack: Vec<String>
}


#[derive(Eq, Hash, PartialEq)]
pub struct VariableKeyInfo {
    name: String,
    scope: String, 
}

pub enum VariableType {
    Immutable,
    Mutable
}

pub struct VariableValueInfo {
    value: Value,
    variable_type: VariableType
}

impl Env {
    pub fn new() -> Self {
        Self{variable_map: HashMap::new(), scope_stack: vec![]}
    }

    pub fn enter_scope(&mut self, scope: String) {
        self.scope_stack.push(scope);
    }
    pub fn leave_scope(&mut self, scope: String) {
        self.scope_stack.pop();
    }

    pub fn set(&mut self, name: String, value: Value, scope: String, variable_type: VariableType) -> Result<(), String> {
        let latest_scope = self.scope_stack.last();
        if latest_scope.is_none() {
            return Err("missing scope".into());
        }
        self.variable_map.insert(VariableKeyInfo{name, scope: scope.clone()}, VariableValueInfo{value, variable_type});
        Ok(())
    }

    pub fn get(&mut self, name: String) -> Option<&VariableValueInfo> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(variable_key_info)  = self.variable_map.get(&VariableKeyInfo{name: name.to_string(), scope: scope.clone()}) {
                return Some(variable_key_info);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
}


#[derive(Debug)]
pub enum ASTNode {
    Literal(Value),
    // -5, !trueなどの一つのオペランドを持つ演算子
    PrefixOp {op: Token, expr: Box<ASTNode>},
    // 1 + 2のような二項演算子
    BinaryOp {left: Box<ASTNode>, op: Token, right: Box<ASTNode>},
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
