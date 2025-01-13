use std::collections::HashMap;
use crate::token::Token;
use crate::value::Value;
use crate::environment::{ValueType, EnvVariableType};

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
    Method {
        name: String,
        arguments: Vec<ASTNode>,
        body: Box<ASTNode>,
        return_type: ValueType,
        is_mut: bool,
    },
    MethodCall {
        method_name: String,
        caller: Box<ASTNode>,
        arguments: Box<ASTNode>
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
    Struct {
        name: String,
        fields: HashMap<String, ASTNode>,  // field_name: StructField
    },
    StructField {
        value_type: ValueType,
        is_public: bool,
    },
    StructFieldAccess {
        instance: Box<ASTNode>,  // StructInstance, variable
        field_name: String,
    },
    StructFieldAssign {
        instance: Box<ASTNode>,  // StructInstance, variable
        field_name: String,
        value: Box<ASTNode>,
    },
    StructInstance {
        name: String,
        fields: HashMap<String, ASTNode>,
    },
    Impl {
        base_struct: Box<ValueType>,
        methods: Vec<ASTNode>,
    },
    CommentBlock(String),
    For {
        variable: String,
        iterable: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    Import {
        module_name: String,
        symbols: Vec<String>,
    },
    Public {
        node: Box<ASTNode>,
    },
}

