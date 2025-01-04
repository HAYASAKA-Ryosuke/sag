use crate::environment::{EnvVariableType, ValueType};
use crate::token::Token;
use crate::ast::ASTNode;
use crate::value::Value;
use std::collections::HashMap;


pub struct Parser {
    tokens: Vec<Vec<Token>>,
    pos: usize,
    line: usize,
    scopes: Vec<String>,
    variables: HashMap<(String, String), (ValueType, EnvVariableType)>, // key: (scope, name), value: value_type
    structs: HashMap<(String, String), (ValueType, EnvVariableType)>, // key: (scope, name), value: value_type
    functions: HashMap<(String, String), ValueType>, // key: (scope, name, arguments), value: (body, return_type)
    current_struct: Option<String>,
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
            structs: HashMap::new(),
            functions: HashMap::new(),
            current_struct: None,
        }
    }

    fn enter_scope(&mut self, scope_name: String) {
        self.scopes.push(scope_name);
    }
    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_current_scope(&self) -> String {
        self.scopes.last().unwrap().to_string()
    }

    fn register_struct(&mut self, scope: String, struct_value: ASTNode) {
        if let ASTNode::Struct { name, fields, ref is_public } = &struct_value {
            let field_types = fields.iter().map(|(name, field)| {
                if let ASTNode::StructField { value_type, is_public } = field {
                    (name.clone(), ValueType::StructField { value_type: Box::new(value_type.clone()), is_public: is_public.clone() })
                } else {
                    panic!("invalid struct field")
                }
            }).collect();
            let insert_value = (ValueType::Struct { name: name.clone(), fields: field_types, is_public: is_public.clone() }, EnvVariableType::Immutable);
            self.structs.insert(
                (scope.to_string(), name.to_string()),
                insert_value,
            );
        }
    }

    fn get_struct(&mut self, scope: String, name: String) -> Option<ValueType> {
        for checked_scope in vec![scope.to_string(), "global".to_string()] {
            match self.structs.get(&(checked_scope.to_string(), name.to_string())) {
                Some((ref value_type, _)) => match value_type.clone() {
                    ValueType::Struct { name, fields, is_public } => return Some(ValueType::Struct { name: name.clone(), fields: fields.clone(), is_public: is_public.clone() }),
                    _ => return None,
                },
                None => {}
            };
        }
        None
    }

    fn register_functions(
        &mut self,
        scope: String,
        name: &String,
        arguments: &Vec<ASTNode>,  // arugmentsも多重定義を許容するときに使う
        return_type: &ValueType,
    ) {
        self.functions.insert(
            (scope.clone(), name.to_string()),
            return_type.clone(),
        );
    }

    fn get_function(&self, scope: String, name: String) -> Option<ValueType> {
        for checked_scope in vec![scope.to_string(), "global".to_string()] {
            match self.functions.get(&(checked_scope.to_string(), name.to_string())) {
                Some(value) => return Some(value.clone()),
                None => {}
            };
        }
        None
    }

    fn register_variables(
        &mut self,
        scope: String,
        name: &String,
        value_type: &ValueType,
        variable_type: &EnvVariableType,
    ) {
        self.variables.insert(
            (scope.clone(), name.to_string()),
            (value_type.clone(), variable_type.clone()),
        );
    }

    fn find_variables(
        &self,
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
                    &ValueType::StructInstance{ref name, ref fields} => {
                        return Some((ValueType::StructInstance{name: name.to_string(), fields: fields.clone()}, value.1.clone()))
                    },
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
                Value::Void => Ok(ValueType::Void),
                Value::Struct { name, fields, is_public, methods: _ } => {
                    let field_types = fields.iter().map(|(name, field)| {
                        if let Value::StructField { value_type, is_public: _ } = field {
                            (name.clone(), value_type.clone())
                        } else {
                            panic!("invalid struct field")
                        }
                    }).collect::<HashMap<_,_>>();
                    Ok(ValueType::Struct { name: name.clone(), fields: field_types.clone(), is_public: is_public.clone() })
                },
                Value::List(values) => {
                    let mut value_type = ValueType::Any;
                    for value in values {
                        let value_type_ = self.infer_type(&ASTNode::Literal(value.clone()))?;
                        if value_type == ValueType::Any {
                            value_type = value_type_;
                        } else if value_type != value_type_ {
                            return Err("type mismatch in list".to_string());
                        }
                    }
                    Ok(value_type)
                },
                _ => Ok(ValueType::Any),
            },
            ASTNode::Lambda { .. } => Ok(ValueType::Lambda),
            ASTNode::PrefixOp { op: _, expr } => {
                let value_type = self.infer_type(&expr)?;
                Ok(value_type)
            }

            ASTNode::StructInstance { name, fields } => {
                let mut field_types = HashMap::new();
                for (field_name, field_value) in fields.iter() {
                    field_types.insert(field_name.clone(), self.infer_type(field_value)?);
                }
                Ok(ValueType::StructInstance {
                    name: name.clone(),
                    fields: field_types,
                })
            }
            ASTNode::FunctionCall { name, arguments: _ } => {
                let function = self.get_function(self.get_current_scope(), name.clone());
                if function.is_none() {
                    return Err(format!("undefined function: {:?}", name));
                }
                let value_type = function.unwrap();
                Ok(value_type.clone())
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
            },
            ASTNode::Variable { name, value_type } => {
                if let Some(value_type) = value_type {
                    Ok(value_type.clone())
                } else {
                    let scope = self.get_current_scope();
                    match self.find_variables(scope, name.clone()) {
                        Some((value_type, _)) => Ok(value_type.clone()),
                        None => Err(format!("undefined variable: {:?}", name).into()),
                    }
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
        let function_scope = self.get_current_scope();
        self.enter_scope(name.to_string());
        self.pos += 1;
        self.extract_token(Token::LParen);

        let arguments = self.parse_function_arguments();
        let return_type = self.parse_return_type();
        self.register_functions(
            function_scope,
            &name,
            &arguments,
            &return_type,
        );
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
        let scope = self.get_current_scope();
        if let Some(struct_value) = self.get_struct(scope, type_name.clone()) {
            return struct_value;
        }

        match type_name.as_str() {
            "number" => ValueType::Number,
            "string" => ValueType::String,
            "bool" => ValueType::Bool,
            "void" => ValueType::Void,
            _ => panic!("undefined type: {:?}", type_name),
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

                self.register_variables(scope.clone(), &name, &value_type, &variable_type);
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
            Token::PrivateStruct => self.parse_struct(false),
            Token::PublicStruct => self.parse_struct(true),
            Token::Impl => self.parse_impl(),
            Token::Minus => self.parse_prefix_op(Token::Minus),
            Token::Return => self.parse_return(),
            Token::Number(value) => self.parse_literal(Value::Number(value)),
            Token::String(value) => self.parse_literal(Value::String(value.into())),
            Token::Function => self.parse_function(),
            Token::Pipe => self.parse_function_call_arguments(),
            Token::BackSlash => self.parse_lambda(),
            Token::Mutable | Token::Immutable => self.parse_assign(),
            Token::For => self.parse_for(),
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
                    Some(Token::LBrace) => {
                        // 構造体のインスタンス化
                        self.consume_token();
                        let mut fields = HashMap::new();
                        while let Some(token) = self.get_current_token() {
                            if token == Token::RBrace {
                                self.consume_token();
                                break;
                            }
                            if token == Token::Comma {
                                self.consume_token();
                                continue;
                            }
                            if let Token::Identifier(field_name) = token {
                                self.consume_token();
                                self.extract_token(Token::Colon);
                                let value = self.parse_expression(0);
                                fields.insert(field_name, value);
                                continue;
                            }
                        }
                        ASTNode::StructInstance { name, fields }
                    }
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
                    Some(Token::Dot) => {
                        self.pos += 2;
                        if Some(Token::LParen) == self.get_current_token() {
                            self.pos -= 1;
                            let method_name = match self.get_current_token() {
                                Some(Token::Identifier(method_name)) => method_name,
                                _ => panic!("missing method name: {:?}", self.get_current_token())
                            };
                            self.pos += 1;
                            let arguments = self.parse_function_call_arguments_paren();
                            return self.parse_method_call(name.to_string(), method_name.to_string(), arguments);
                        }
                        self.pos -= 2;
                        
                        // 構造体のフィールドアクセス
                        let struct_instance_access = self.parse_struct_instance_access(name.clone());
                        // 代入
                        if let Some(Token::Equal) = self.get_current_token() {
                            self.consume_token();
                            let value = self.parse_expression(0);
                            let field_name = match struct_instance_access.clone() {
                                ASTNode::StructFieldAccess { field_name, .. } => field_name,
                                _ => panic!("unexpected token"),
                            };
                            ASTNode::StructFieldAssign {
                                instance: Box::new(struct_instance_access),
                                field_name: field_name.clone(),
                                value: Box::new(value),
                            }
                        } else {
                            struct_instance_access
                        }
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
            Token::CommentBlock(comment) => {ASTNode::CommentBlock(comment.to_string())},
            _ => panic!("undefined token: {:?}", token),
        }
    }

    fn parse_impl(&mut self) -> ASTNode {
        self.consume_token();
        let scope = self.get_current_scope().clone();
        let struct_name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        let base_struct = self.get_struct(scope.clone(),struct_name.to_string()).expect("undefined struct");
        self.current_struct = Some(struct_name.clone());
        self.consume_token();
        self.extract_token(Token::LBrace);
        let mut methods = Vec::new();
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrace {
                self.consume_token();
                break;
            }
            if token == Token::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            if token == Token::Comma {
                self.consume_token();
                continue;
            }
            if token == Token::Function {
                let method = self.parse_method();
                methods.push(method);
                continue;
            }
        }
        self.current_struct = None;
        ASTNode::Impl {
            base_struct: Box::new(base_struct),
            methods,
        }
    }

    fn parse_method(&mut self) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        self.enter_scope(name.to_string());
        self.consume_token();
        self.extract_token(Token::LParen);
        let self_type = ValueType::StructInstance {
            name: self.get_current_scope(),
            fields: HashMap::new(), // 仮のフィールド
        };
        let scope = self.get_current_scope();
        self.register_variables(
            scope.clone(),
            &"self".to_string(),
            &self_type,
            &EnvVariableType::Immutable,
        );
        let mut arguments = self.parse_function_arguments();
        arguments.insert(
            0,
            ASTNode::Variable {
                name: "self".to_string(),
                value_type: None,
            },
        );

        let return_type = self.parse_return_type();
        let body = self.parse_block();
        self.leave_scope();
        ASTNode::Method {
            name,
            arguments,
            body: Box::new(body),
            return_type,
        }
    }

    fn parse_struct_instance_access(&mut self, name: String) -> ASTNode {
        self.consume_token();
        let field_name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        self.consume_token();
        let scope = self.get_current_scope().clone();
        if name == "self" {
            if self.current_struct.is_none() {
                panic!("undefined struct for self");
            }
            let current_struct = self.current_struct.clone().unwrap();
            let struct_type = self
                .get_struct(scope.clone(), current_struct.to_string())
                .expect("undefined struct for self");
    
            return ASTNode::StructFieldAccess {
                instance: Box::new(ASTNode::Variable {
                    name: "self".to_string(),
                    value_type: Some(struct_type.clone()),
                }),
                field_name,
            };
        }

        match self.find_variables(scope.clone(), name.clone()) {
            Some((ValueType::StructInstance { name: instance_name, ref fields }, _)) => {
                ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable { name: name.clone(), value_type: Some(ValueType::StructInstance {name: instance_name, fields: fields.clone()}) }),
                    field_name,
                }
            }
            _ => panic!("undefined struct: {:?}", name),
        }
    }

    fn parse_method_call(&mut self, caller: String, method_name: String, arguments: ASTNode) -> ASTNode {
        self.consume_token();
        ASTNode::MethodCall {
            method_name,
            caller,
            arguments: Box::new(arguments),
        }
    }

    fn parse_struct(&mut self, is_public: bool) -> ASTNode {
        self.consume_token();
        let name = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        if name[0..1] != name[0..1].to_uppercase() {
            panic!("struct name must start with a capital letter");
        }
        self.consume_token();
        self.extract_token(Token::LBrace);
        let mut fields = HashMap::new();
        let mut field_is_public = false;
        while let Some(token) = self.get_current_token() {
            if token == Token::RBrace {
                self.consume_token();
                break;
            }
            if token == Token::Comma {
                self.consume_token();
                continue;
            }
            if token == Token::Eof {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            if token == Token::Pub {
                field_is_public = true;
                self.consume_token();
                continue;
            }

            if let Token::Identifier(name) = token {
                self.consume_token();
                self.extract_token(Token::Colon);
                let value_type = match self.get_current_token() {
                    Some(Token::Identifier(type_name)) => self.string_to_value_type(type_name),
                    _ => panic!("undefined type"),
                };
                fields.insert(name, ASTNode::StructField {
                    value_type,
                    is_public: field_is_public,
                });
                self.consume_token();
                field_is_public = false;
                continue;
            }
        }
        let result = ASTNode::Struct { name, fields, is_public };
        let scope = self.get_current_scope().clone();
        self.register_struct(scope, result.clone());
        result
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

    fn parse_for(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::For) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let variable = match self.get_current_token() {
            Some(Token::Identifier(name)) => name,
            _ => panic!("unexpected token"),
        };
        self.consume_token();
        self.extract_token(Token::In);
        let iterable = self.parse_expression(0);
        let body = self.parse_expression(0);
        ASTNode::For {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
        }
    }

    fn parse_if(&mut self) -> ASTNode {
        match self.get_current_token() {
            Some(Token::If) => self.consume_token(),
            _ => panic!("unexpected token"),
        };
        let condition = match self.get_current_token() {
            Some(Token::LParen) => self.parse_expression(0),
            _ => panic!("unexpected token missing ("),
        };
        let then = self.parse_expression(0);
        if self.get_current_token() == Some(Token::Eof) {
            self.pos = 0;
            self.line += 1;
        }

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
                self.consume_token();
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

        loop {
            let token = self.get_current_token();
            if token == Some(Token::RBrace) {
                self.consume_token();
                break;
            }
            if token == Some(Token::Eof) {
                self.pos = 0;
                self.line += 1;
                continue;
            }
            if token == None {
                if self.line >= self.tokens.len() {
                    break;
                }
                self.pos = 0;
                self.line += 1;
                continue;
            }
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
    use fraction::Fraction;

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
            Token::LParen,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::RParen,
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
            Token::LParen,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::RParen,
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
            Token::LParen,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::RParen,
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
            Token::LParen,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Number(Fraction::from(1)),
            Token::RParen,
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
           Token::LParen,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(1)),
           Token::RParen,
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(1)),
           Token::Eof,
           Token::RBrace,
           Token::Eof,
           Token::Else,
           Token::If,
           Token::LParen,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(2)),
           Token::RParen,
           Token::LBrace,
           Token::Eof,
           Token::Return,
           Token::Number(Fraction::from(2)),
           Token::Eof,
           Token::RBrace,
           Token::Eof,
           Token::Else,
           Token::If,
           Token::LParen,
           Token::Identifier("x".into()),
           Token::Eq,
           Token::Number(Fraction::from(3)),
           Token::RParen,
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

    #[test]
    fn test_struct() {
        let tokens = vec![
            Token::PrivateStruct,
            Token::Identifier("Point".into()),
            Token::LBrace,
            Token::Eof,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Comma,
            Token::Eof,
            Token::Identifier("y".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Eof,
            Token::RBrace,
            Token::Eof
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.parse(),
            ASTNode::Struct {
                name: "Point".into(),
                is_public: false,
                fields: HashMap::from_iter(vec![
                    ("x".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    }),
                    ("y".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    })
                ])
            }
        );
    }

    #[test]
    fn test_struct_instance() {
        let tokens = vec![
            Token::Identifier("Point".into()),
            Token::LBrace,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Number(Fraction::from(1)),
            Token::Comma,
            Token::Identifier("y".into()),
            Token::Colon,
            Token::Number(Fraction::from(2)),
            Token::RBrace,
            Token::Eof
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.parse(),
            ASTNode::StructInstance {
                name: "Point".into(),
                fields: HashMap::from_iter(vec![
                    ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                    ("y".into(), ASTNode::Literal(Value::Number(Fraction::from(2)))
                    )
                ])
            }
        );
    }

    #[test]
    fn test_struct_field_access() {
        let tokens = vec![
            Token::PublicStruct,
            Token::Identifier("Point".into()),
            Token::LBrace,
            Token::Eof,
            Token::Pub,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Comma,
            Token::Eof,
            Token::Pub,
            Token::Identifier("y".into()),
            Token::Colon,
            Token::Identifier("number".into()),
            Token::Eof,
            Token::RBrace,
            Token::Eof,
            Token::Immutable,
            Token::Identifier("point".into()),
            Token::Equal,
            Token::Identifier("Point".into()),
            Token::LBrace,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Number(Fraction::from(1)),
            Token::Comma,
            Token::Identifier("y".into()),
            Token::Colon,
            Token::Number(Fraction::from(2)),
            Token::RBrace,
            Token::Eof,
            Token::Identifier("point".into()),
            Token::Dot,
            Token::Identifier("x".into()),
            Token::Eof,
            Token::Identifier("point".into()),
            Token::Dot,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Number(Fraction::from(3)),
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.parse_lines(),
            vec![
                ASTNode::Struct {
                    name: "Point".into(),
                    is_public: true,
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ASTNode::StructField {
                            value_type: ValueType::Number,
                            is_public: true
                        }),
                        ("y".into(), ASTNode::StructField {
                            value_type: ValueType::Number,
                            is_public: true
                        })
                    ])
                },
                ASTNode::Assign {
                    name: "point".into(),
                    variable_type: EnvVariableType::Immutable,
                    is_new: true,
                    value_type: ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                        ("x".into(), ValueType::Number),
                        ("y".into(), ValueType::Number)
                    ])},
                    value: Box::new(ASTNode::StructInstance {
                        name: "Point".into(),
                        fields: HashMap::from_iter(vec![
                            ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                            ("y".into(), ASTNode::Literal(Value::Number(Fraction::from(2)))),
                        ]),
                    }),
                },
                ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable {
                        name: "point".into(),
                        value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                            ("x".into(), ValueType::Number),
                            ("y".into(), ValueType::Number)
                        ])})
                    }),
                    field_name: "x".into()
                },
                ASTNode::StructFieldAssign {
                    instance: Box::new(ASTNode::StructFieldAccess {
                        instance: Box::new(ASTNode::Variable {
                            name: "point".into(),
                            value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                                ("x".into(), ValueType::Number),
                                ("y".into(), ValueType::Number)
                            ])})
                        }),
                        field_name: "x".into()
                    }),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                    field_name: "x".into()
                }
            ]
        );
    }

    #[test]
    fn test_impl() {
        let tokens = vec![Token::Impl, Token::Identifier("Point".into()), Token::LBrace, Token::Eof, Token::Function, Token::Identifier("move".into()), Token::LParen, Token::Identifier("dx".into()), Token::Colon, Token::Identifier("number".into()), Token::RParen, Token::LBrace, Token::Eof, Token::Identifier("self".into()), Token::Dot, Token::Identifier("x".into()), Token::Equal, Token::Identifier("self".into()), Token::Dot, Token::Identifier("x".into()), Token::Plus, Token::Identifier("dx".into()), Token::Eof, Token::RBrace, Token::Eof, Token::RBrace, Token::Eof];
        let mut parser = Parser::new(tokens);
        let base_struct = ASTNode::Struct {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("x".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: false
                }),
            ]),
            is_public: false
        };
        parser.register_struct("global".into(), base_struct);
        let base_struct_type = ValueType::Struct {
            name: "Point".into(),
            is_public: false,
            fields: HashMap::from_iter(vec![
                ("x".into(), ValueType::StructField {
                    value_type: Box::new(ValueType::Number),
                    is_public: false
                })
            ])
        };
        assert_eq!(parser.parse_lines(), vec![ASTNode::Impl {
            base_struct: Box::new(base_struct_type.clone()),
            methods: vec![ASTNode::Method {
                name: "move".into(),
                arguments: vec![
                    ASTNode::Variable {
                        name: "self".into(),
                        value_type: None
                    },
                    ASTNode::Variable {
                        name: "dx".into(),
                        value_type: Some(ValueType::Number)
                    }
                ],
                body: Box::new(ASTNode::Block(vec![
                          ASTNode::StructFieldAssign {
                              instance: Box::new(ASTNode::StructFieldAccess {
                                  instance: Box::new(ASTNode::Variable {
                                      name: "self".into(),
                                      value_type: Some(base_struct_type.clone())
                                  }),
                                  field_name: "x".into()
                              }),
                              field_name: "x".into(),
                              value: Box::new(ASTNode::BinaryOp {
                                  left: Box::new(ASTNode::StructFieldAccess {
                                      instance: Box::new(ASTNode::Variable {
                                          name: "self".into(),
                                          value_type: Some(base_struct_type.clone())
                                      }),
                                      field_name: "x".into()
                                  }),
                                  op: Token::Plus,
                                  right: Box::new(ASTNode::Variable {
                                      name: "dx".into(),
                                      value_type: Some(ValueType::Number)
                                  })
                              })
                          }
                ])),
                return_type: ValueType::Void
            }]
        }]);
    }

    #[test]
    fn test_for() {
        let tokens = vec![
            Token::For,
            Token::Identifier("i".into()),
            Token::In,
            Token::LBrancket,
            Token::Number(Fraction::from(1)),
            Token::Comma,
            Token::Number(Fraction::from(2)),
            Token::Comma,
            Token::Number(Fraction::from(3)),
            Token::RBrancket,
            Token::LBrace,
            Token::Eof,
            Token::Identifier("print".into()),
            Token::LParen,
            Token::Identifier("i".into()),
            Token::RParen,
            Token::Eof,
            Token::RBrace,
            Token::Eof
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse(), ASTNode::For {
            variable: "i".into(),
            iterable: Box::new(ASTNode::Literal(Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3))
            ]))),
            body: Box::new(ASTNode::Block(vec![ASTNode::FunctionCall {
                name: "print".into(),
                arguments: Box::new(ASTNode::FunctionCallArgs(vec![ASTNode::Variable {
                    name: "i".into(),
                    value_type: None
                }]))
            }]))
        });
    }
}
