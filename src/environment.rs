use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use wasm_bindgen::prelude::*;
use crate::tokenizer::tokenize;
use crate::parsers::Parser;
use crate::evals::evals;
use crate::builtin::register_builtins;


#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    variable_map: HashMap<VariableKeyInfo, EnvVariableValueInfo>,
    scope_stack: Vec<String>,
    functions: HashMap<String, FunctionInfo>,
    structs: HashMap<String, Value>,
    builtins: HashMap<String, FunctionInfo>,
    modules: HashMap<String, Env>,
    exported_symbols: HashMap<String, ExportedSymbolType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportedSymbolType {
    Function,
    Variable,
    Struct
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInfo {
    pub arguments: Vec<ASTNode>,
    pub return_type: ValueType,
    pub body: Option<ASTNode>,
    pub builtin: Option<fn(Vec<Value>) -> Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodInfo {
    pub arguments: Vec<ASTNode>,
    pub return_type: ValueType,
    pub body: Option<ASTNode>,
    pub is_mut: bool,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct VariableKeyInfo {
    name: String,
    scope: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum EnvVariableType {
    Immutable,
    Mutable,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ValueType {
    Any,
    Number,
    String,
    Bool,
    Void,
    SelfType,
    MutSelfType,
    List(Box<ValueType>),
    Function,
    Lambda,
    Return,
    Struct{name: String, fields: HashMap<String, ValueType>, methods: HashMap<String, MethodInfo>},
    StructField{value_type: Box<ValueType>, is_public: bool},
    StructInstance{name: String, fields: HashMap<String, ValueType>},
    Impl{base_struct: Box<ValueType>, methods: HashMap<String, MethodInfo>},
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvVariableValueInfo {
    pub value: Value,
    pub variable_type: EnvVariableType,
    pub value_type: ValueType,
}

impl Env {
    pub fn new() -> Self {
        Self {
            variable_map: HashMap::new(),
            scope_stack: vec!["global".to_string()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            builtins: HashMap::new(),
            modules: HashMap::new(),
            exported_symbols: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module_name: &String, module_path: &String) -> Result<(), String> {
        if self.modules.contains_key(module_name) {
            // 登録済
            return Ok(());
        }

        let file_content = std::fs::read_to_string(module_path)
            .map_err(|e| format!("Failed to read file '{}': {}", module_path, e))?;

        let tokens = tokenize(&file_content);
        let builtins = register_builtins(self);
        let mut parser = Parser::new(tokens, builtins);
        let ast_nodes = parser.parse_lines();
        if let Err(e) = ast_nodes {
            return Err(format!("Error: {:?}", e));
        }

        let mut module_env = Env::new();
        evals(ast_nodes.unwrap(), &mut module_env);
        self.modules.insert(module_name.to_string(), module_env);
        Ok(())
    }

    pub fn get_module(&self, module_name: &String) -> Option<&Env> {
        self.modules.get(module_name)
    }

    pub fn register_exported_symbol(&mut self, name: String) {
        if let Some(_) = self.variable_map.get(&VariableKeyInfo {
            name: name.clone(),
            scope: "global".to_string(),
        }) {
            self.exported_symbols.insert(name, ExportedSymbolType::Variable);
        } else if let Some(_) = self.get_function(&name) {
            self.exported_symbols.insert(name, ExportedSymbolType::Function);
        } else if let Some(_) = self.get_struct(&name) {
            self.exported_symbols.insert(name, ExportedSymbolType::Struct);
        }
    }

    pub fn get_exported_symbol(&self, name: &String) -> Option<&ExportedSymbolType> {
        self.exported_symbols.get(name)
    }

    pub fn register_struct(&mut self, struct_value: Value) {
        let name = match struct_value {
            Value::Struct { ref name, .. } => name.clone(),
            _ => panic!("Invalid struct value"),
        };
        if self.structs.contains_key(&name) {
            panic!("Struct {} already exists", name);
        }
        self.structs.insert(name.clone(), struct_value.clone());
    }

    pub fn get_struct(&self, name: &String) -> Option<&Value> {
        self.structs.get(name)
    }

    pub fn register_impl(&mut self, impl_value: Value) {
        match impl_value {
            Value::Impl { base_struct, methods } => {
                if let ValueType::Struct { name, .. } = base_struct {
                    if let Some(Value::Struct { methods: ref mut struct_methods, .. }) = self.structs.get_mut(&name) {
                        for (method_name, method_info) in methods {
                            struct_methods.insert(method_name.clone(), method_info.clone());
                        }
                    } else {
                        panic!("Struct '{}' not found for Impl", name);
                    }
                } else {
                    panic!("Invalid base_struct in Impl");
                }
            }
            _ => panic!("Invalid impl value"),
        }
    }

    pub fn register_builtin(&mut self, name: String, function: fn(Vec<Value>) -> Value) {
        let function_info = FunctionInfo {
            arguments: vec![],
            return_type: ValueType::Any,
            body: None,
            builtin: Some(function),
        };
        self.builtins.insert(name, function_info);
    }

    pub fn get_builtin(&self, name: &String) -> Option<&FunctionInfo> {
        self.builtins.get(name)
    }

    pub fn enter_scope(&mut self, scope: String) {
        self.scope_stack.push(scope);
    }
    pub fn leave_scope(&mut self) {
        if self.scope_stack.len() == 1 && self.scope_stack[0] == "global".to_string() {
            return;
        }

        self.scope_stack.pop();
    }

    pub fn get_current_scope(&self) -> String {
        match self.scope_stack.last() {
            Some(scope) => scope.clone(),
            None => "global".to_string(),
        }
    }

    pub fn register_function(&mut self, name: String, function: FunctionInfo) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&mut self, name: &String) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    pub fn update_global_env(&mut self, local_env: &Self) {
        for (local_key, local_value) in &local_env.variable_map {
            if local_key.scope == "global" && self.variable_map.contains_key(local_key) {
                self.variable_map
                    .insert(local_key.clone(), local_value.clone());
            }
        }
    }

    pub fn set(
        &mut self,
        name: String,
        value: Value,
        variable_type: EnvVariableType,
        value_type: ValueType,
        is_new: bool,
    ) -> Result<(), String> {
        let latest_scope = match self.scope_stack.last() {
            Some(scope) => scope.clone(),
            None => return Err("Missing scope".into()),
        };

        // 新規の場合はそのまま書き込み
        if is_new {
            self.variable_map.insert(
                VariableKeyInfo {
                    name: name.clone(),
                    scope: latest_scope,
                },
                EnvVariableValueInfo {
                    value,
                    variable_type,
                    value_type,
                },
            );
            return Ok(());
        }

        // ローカルスコープの変数をチェックと存在したら更新
        if let Some(value_info) = self.get_with_scope(name.clone(), latest_scope.clone()) {
            if value_info.variable_type == EnvVariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
            self.variable_map.insert(
                VariableKeyInfo {
                    name,
                    scope: latest_scope,
                },
                EnvVariableValueInfo {
                    value,
                    variable_type,
                    value_type,
                },
            );
            return Ok(());
        }

        // グローバルスコープの変数をチェックと存在したら更新
        if let Some(value_info) = self.get_with_scope(name.clone(), "global".to_string()) {
            if value_info.variable_type == EnvVariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
            // グローバル変数を更新
            self.variable_map.insert(
                VariableKeyInfo {
                    name,
                    scope: "global".to_string(),
                },
                EnvVariableValueInfo {
                    value,
                    variable_type,
                    value_type,
                },
            );
            return Ok(());
        }

        // どこにも存在しないので新しい変数としてローカルスコープに追加
        self.variable_map.insert(
            VariableKeyInfo {
                name,
                scope: latest_scope,
            },
            EnvVariableValueInfo {
                value,
                variable_type,
                value_type,
            },
        );

        Ok(())
    }

    fn get_with_scope(&self, name: String, scope: String) -> Option<&EnvVariableValueInfo> {
        if let Some(variable_key_info) = self.variable_map.get(&VariableKeyInfo {
            name: name.to_string(),
            scope: scope.clone(),
        }) {
            return Some(variable_key_info);
        }
        None
    }

    pub fn get(
        &self,
        name: &String,
        value_type: Option<&ValueType>,
    ) -> Option<&EnvVariableValueInfo> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(variable_key_info) = self.variable_map.get(&VariableKeyInfo {
                name: name.to_string(),
                scope: scope.clone(),
            }) {
                if value_type.is_some() {
                    if variable_key_info.value_type != *value_type.unwrap() {
                        continue;
                    }
                }
                return Some(variable_key_info);
            }
        }
        None
    }
}
