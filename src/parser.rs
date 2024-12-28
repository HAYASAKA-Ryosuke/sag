use crate::environment::{Env, EnvVariableType, ValueType};
use crate::tokenizer::Token;
use fraction::Fraction;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(Fraction),
    String(String),
    Bool(bool),
    Void,
    List(Vec<Value>),
    Function,
    Return(Box<Value>),
    Lambda {
        arguments: Vec<ASTNode>,
        body: Box<ASTNode>,
        env: Env,
    },
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::Void => ValueType::Void,
            Value::List(_) => ValueType::List(Box::new(ValueType::Any)),
            Value::Function => ValueType::Function,
            Value::Return(value) => {
                if let Value::Void = **value {
                    ValueType::Void
                } else {
                    value.value_type()
                }
            },
            Value::Lambda { .. } => ValueType::Lambda,
        }
    }
    pub fn to_number(&self) -> Fraction {
        match self {
            Value::Number(value) => value.clone(),
            _ => panic!("expected number"),
        }
    }
    pub fn to_str(&self) -> String {
        match self {
            Value::String(value) => value.clone(),
            _ => panic!("expected string"),
        }
    }
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(value) => value.clone(),
            _ => panic!("expected bool"),
        }
    }
    pub fn to_list(&self) -> Vec<Value> {
        match self {
            Value::List(value) => value.clone(),
            _ => panic!("expected list"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{}", value),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Void => write!(f, "Void"),
            Value::Function => write!(f, "Function"),
            Value::Lambda { .. } => write!(f, "Lambda"),
            Value::Return(value) => write!(f, "{}", value),
            Value::List(list) => {
                let mut result = String::new();
                for (i, value) in list.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("{}", value));
                }
                write!(f, "[{}]", result)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    // 数値や文字列などのリテラル
    Literal(Value),
    // 変数
    Variable {
        name: String,
        value_type: Option<ValueType>,
    },
    Block(Vec<ASTNode>),
    // -5, !trueなどの一つのオペランドを持つ演算子
    PrefixOp {
        op: Token,
        expr: Box<ASTNode>,
    },
    // 1 + 2のような二項演算子
    BinaryOp {
        left: Box<ASTNode>,
        op: Token,
        right: Box<ASTNode>,
    },
    // 変数の代入
    Assign {
        name: String,
        value: Box<ASTNode>,
        variable_type: EnvVariableType,
        value_type: ValueType,
        is_new: bool,
    },
    Function {
        name: String,
        arguments: Vec<ASTNode>,
        body: Box<ASTNode>,
        return_type: ValueType,
    },
    FunctionCall {
        name: String,
        arguments: Box<ASTNode>,
    },
    FunctionCallArgs(Vec<ASTNode>),
    Return(Box<ASTNode>),
    Lambda {
        arguments: Vec<ASTNode>,
        body: Box<ASTNode>,
    },
    LambdaCall {
        lambda: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    Eq {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Gte{
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Gt {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Lte {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Lt {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    If {
        condition: Box<ASTNode>,
        then: Box<ASTNode>,
        else_: Option<Box<ASTNode>>,
        value_type: ValueType,
    },
}

pub struct Parser {
    tokens: Vec<Vec<Token>>,
    pos: usize,
    line: usize,
    scopes: Vec<String>,
    variables: HashMap<(String, String), (ValueType, EnvVariableType)>, // key: (scope, name), value: value_type
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let lines = Self::split_lines(tokens);
        Parser {
            tokens: lines.clone(),
            pos: 0,
            line: 0,
            scopes: vec!["global".into()],
            variables: HashMap::new(),
        }
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

    fn register_variables(
        &mut self,
        scope: String,
        name: &String,
        value_type: &ValueType,
        variable_type: &EnvVariableType,
    ) {
        self.variables.insert(
            (scope, name.to_string()),
            (value_type.clone(), variable_type.clone()),
        );
    }

    fn find_variables(
        &mut self,
        scope: String,
        name: String,
    ) -> Option<(ValueType, EnvVariableType)> {
        for checked_scope in vec![scope.to_string(), "global".to_string()] {
            match self
                .variables
                .get(&(checked_scope.to_string(), name.to_string()))
            {
                Some(value) => match &value.0 {
                    &ValueType::Number => return Some((ValueType::Number, value.1.clone())),
                    &ValueType::String => return Some((ValueType::String, value.1.clone())),
                    &ValueType::Bool => return Some((ValueType::Bool, value.1.clone())),
                    &ValueType::Function => return Some((ValueType::Function, value.1.clone())),
                    _ => return None,
                },
                None => {}
            };
        }
        None
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
        if self.line >= self.tokens.len() || self.pos >= self.tokens[self.line].len() {
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
            }
            _ => panic!("unexpected token: {:?}", token),
        }
    }

    fn infer_type(&self, ast: &ASTNode) -> Result<ValueType, String> {
        match ast {
            ASTNode::Literal(ref v) => match v {
                Value::Number(_) => Ok(ValueType::Number),
                Value::String(_) => Ok(ValueType::String),
                Value::Bool(_) => Ok(ValueType::Bool),
                _ => Ok(ValueType::Any),
            },
            ASTNode::Lambda { .. } => Ok(ValueType::Lambda),
            ASTNode::PrefixOp { op: _, expr } => {
                let value_type = self.infer_type(&expr)?;
                Ok(value_type)
            }
            ASTNode::BinaryOp { left, op, right } => {
                let left_type = self.infer_type(&left)?;
                let right_type = self.infer_type(&right)?;

                match (&left_type, &right_type) {
                    (ValueType::Number, ValueType::Number) => Ok(ValueType::Number),
                    (ValueType::Number, ValueType::String) => Ok(ValueType::String),
                    (ValueType::String, ValueType::Number) => Ok(ValueType::String),
                    (ValueType::Bool, ValueType::Bool) => Ok(ValueType::Bool),
                    _ => Err(
                        format!("type mismatch: {:?} {:?} {:?}", left_type, op, right_type).into(),
                    ),
                }
            }
            _ => Ok(ValueType::Any),
        }
    }

    fn parse_prefix_op(&mut self, op: Token) -> ASTNode {
        self.pos += 1;
        let value = self.parse_expression(std::u8::MAX);
        ASTNode::PrefixOp {
            op,
            expr: Box::new(value),
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

    fn is_lparen_call(&mut self) -> bool {
        self.pos += 1;
        let next_token = self.get_current_token();
        self.pos -= 1;
        next_token == Some(Token::LParen)
    }

    fn is_lambda_call(&mut self) -> bool {
        self.pos += 1;
        let next_token = self.get_current_token();
        self.pos -= 1;
        next_token == Some(Token::BackSlash)
    }

    fn parse_lambda_call(&mut self, left: ASTNode) -> ASTNode {
        self.consume_token();
        let lambda = self.parse_lambda();
        let arguments = match left {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => vec![left],
        };
        ASTNode::LambdaCall {
            lambda: Box::new(lambda),
            arguments,
        }
    }

    fn parse_function_call_front(&mut self, name: String, arguments: ASTNode) -> ASTNode {
        self.consume_token();

        let function_call = ASTNode::FunctionCall {
            name,
            arguments: Box::new(arguments),
        };
        function_call
    }

    fn parse_function_call(&mut self, left: ASTNode) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("failed take function name: {:?}", self.get_current_token()),
        };

        let arguments = ASTNode::FunctionCallArgs(match left {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => vec![left],
        });

        self.consume_token();

        let function_call = ASTNode::FunctionCall {
            name,
            arguments: Box::new(arguments),
        };
        function_call
    }

    fn parse_lambda(&mut self) -> ASTNode {
        self.consume_token();
        let mut arguments = vec![];

        self.enter_scope("lambda".to_string());
        match self.get_current_token() {
            Some(Token::Pipe) => {
                self.consume_token();
                while let Some(token) = self.get_current_token() {
                    if token == Token::Pipe {
                        self.consume_token();
                        break;
                    }
                    if self.get_current_token() == Some(Token::Comma) {
                        self.consume_token();
                        continue;
                    }
                    if let Token::Identifier(argument) = token {
                        self.consume_token();
                        self.extract_token(Token::Colon);
                        let value_type =
                            if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                                Some(self.string_to_value_type(type_name))
                            } else {
                                None
                            };
                        arguments.push(ASTNode::Variable {
                            name: argument.clone(),
                            value_type: value_type.clone(),
                        });
                        self.register_variables(
                            "lambda".to_string(),
                            &argument,
                            &value_type.unwrap(),
                            &EnvVariableType::Immutable,
                        );
                        self.consume_token();
                        continue;
                    }
                }
            }
            Some(Token::Identifier(argument)) => {
                self.consume_token();
                self.extract_token(Token::Colon);
                let value_type =
                    if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                        Some(self.string_to_value_type(type_name))
                    } else {
                        None
                    };
                arguments.push(ASTNode::Variable {
                    name: argument.clone(),
                    value_type,
                });
                self.consume_token();
            }
            _ => {}
        };

        self.extract_token(Token::RRocket);

        let result = match self.get_current_token() {
            Some(Token::LBrace) => {
                let statement = self.parse_block();
                ASTNode::Lambda {
                    arguments,
                    body: Box::new(statement),
                }
            }
            _ => {
                let statement = self.parse_expression(0);
                ASTNode::Lambda {
                    arguments,
                    body: Box::new(statement),
                }
            }
        };
        self.leave_scope();
        result
    }

    fn parse_function(&mut self) -> ASTNode {
        self.pos += 1;
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("undefined function name"),
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
            return_type,
        }
    }
    fn string_to_value_type(&mut self, type_name: String) -> ValueType {
        match type_name.as_str() {
            "number" => ValueType::Number,
            "string" => ValueType::String,
            "bool" => ValueType::Bool,
            _ => panic!("undefined type"),
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
                self.register_variables(
                    scope.to_string(),
                    &name,
                    &arg_type,
                    &EnvVariableType::Immutable,
                );
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
            _ => panic!("unexpected token"),
        };
        match self.consume_token() {
            Some(Token::Equal) => {
                let value = self.parse_expression(0);
                let value_type = match self.infer_type(&value) {
                    Ok(value_type) => value_type,
                    Err(e) => panic!("{}", e),
                };
                let variable_type = if mutable_or_immutable == Token::Mutable {
                    EnvVariableType::Mutable
                } else {
                    EnvVariableType::Immutable
                };
                self.register_variables(scope, &name, &value_type, &variable_type);
                ASTNode::Assign {
                    name,
                    value: Box::new(value),
                    variable_type,
                    value_type,
                    is_new: true,
                }
            }
            Some(Token::Colon) => {
                let value_type = match self.consume_token() {
                    Some(token) => match token {
                        Token::Identifier(value_type) => match value_type.as_str() {
                            "number" => ValueType::Number,
                            "str" => ValueType::String,
                            "bool" => ValueType::Bool,
                            _ => panic!("undefined type: {:?}", value_type),
                        },
                        _ => panic!("undefined type"),
                    },
                    _ => panic!("undefined type"),
                };
                match self.consume_token() {
                    Some(Token::Equal) => {
                        let value = self.parse_expression(0);
                        let variable_type = if mutable_or_immutable == Token::Mutable {
                            EnvVariableType::Mutable
                        } else {
                            EnvVariableType::Immutable
                        };
                        self.register_variables(scope, &name, &value_type, &variable_type);
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type,
                            value_type,
                            is_new: true,
                        }
                    }
                    _ => panic!("No valid statement found on the right-hand side"),
                }
            }
            _ => panic!("unexpected token"),
        }
    }

    fn parse_primary(&mut self) -> ASTNode {
        let token = match self.get_current_token() {
            Some(token) => token,
            _ => panic!("token not found!"),
        };
        match token {
            Token::Minus => self.parse_prefix_op(Token::Minus),
            Token::Return => self.parse_return(),
            Token::Number(value) => self.parse_literal(Value::Number(value)),
            Token::String(value) => self.parse_literal(Value::String(value.into())),
            Token::Function => self.parse_function(),
            Token::Pipe => self.parse_function_call_arguments(),
            Token::BackSlash => self.parse_lambda(),
            Token::Mutable | Token::Immutable => self.parse_assign(),
            Token::If => {
                let ast_if = self.parse_if();
                match ast_if {
                    ASTNode::If {
                        condition: _,
                        then: _,
                        ref else_,
                        ref value_type,
                    } => {
                        if *value_type != ValueType::Void {
                            if else_.is_none() {
                                panic!("if expressions without else");
                            }
                        }
                    }
                    _ => {}
                }
                ast_if
            },
            Token::LParen => {
                self.pos += 1;
                let expr = self.parse_expression(0);
                self.pos += 1;
                expr
            }
            Token::LBrace => self.parse_block(),
            Token::LBrancket => self.parse_list(),
            Token::Identifier(name) => {
                self.pos += 1;
                let scope = self.get_current_scope().to_string();
                let variable_info = self.find_variables(scope, name.clone());
                match self.get_current_token() {
                    Some(Token::LParen) => {
                        // 関数呼び出し
                        self.consume_token();
                        let arguments = self.parse_function_call_arguments_paren();
                        let function_call = self.parse_function_call_front(name, arguments);
                        function_call
                    }
                    Some(Token::Equal) => {
                        // 再代入
                        self.consume_token();
                        if variable_info.is_none() {
                            panic!("missing variable: {:?}", name);
                        }
                        let (value_type, variable_type) = variable_info.clone().unwrap();
                        if variable_type == EnvVariableType::Immutable {
                            panic!(
                                "It is an immutable variable and cannot be reassigned: {:?}",
                                name
                            );
                        }
                        let value = self.parse_expression(0);
                        ASTNode::Assign {
                            name,
                            value: Box::new(value),
                            variable_type,
                            value_type,
                            is_new: false,
                        }
                    }
                    Some(Token::Colon) => {
                        self.consume_token();
                        let value_type =
                            if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                                Some(self.string_to_value_type(type_name))
                            } else {
                                panic!("undefined type")
                            };
                        ASTNode::Variable { name, value_type }
                    }
                    _ => {
                        // 代入
                        let value_type = if variable_info.is_some() {
                            Some(variable_info.unwrap().0)
                        } else {
                            None
                        };
                        ASTNode::Variable { name, value_type }
                    }
                }
            }
            _ => panic!("undefined token: {:?}", token),
        }
    }

    fn parse_eq(&mut self, left: ASTNode) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Eq) => self.consume_token(),
            _ => panic!("unexpected token"),
        };

        let right = self.parse_expression(0);

        ASTNode::Eq {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_gte(&mut self, left: ASTNode) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Gte) => self.consume_token(),
            _ => panic!("unexpected token"),
        };

        let right = self.parse_expression(0);

        ASTNode::Gte {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_gt(&mut self, left: ASTNode) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Gt) => self.consume_token(),
            _ => panic!("unexpected token"),
        };

        let right = self.parse_expression(0);

        ASTNode::Gt {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_lte(&mut self, left: ASTNode) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Lte) => self.consume_token(),
            _ => panic!("unexpected token"),
        };

        let right = self.parse_expression(0);

        ASTNode::Lte {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_lt(&mut self, left: ASTNode) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Lt) => self.consume_token(),
            _ => panic!("unexpected token"),
        };

        let right = self.parse_expression(0);

        ASTNode::Lt {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_if(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::If) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let condition = self.parse_expression(0);
        let then = self.parse_expression(0);
        match then {
            ASTNode::Block(_) => {
                self.pos = 0;
                self.line += 1;
            }
            _ => {}
        };
        let else_ = match self.get_current_token() {
            Some(Token::Else) => {
                self.consume_token();
                if self.get_current_token() == Some(Token::If) {
                    Some(Box::new(self.parse_if()))
                } else {
                    Some(Box::new(self.parse_expression(0)))
                }
            }
            _ => None,
        };

        let value_type = {
            let mut then_type = None;
            let mut else_type = None;

            match then {
                ASTNode::Return(ref value) => {
                    then_type = Some(self.infer_type(&value));
                },
                ASTNode::Block(ref statements) => {
                    for statement in statements {
                        if let ASTNode::Return(ref value) = statement {
                            then_type = Some(self.infer_type(&value));
                        }
                    }
                }
                _ => {}
            }

            if let Some(else_node) = &else_ {
                match &**else_node {
                    ASTNode::Return(ref value) => {
                        else_type = Some(self.infer_type(&value));
                    },
                    ASTNode::Block(ref statements) => {
                        for statement in statements {
                            if let ASTNode::Return(ref value) = statement { 
                                else_type = Some(self.infer_type(&value));
                            }
                        }
                    }
                    _ => {}
                }
            }

            match (then_type, else_type) {
                (Some(t), Some(e)) if t == e => t,
                (Some(t), None) => t,
                (None, Some(e)) => e,
                (None, None) => Ok(ValueType::Void),
                _ => Err("Type mismatch in if statement".to_string())
            }
        };
        if value_type.is_err() {
            panic!("{}", value_type.err().unwrap());
        }

        ASTNode::If {
            condition: Box::new(condition),
            then: Box::new(then),
            else_,
            value_type: value_type.unwrap(),
        }
    }

    fn parse_function_call_arguments(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::Pipe) => self.consume_token(),
            _ => None,
        };
        let mut arguments = vec![];
        while let Some(token) = self.get_current_token() {
            if token == Token::Comma {
                self.pos += 1;
                continue;
            }
            if token == Token::Pipe {
                self.pos += 1;
                break;
            }
            let value = self.parse_expression(0);
            arguments.push(value);
        }
        ASTNode::FunctionCallArgs(arguments)
    }

    fn parse_function_call_arguments_paren(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::LParen) => self.consume_token(),
            _ => None,
        };
        let mut arguments = vec![];
        while let Some(token) = self.get_current_token() {
            if token == Token::Comma {
                self.pos += 1;
                continue;
            }
            if token == Token::RParen {
                self.pos += 1;
                break;
            }
            let value = self.parse_expression(0);
            arguments.push(value);
        }
        ASTNode::FunctionCallArgs(arguments)
    }

    fn parse_list(&mut self) -> ASTNode {
        self.consume_token();
        let mut list = vec![];
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrancket {
                break;
            }
            if token == Token::Comma {
                self.consume_token();
                continue;
            }
            let value = match token {
                Token::Number(value) => Value::Number(value),
                Token::String(value) => Value::String(value),
                _ => panic!("unexpected token: {:?}", token),
            };
            list.push(ASTNode::Literal(value));
            self.consume_token();
        }
        ASTNode::Literal(Value::List(
            list.iter()
                .map(|x| match x {
                    ASTNode::Literal(value) => value.clone(),
                    _ => panic!("unexpected node"),
                })
                .collect(),
        ))
    }

    fn parse_block(&mut self) -> ASTNode {
        let mut statements = Vec::new();
        if self.get_current_token() != Some(Token::LBrace) {
            panic!("Expected LBrace at the start of a block");
        }
        self.consume_token();

        while let Some(token) = self.get_current_token() {
            if token == Token::RBrace {
                self.consume_token();
                break;
            }
            match self.get_current_token() {
                Some(Token::Eof) => {
                    self.pos = 0;
                    self.line += 1;
                    continue;
                }
                _ => {}
            };
            let statement = self.parse_expression(0);
            statements.push(statement);
        }
        ASTNode::Block(statements)
    }

    fn parse_expression(&mut self, min_priority: u8) -> ASTNode {
        let mut lhs = self.parse_primary();
        loop {
            let token = match self.get_current_token() {
                Some(token) => token,
                _ => break,
            };
            if token == Token::Eq {
                lhs = self.parse_eq(lhs);
                continue;
            }
            if token == Token::Gte {
                lhs = self.parse_gte(lhs);
                continue;
            }
            if token == Token::Gt {
                lhs = self.parse_gt(lhs);
                continue;
            }
            if token == Token::Lte {
                lhs = self.parse_lte(lhs);
                continue;
            }
            if token == Token::Lt {
                lhs = self.parse_lt(lhs);
                continue;
            }
            if token == Token::RArrow {
                if self.is_lparen_call() {
                    self.pos += 1;
                    let rhs = self.parse_primary();
                    lhs = ASTNode::LambdaCall {
                        lambda: Box::new(rhs),
                        arguments: vec![lhs],
                    };
                    continue;
                }
                if self.is_lambda_call() {
                    lhs = self.parse_lambda_call(lhs);
                } else {
                    lhs = self.parse_function_call(lhs);
                }
                continue;
            }

            // 二項演算
            if let Some((left_priority, right_priority)) = self.get_priority(&token) {
                if left_priority < min_priority {
                    break;
                }
                self.pos += 1;

                let rhs = self.parse_expression(right_priority);
                lhs = ASTNode::BinaryOp {
                    left: Box::new(lhs),
                    op: token,
                    right: Box::new(rhs),
                }
            } else {
                break;
            }
        }
        lhs
    }
    fn get_priority(&self, token: &Token) -> Option<(u8, u8)> {
        match token {
            Token::Plus | Token::Minus => Some((1, 2)),
            Token::Mul | Token::Div => Some((3, 4)),
            _ => None,
        }
    }

    pub fn parse(&mut self) -> ASTNode {
        self.parse_expression(0)
    }

    pub fn parse_lines(&mut self) -> Vec<ASTNode> {
        let mut ast_nodes = vec![];
        for _ in 0..self.tokens.len() {
            ast_nodes.push(self.parse());
            self.line += 1;
            if self.line >= self.tokens.len() {
                break;
            }
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
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::Number(Fraction::from(1)),
            Token::Plus,
            Token::Number(Fraction::from(2)),
            Token::Mul,
            Token::Number(Fraction::from(3)),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::BinaryOp {
                left: Box::new(ASTNode::PrefixOp {
                    op: Token::Minus,
                    expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }),
                op: Token::Plus,
                right: Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    op: Token::Mul,
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3))))
                })
            }
        );
    }

    #[test]
    fn test_type_specified() {
        let mut parser = Parser::new(vec![
            Token::Mutable,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Equal,
            Token::Number(Fraction::from(1)),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Assign {
                name: "x".into(),
                value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                variable_type: EnvVariableType::Mutable,
                value_type: ValueType::Number,
                is_new: true
            }
        );
    }

    #[test]
    fn test_type_estimate() {
        let mut parser = Parser::new(vec![
            Token::Mutable,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(1)),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Assign {
                name: "x".into(),
                value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                variable_type: EnvVariableType::Mutable,
                value_type: ValueType::Number,
                is_new: true
            }
        );
    }

    #[test]
    fn test_register_function() {
        let mut parser = Parser::new(vec![
            Token::Function,
            Token::Identifier("foo".into()),
            Token::Equal,
            Token::LParen,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Comma,
            Token::Identifier("y".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::RParen,
            Token::Colon,
            Token::Identifier("number".into()),
            Token::LBrace,
            Token::Return,
            Token::Identifier("x".into()),
            Token::Plus,
            Token::Identifier("y".into()),
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Function {
                name: "foo".into(),
                arguments: vec![
                    ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number)
                    },
                    ASTNode::Variable {
                        name: "y".into(),
                        value_type: Some(ValueType::Number)
                    }
                ],
                body: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(
                    ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number)
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Variable {
                            name: "y".into(),
                            value_type: Some(ValueType::Number)
                        }),
                    }
                ))])),
                return_type: ValueType::Number
            }
        )
    }

    #[test]
    fn test_block() {
        let mut parser = Parser::new(vec![
            Token::LBrace,
            Token::Identifier("x".into()),
            Token::Plus,
            Token::Identifier("y".into()),
            Token::Eof,
            Token::Return,
            Token::Number(Fraction::from(1)),
            Token::Minus,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse_block(),
            ASTNode::Block(vec![
                ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: None
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "y".into(),
                        value_type: None
                    })
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                    op: Token::Minus,
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }))
            ])
        );
    }

    #[test]
    fn test_reassign_to_mutable_variable() {
        let mut parser = Parser::new(vec![
            Token::Mutable,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(2)),
            Token::Eof,
        ]);

        let expected_ast = vec![
            ASTNode::Assign {
                name: "x".into(),
                value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                variable_type: EnvVariableType::Mutable,
                value_type: ValueType::Number,
                is_new: true, // 新規定義
            },
            ASTNode::Assign {
                name: "x".into(),
                value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                variable_type: EnvVariableType::Mutable,
                value_type: ValueType::Number,
                is_new: false, // 再代入
            },
        ];

        assert_eq!(parser.parse_lines(), expected_ast);
    }

    #[test]
    fn test_function_call() {
        let mut parser = Parser::new(vec![
            Token::Pipe,
            Token::Identifier("x".into()),
            Token::Comma,
            Token::Identifier("y".into()),
            Token::Comma,
            Token::Number(Fraction::from(1)),
            Token::Pipe,
            Token::RArrow,
            Token::Identifier("f1".into()),
            Token::Eof,
        ]);

        assert_eq!(
            parser.parse(),
            ASTNode::FunctionCall {
                name: "f1".into(),
                arguments: Box::new(ASTNode::FunctionCallArgs(
                    [
                        ASTNode::Variable {
                            name: "x".into(),
                            value_type: None,
                        },
                        ASTNode::Variable {
                            name: "y".into(),
                            value_type: None,
                        },
                        ASTNode::Literal(Value::Number(Fraction::from(1))),
                    ]
                    .to_vec()
                )),
            }
        );
    }
    #[test]
    #[should_panic(expected = "It is an immutable variable and cannot be reassigned")]
    fn test_reassign_to_immutable_variable_should_panic() {
        let mut parser = Parser::new(vec![
            Token::Immutable,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(10)),
            Token::Eof,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(20)),
            Token::Eof,
        ]);
        // 最初のparseで変数定義
        let _ = parser.parse();
        parser.line += 1;
        parser.pos = 0;
        // 2回目のparseで不変変数の再代入を試みてパニックになる
        let _ = parser.parse();
    }
    #[test]
    fn test_function_without_arguments_and_void_return() {
        let mut parser = Parser::new(vec![
            Token::Function,
            Token::Identifier("no_args".into()),
            Token::Equal,
            Token::LParen,
            Token::RParen, // 引数なし
            // 戻り値の型指定なし → void
            Token::LBrace,
            Token::Return,
            Token::Number(Fraction::from(42)),
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Function {
                name: "no_args".into(),
                arguments: vec![],
                body: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(
                    ASTNode::Literal(Value::Number(Fraction::from(42)))
                ))])),
                return_type: ValueType::Void
            }
        )
    }
    #[test]
    fn test_function_call_with_no_arguments() {
        let mut parser = Parser::new(vec![
            Token::Pipe,
            Token::Pipe,
            Token::RArrow,
            Token::Identifier("func".into()),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::FunctionCall {
                name: "func".into(),
                arguments: Box::new(ASTNode::FunctionCallArgs(vec![]))
            }
        );
    }

    #[test]
    fn test_nested_block_scope() {
        let mut parser = Parser::new(vec![
            Token::LBrace,
            Token::Mutable,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(10)),
            Token::Eof,
            Token::LBrace,
            Token::Immutable,
            Token::Identifier("y".into()),
            Token::Equal,
            Token::Number(Fraction::from(20)),
            Token::Eof,
            Token::RBrace,
            Token::Return,
            Token::Identifier("x".into()),
            Token::Plus,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse_block(),
            ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "x".into(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(10)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: true,
                },
                ASTNode::Block(vec![ASTNode::Assign {
                    name: "y".into(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(20)))),
                    variable_type: EnvVariableType::Immutable,
                    value_type: ValueType::Number,
                    is_new: true,
                }]),
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number)
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }))
            ])
        );
    }

    #[test]
    fn test_prefix_operator_only() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::Number(Fraction::from(5)),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::PrefixOp {
                op: Token::Minus,
                expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5))))
            }
        )
    }

    #[test]
    fn test_list() {
        let mut parser = Parser::new(vec![
            Token::LBrancket,
            Token::Number(Fraction::from(1)),
            Token::Comma,
            Token::Number(Fraction::from(2)),
            Token::Comma,
            Token::Number(Fraction::from(3)),
            Token::RBrancket,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Literal(Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3))
            ]))
        );
    }

    #[test]
    fn test_fraction_and_decimal_operations() {
        // 小数のテスト
        let mut parser = Parser::new(vec![
            Token::Number(Fraction::new(5u64, 2u64)), // 2.5
            Token::Plus,
            Token::Number(Fraction::new(3u64, 2u64)), // 1.5
            Token::Eof,
        ]);

        assert_eq!(
            parser.parse(),
            ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::new(5u64, 2u64)))),
                op: Token::Plus,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::new(3u64, 2u64))))
            }
        );

        // 分数の演算テスト
        let mut parser = Parser::new(vec![
            Token::Number(Fraction::new(1u64, 3u64)), // 1/3
            Token::Mul,
            Token::Number(Fraction::new(2u64, 5u64)), // 2/5
            Token::Eof,
        ]);

        assert_eq!(
            parser.parse(),
            ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::new(1u64, 3u64)))),
                op: Token::Mul,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::new(2u64, 5u64))))
            }
        );
    }

    #[test]
    fn test_function_call_chain() {
        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(1)),
            Token::RArrow,
            Token::Identifier("f1".into()),
            Token::RArrow,
            Token::Identifier("f2".into()),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::FunctionCall {
                name: "f2".into(),
                arguments: Box::new(ASTNode::FunctionCallArgs(vec![ASTNode::FunctionCall {
                    name: "f1".into(),
                    arguments: Box::new(ASTNode::FunctionCallArgs(vec![ASTNode::Literal(
                        Value::Number(Fraction::from(1))
                    )]))
                }]))
            }
        );
    }

    #[test]
    fn test_lambda() {
        let mut parser = Parser::new(vec![
            Token::Immutable,
            Token::Identifier("inc".into()),
            Token::Equal,
            Token::BackSlash,
            Token::Pipe,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Pipe,
            Token::RRocket,
            Token::Identifier("x".into()),
            Token::Plus,
            Token::Number(Fraction::from(1)),
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::Assign {
                name: "inc".into(),
                variable_type: EnvVariableType::Immutable,
                is_new: true,
                value_type: ValueType::Lambda,
                value: Box::new(ASTNode::Lambda {
                    arguments: vec![ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number)
                    }],
                    body: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number)
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                    })
                }),
            }
        );
    }

    #[test]
    fn test_if() {
        let mut parser = Parser::new(vec![
            Token::If,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::LBrace,
            Token::Eof,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::If {
                condition: Box::new(ASTNode::Eq {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: None
                    }),
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }),
                then: Box::new(ASTNode::Block(vec![
                    ASTNode::Literal(Value::Number(Fraction::from(1)))
                ])),
                else_: None,
                value_type: ValueType::Void
            }
        );
    }

    #[test]
    #[should_panic(expected = "if expressions without else")]
    fn test_partial_return_if() {
        let mut parser = Parser::new(vec![
            Token::If,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::LBrace,
            Token::Eof,
            Token::Return,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::If {
                condition: Box::new(ASTNode::Eq {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: None
                    }),
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }),
                then: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(
                    ASTNode::Literal(Value::Number(Fraction::from(1)))
                ))])),
                else_: None,
                value_type: ValueType::Number
            }
        );
    }

    #[test]
    fn test_non_return_if() {
        let mut parser = Parser::new(vec![
            Token::If,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::LBrace,
            Token::Eof,
            Token::Number(Fraction::from(1)),
            Token::RBrace,
            Token::Eof,
        ]);
        assert_eq!(
            parser.parse(),
            ASTNode::If {
                condition: Box::new(ASTNode::Eq {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: None
                    }),
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
                }),
                then: Box::new(ASTNode::Block(vec![ASTNode::Literal(Value::Number(Fraction::from(1)))])),
                else_: None,
                value_type: ValueType::Void
            }
        );
    }

    #[test]
    fn test_else() {
        let tokens = vec![
            Token::If,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::LBrace,
            Token::Eof,
            Token::Return,
            Token::Number(Fraction::from(1)),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
            Token::Else,
            Token::LBrace,
            Token::Eof,
            Token::Return,
            Token::Number(Fraction::from(0)),
            Token::Eof,
            Token::RBrace,
            Token::Eof
        ];
        let mut parser = Parser::new(tokens);
        let condition = Box::new(ASTNode::Eq {
            left: Box::new(ASTNode::Variable {
                name: "x".into(),
                value_type: None
            }),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        });
        let then = Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))))]));
        let else_ = Some(Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(0)))))])));
        assert_eq!(
            parser.parse_lines(),
            vec![
            ASTNode::If {
                condition,
                then,
                else_,
                value_type: ValueType::Number
            }
            ]
        );
    }
    #[test]
    fn test_else_if() {
        let tokens = vec![
           Token::If,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(1)),
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(1)),
           Token::Eof,
           Token::RBrace,
           Token::Eof,
           Token::Else,
           Token::If,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(2)),
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(2)),
           Token::Eof,
           Token::RBrace,
           Token::Eof,
           Token::Else,
           Token::If,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(3)),
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(3)),
           Token::Eof,
           Token::RBrace,
           Token::Eof,
           Token::Else,
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(0)),
           Token::Eof,
           Token::RBrace,
           Token::Eof
        ];
        let mut parser = Parser::new(tokens);
        let condition = Box::new(ASTNode::Eq {
            left: Box::new(ASTNode::Variable {
                name: "x".into(),
                value_type: None
            }),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        });
        let then = Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))))]));
        let else_ = Some(Box::new(ASTNode::If {
            condition: Box::new(ASTNode::Eq {
                left: Box::new(ASTNode::Variable {
                    name: "x".into(),
                    value_type: None
                }),
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))
                )
            }),
            then: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))))])),
            else_: Some(Box::new(ASTNode::If {
                condition: Box::new(ASTNode::Eq {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: None
                    }),
                    right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))
                    )
                }),
                then: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))))])),
                else_: Some(Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(Fraction::from(0)))))]))),
                value_type: ValueType::Number
            })),
            value_type: ValueType::Number
        }));
        assert_eq!(
            parser.parse(),
            ASTNode::If {
                condition,
                then,
                else_,
                value_type: ValueType::Number
            }
        );
    }

    #[test]
    fn test_comparison_operations() {
        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(1)),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::Eof
        ]);
        assert_eq!(parser.parse(), ASTNode::Eq {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        });
        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(2)),
            Token::Gt,
            Token::Number(Fraction::from(1)),
            Token::Eof
        ]);
        assert_eq!(parser.parse(), ASTNode::Gt {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        });

        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(3)),
            Token::Gte,
            Token::Number(Fraction::from(3)),
            Token::Eof
        ]);
        assert_eq!(parser.parse(), ASTNode::Gte {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3))))
        });

        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(1)),
            Token::Lt,
            Token::Number(Fraction::from(2)),
            Token::Eof
        ]);
        assert_eq!(parser.parse(), ASTNode::Lt {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2))))
        });

        let mut parser = Parser::new(vec![
            Token::Number(Fraction::from(4)),
            Token::Lte,
            Token::Number(Fraction::from(4)),
            Token::Eof
        ]);
        assert_eq!(parser.parse(), ASTNode::Lte {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4))))
        });
    }
}
