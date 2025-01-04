use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::ASTNode;
use crate::value::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    variable_map: Rc<RefCell<HashMap<VariableKeyInfo, EnvVariableValueInfo>>>,
    scope_stack: Rc<RefCell<Vec<String>>>,
    functions: Rc<RefCell<HashMap<String, FunctionInfo>>>,
    structs: Rc<RefCell<HashMap<String, Value>>>,
    builtins: Rc<RefCell<HashMap<String, FunctionInfo>>>,
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
    List(Box<ValueType>),
    Function,
    Lambda,
    Return,
    Struct{name: String, fields: HashMap<String, ValueType>, is_public: bool},
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
            variable_map: Rc::new(RefCell::new(HashMap::new())),
            scope_stack: Rc::new(RefCell::new(vec!["global".to_string()])),
            functions: Rc::new(RefCell::new(HashMap::new())),
            structs: Rc::new(RefCell::new(HashMap::new())),
            builtins: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_struct(&self, struct_value: Value) {
        let name = match struct_value {
            Value::Struct { ref name, .. } => name.clone(),
            _ => panic!("Invalid struct value"),
        };
        let mut structs = self.structs.borrow_mut();
        if structs.contains_key(&name) {
            panic!("Struct {} already exists", name);
        }
        structs.insert(name.clone(), struct_value.clone());
    }

    pub fn get_struct(&self, name: String) -> Option<Value> {
        let structs = self.structs.borrow();
        structs.get(&name).cloned()
    }

    pub fn register_impl(&self, impl_value: Value) {
        match impl_value {
            Value::Impl { base_struct, methods } => {
                if let ValueType::Struct { name, .. } = base_struct {
                    let mut structs = self.structs.borrow_mut();
                    if let Some(Value::Struct { methods: ref mut struct_methods, .. }) = structs.get_mut(&name) {
                        for (method_name, method_info) in methods {
                            struct_methods.insert(method_name, method_info);
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
    
    pub fn register_builtin(&self, name: String, function: fn(Vec<Value>) -> Value) {
        let function_info = FunctionInfo {
            arguments: vec![],
            return_type: ValueType::Any,
            body: None,
            builtin: Some(function),
        };
        let mut builtins = self.builtins.borrow_mut();
        builtins.insert(name, function_info);
    }

    pub fn get_builtin(&self, name: String) -> Option<FunctionInfo> {
        let builtins = self.builtins.borrow();
        builtins.get(&name).cloned()
    }

    pub fn enter_scope(&self, scope: String) {
        let mut scope_stack = self.scope_stack.borrow_mut();
        scope_stack.push(scope);
    }
    pub fn leave_scope(&self) {
        let mut scope_stack = self.scope_stack.borrow_mut();
        if scope_stack.len() == 1 && scope_stack[0] == "global".to_string() {
            return;
        }

        scope_stack.pop();
    }

    pub fn register_function(&self, name: String, function: FunctionInfo) {
        let mut functions = self.functions.borrow_mut();
        functions.insert(name, function);
    }

    pub fn get_function(&self, name: String) -> Option<FunctionInfo> {
        let functions = self.functions.borrow();
        functions.get(&name).cloned()
    }

    pub fn update_global_env(&self, local_env: &Self) {
        let mut variable_map = self.variable_map.borrow_mut();
        let local_variable_map = local_env.variable_map.borrow();
        for (local_key, local_value) in &*local_variable_map {
            if local_key.scope == "global" && variable_map.contains_key(local_key) {
                variable_map
                    .insert(local_key.clone(), local_value.clone());
            }
        }
    }

    pub fn set(
        &self,
        name: String,
        value: Value,
        variable_type: EnvVariableType,
        value_type: ValueType,
        is_new: bool,
    ) -> Result<(), String> {
        let latest_scope = {
            let scope_stack = self.scope_stack.borrow();
            match scope_stack.last() {
                Some(scope) => scope.clone(),
                None => return Err("Missing scope".into()),
            }
        };

        let mut variable_map = self.variable_map.borrow_mut();

        // 新規の場合はそのまま書き込み
        if is_new {
            variable_map.insert(
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
            variable_map.insert(
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
            variable_map.insert(
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
        variable_map.insert(
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

    fn get_with_scope(&self, name: String, scope: String) -> Option<EnvVariableValueInfo> {
        let variable_map = self.variable_map.borrow();
        if let Some(variable_key_info) = variable_map.get(&VariableKeyInfo {
            name: name.to_string(),
            scope: scope.clone(),
        }) {
            return Some(variable_key_info.clone());
        }
        None
    }

    pub fn get(
        &self,
        name: String,
        value_type: Option<&ValueType>,
    ) -> Option<EnvVariableValueInfo> {
        let variable_map = self.variable_map.borrow();
        let scope_stack = self.scope_stack.borrow();
        for scope in scope_stack.iter().rev() {
            if let Some(variable_key_info) = variable_map.get(&VariableKeyInfo {
                name: name.to_string(),
                scope: scope.clone(),
            }) {
                if value_type.is_some() {
                    if variable_key_info.value_type != *value_type.unwrap() {
                        continue;
                    }
                }
                return Some(variable_key_info.clone());
            }
        }
        None
    }
}
