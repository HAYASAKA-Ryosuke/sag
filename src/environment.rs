use std::collections::HashMap;
use crate::parser::{ASTNode, Value};

#[derive(Debug, Clone)]
pub struct Env {
    variable_map: HashMap<VariableKeyInfo, EnvVariableValueInfo>,
    scope_stack: Vec<String>,
    functions: HashMap<String, FunctionInfo>
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub arguments: Vec<ASTNode>,
    pub return_type: ValueType,
    pub body: ASTNode,
}


#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct VariableKeyInfo {
    name: String,
    scope: String, 
}

#[derive(PartialEq, Debug, Clone)]
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

#[derive(Debug, Clone)]
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
        if self.scope_stack.len() == 1 && self.scope_stack[0] == "global".to_string() {
            return
        }

        self.scope_stack.pop();
    }

    pub fn register_function(&mut self, name: String, function: FunctionInfo) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&mut self, name: String) -> Option<&FunctionInfo> {
        self.functions.get(&name)
    }

    pub fn update_global_env(&mut self, local_env: &Self) {
        for (local_key, local_value) in &local_env.variable_map {
            if local_key.scope == "global" && self.variable_map.contains_key(local_key) {
                self.variable_map.insert(local_key.clone(), local_value.clone());
            }
        }
    }

    pub fn set(&mut self, name: String, value: Value, variable_type: EnvVariableType, value_type: ValueType, is_new: bool) -> Result<(), String> {
        let latest_scope = match self.scope_stack.last() {
            Some(scope) => scope.clone(),
            None => return Err("Missing scope".into()),
        };

        // 新規の場合はそのまま書き込み
        if is_new {
            self.variable_map.insert(
                VariableKeyInfo { name: name.clone(), scope: latest_scope },
                EnvVariableValueInfo { value, variable_type, value_type },
            );
            return Ok(())
        }

        // ローカルスコープの変数をチェックと存在したら更新
        if let Some(value_info) = self.get_with_scope(name.clone(), latest_scope.clone()) {
            if value_info.variable_type == EnvVariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
            self.variable_map.insert(VariableKeyInfo {name, scope: latest_scope}, EnvVariableValueInfo {value, variable_type, value_type});
            return Ok(());
        }

        // グローバルスコープの変数をチェックと存在したら更新
        if let Some(value_info) = self.get_with_scope(name.clone(), "global".to_string()) {
            if value_info.variable_type == EnvVariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
            // グローバル変数を更新
            self.variable_map.insert(VariableKeyInfo {name, scope: "global".to_string()}, EnvVariableValueInfo {value, variable_type, value_type});
            return Ok(());
        }

        // どこにも存在しないので新しい変数としてローカルスコープに追加
        self.variable_map.insert(VariableKeyInfo {name, scope: latest_scope}, EnvVariableValueInfo {value, variable_type, value_type});

        Ok(())
    }

    fn get_with_scope(&self, name: String, scope: String) -> Option<&EnvVariableValueInfo> {
        if let Some(variable_key_info) = self.variable_map.get(&VariableKeyInfo{name: name.to_string(), scope: scope.clone()}) {
            return Some(variable_key_info);
        }
        None
    }

    pub fn get(&self, name: String, value_type: Option<&ValueType>) -> Option<&EnvVariableValueInfo> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(variable_key_info) = self.variable_map.get(&VariableKeyInfo{name: name.to_string(), scope: scope.clone()}) {
                if value_type.is_some() {
                    if variable_key_info.value_type != *value_type.unwrap() {
                        continue
                    }
                }
                return Some(variable_key_info);
            }
        }
        None
    }
}

