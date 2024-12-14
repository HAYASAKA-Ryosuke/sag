use std::collections::HashMap;
use crate::parser::{ASTNode, Value};

#[derive(Debug)]
pub struct Env {
    variable_map: HashMap<VariableKeyInfo, EnvVariableValueInfo>,
    scope_stack: Vec<String>,
    functions: HashMap<String, FunctionInfo>
}

#[derive(Debug)]
pub struct FunctionInfo {
    pub arguments: Vec<ASTNode>,
    pub return_type: ValueType,
    pub body: ASTNode,
}


#[derive(Eq, Hash, PartialEq, Debug)]
pub struct VariableKeyInfo {
    name: String,
    scope: String, 
}

#[derive(PartialEq, Debug)]
pub enum EnvVariableType {
    Immutable,
    Mutable
}

#[derive(PartialEq, Debug, Clone)]
pub enum ValueType {
    Any,
    Number,
    Str,
    Bool,
    Void,
    Function
}

#[derive(Debug)]
pub struct EnvVariableValueInfo {
    pub value: Value,
    pub variable_type: EnvVariableType,
    pub value_type: ValueType
}

impl Env {
    pub fn new() -> Self {
        Self{variable_map: HashMap::new(), scope_stack: vec!["global".to_string()], functions: HashMap::new()}
    }

    pub fn enter_scope(&mut self, scope: String) {
        self.scope_stack.push(scope);
    }
    pub fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn register_function(&mut self, name: String, function: FunctionInfo) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&mut self, name: String, function: FunctionInfo) -> Option<&FunctionInfo> {
        self.functions.get(&name)
    }

    pub fn set(&mut self, name: String, value: Value, variable_type: EnvVariableType, value_type: ValueType) -> Result<(), String> {
        let latest_scope = &self.scope_stack.last();
        if latest_scope.is_none() {
            return Err("missing scope".into());
        }
        let value_info = &self.get(name.clone());
        if value_info.is_some() {
            if value_info.unwrap().variable_type == EnvVariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
        }
        self.variable_map.insert(VariableKeyInfo{name, scope: latest_scope.unwrap().clone()}, EnvVariableValueInfo{value, variable_type, value_type});
        Ok(())
    }

    pub fn get(&self, name: String) -> Option<&EnvVariableValueInfo> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(variable_key_info)  = self.variable_map.get(&VariableKeyInfo{name: name.to_string(), scope: scope.clone()}) {
                return Some(variable_key_info);
            }
        }
        None
    }
}

