pub mod struct_ast;
pub mod function_ast;
pub mod return_ast;
pub mod assign_ast;
pub mod literal_ast;
pub mod pipe_ast;
pub mod lambda_ast;
pub mod infer_type;
pub mod for_ast;
pub mod if_ast;
pub mod identifier_ast;
pub mod method_ast;
pub mod list_ast;
pub mod block_ast;
pub mod prefix_op_ast;
pub mod string_to_value_type;
pub mod import_ast;

use crate::environment::{EnvVariableType, ValueType, MethodInfo};
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
    structs: HashMap<(String, String), (ValueType, EnvVariableType, HashMap<String, ASTNode>)>, // key: (scope, name), value: value_type
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

    fn enter_struct(&mut self, struct_name: String) {
        self.current_struct = Some(struct_name);
    }

    fn leave_struct(&mut self) {
        self.current_struct = None;
    }

    fn get_current_struct(&self) -> Option<String> {
        self.current_struct.clone()
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
        if let ASTNode::Struct { name, fields } = &struct_value {
            let field_types = fields.iter().map(|(name, field)| {
                if let ASTNode::StructField { value_type, is_public } = field {
                    (name.clone(), ValueType::StructField { value_type: Box::new(value_type.clone()), is_public: is_public.clone() })
                } else {
                    panic!("invalid struct field")
                }
            }).collect();
            let methods = HashMap::new();
            let insert_value = (ValueType::Struct { name: name.clone(), fields: field_types, methods }, EnvVariableType::Immutable, HashMap::new());
            self.structs.insert(
                (scope.to_string(), name.to_string()),
                insert_value,
            );
        }
    }

    fn register_method(&mut self, scope: String, struct_name: String, method: ASTNode) {
        if let ASTNode::Method { name: method_name, arguments, body, return_type, is_mut } = method.clone() {
            for scope in vec![scope.to_string(), "global".to_string()] {
                if let Some((value_type, _, _)) = self.structs.get_mut(&(scope.to_string(), struct_name.to_string())) {
                    match value_type {
                        ValueType::Struct { name: _, fields: _, methods } => {
                            let method_info = MethodInfo {
                                arguments: arguments.clone(),
                                body: Some(*body),
                                return_type: return_type.clone(),
                                is_mut: is_mut.clone(),
                            };
                            methods.insert(method_name.clone(), method_info);
                            break
                        }
                        _ => panic!("invalid method")
                    }
                }
            }
        } else {
            panic!("invalid method")
        }
    }

    fn get_struct(&mut self, scope: String, name: String) -> Option<ValueType> {
        for checked_scope in vec![scope.to_string(), "global".to_string()] {
            match self.structs.get(&(checked_scope.to_string(), name.to_string())) {
                Some((ref value_type, _, ..)) => match value_type.clone() {
                    ValueType::Struct { .. } => {
                        return Some(value_type.clone())
                    },
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
        _arguments: &Vec<ASTNode>,  // arugmentsも多重定義を許容するときに使う
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

    fn get_method(&self, scope: String, value_type: ValueType, method_name: String) -> Option<MethodInfo> {
        match value_type {
            ValueType::Struct { name, fields: _, methods } => {
                match methods.get(&method_name) {
                    Some(method) => Some(method.clone()),
                    None => None
                }
            },
            ValueType::Number => {
                if method_name == "to_string" {
                    Some(MethodInfo {
                        arguments: vec![],
                        body: None,
                        return_type: ValueType::String,
                        is_mut: false,
                    })
                } else {
                    None
                }
            }
            _ => None
        }
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

    fn is_lparen_call(&mut self) -> bool {
        self.pos += 1;
        let next_token = self.get_current_token();
        self.pos -= 1;
        next_token == Some(Token::LParen)
    }


    fn parse_primary(&mut self) -> ASTNode {
        let token = match self.get_current_token() {
            Some(token) => token,
            _ => panic!("token not found!"),
        };
        println!("token: {:?}", token);
        match token {
            Token::Struct => self.parse_struct(),
            Token::Pub => self.parse_public(),
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
            Token::Import => self.parse_import(),
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
                self.parse_identifier(name)
            }
            Token::CommentBlock(comment) => {ASTNode::CommentBlock(comment.to_string())},
            _ => panic!("undefined token: {:?}", token),
        }
    }

    fn parse_expression(&mut self, min_priority: u8) -> ASTNode {
        let mut lhs = self.parse_primary();
        loop {
            let token = match self.get_current_token() {
                Some(token) => token,
                _ => break,
            };
            if token == Token::Dot {
                self.pos += 2;
                if let Token::LParen = self.get_current_token().unwrap() {
                    self.pos -= 1;
                    if let Token::Identifier(method_name) = self.get_current_token().unwrap() {
                        println!("method_name: {:?}", method_name);
                        self.pos += 1;
                        let args = self.parse_function_call_arguments_paren();
                        println!("args: {:?}", args);
                        println!("pos: {:?}, {:?}", self.pos, self.get_current_token());

                        let builtin = match lhs {
                            ASTNode::Literal(Value::Number(_)) => true,
                            ASTNode::Literal(Value::String(_)) => true,
                            ASTNode::Literal(Value::Bool(_)) => true,
                            ASTNode::Literal(Value::Void) => true,
                            ASTNode::MethodCall { ref caller, .. } => {
                                match self.infer_type(&caller) {
                                    Ok(ValueType::Number) => true,
                                    Ok(ValueType::String) => true,
                                    Ok(ValueType::Bool) => true,
                                    Ok(ValueType::Void) => true,
                                    _ => false,
                                }
                            },
                            _ => false,
                        };

                        lhs = ASTNode::MethodCall{
                            caller: Box::new(lhs.clone()),
                            method_name,
                            builtin,
                            arguments: match args {
                                ASTNode::FunctionCallArgs(args) => {
                                    Box::new(ASTNode::FunctionCallArgs(vec![lhs].into_iter().chain(args.into_iter()).collect()))
                                }
                                _ => Box::new(ASTNode::FunctionCallArgs(vec![lhs])),
                            }
                        };
                        continue;
                    }
                }
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

            if let Some((left_priority, right_priority)) = self.get_priority(&token) {
                if left_priority < min_priority {
                    break;
                }
                self.pos += 1;

                let rhs = self.parse_expression(right_priority);
                if let Token::Eq = token {
                    lhs = ASTNode::Eq {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else if let Token::Gte = token {
                    lhs = ASTNode::Gte {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else if let Token::Gt = token {
                    lhs = ASTNode::Gt {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else if let Token::Lte = token {
                    lhs = ASTNode::Lte {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else if let Token::Lt = token {
                    lhs = ASTNode::Lt {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else {
                    lhs = ASTNode::BinaryOp {
                        left: Box::new(lhs),
                        op: token,
                        right: Box::new(rhs),
                    }
                }
            } else {
                break;
            }
        }
        lhs
    }
    fn get_priority(&self, token: &Token) -> Option<(u8, u8)> {
        match token {
            Token::Eq | Token::Gt | Token::Gte | Token::Lt | Token::Lte => Some((1, 2)),
            Token::Plus | Token::Minus => Some((3, 4)),
            Token::Mul | Token::Div | Token::Mod => Some((5, 6)),
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
            Token::Mod,
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
                    left: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                        op: Token::Mul,
                        right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3))))
                    }),
                    op: Token::Mod,
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
    #[should_panic(expected = "Return type mismatch Expected type: Void, Actual type: Number")]
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
            Token::Struct,
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
            Token::Pub,
            Token::Struct,
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
                ASTNode::Public { node: Box::new(
                    ASTNode::Struct {
                        name: "Point".into(),
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
                    }
                )},
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
        let tokens = vec![Token::Impl, Token::Identifier("Point".into()), Token::LBrace, Token::Eof, Token::Function, Token::Identifier("move".into()), Token::LParen, Token::Identifier("self".into()), Token::Comma, Token::Identifier("dx".into()), Token::Colon, Token::Identifier("number".into()), Token::RParen, Token::LBrace, Token::Eof, Token::Identifier("self".into()), Token::Dot, Token::Identifier("x".into()), Token::Equal, Token::Identifier("self".into()), Token::Dot, Token::Identifier("x".into()), Token::Plus, Token::Identifier("dx".into()), Token::Eof, Token::RBrace, Token::Eof, Token::RBrace, Token::Eof];
        let mut parser = Parser::new(tokens);
        let base_struct = ASTNode::Struct {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("x".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: false
                }),
            ])
        };
        parser.register_struct("global".into(), base_struct);
        let base_struct_type = ValueType::Struct {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("x".into(), ValueType::StructField {
                    value_type: Box::new(ValueType::Number),
                    is_public: false
                })
            ]),
            methods: HashMap::new()
        };
        assert_eq!(parser.parse_lines(), vec![ASTNode::Impl {
            base_struct: Box::new(base_struct_type.clone()),
            methods: vec![ASTNode::Method {
                name: "move".into(),
                arguments: vec![
                    ASTNode::Variable {
                        name: "self".into(),
                        value_type: Some(ValueType::SelfType)
                    },
                    ASTNode::Variable {
                        name: "dx".into(),
                        value_type: Some(ValueType::Number)
                    }
                ],
                is_mut: false,
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
