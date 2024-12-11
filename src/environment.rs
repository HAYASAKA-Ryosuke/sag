use std::collections::HashMap;
use crate::parser::Value;


pub struct Env {
    variable_map: HashMap<VariableKeyInfo, VariableValueInfo>,
    scope_stack: Vec<String>
}


#[derive(Eq, Hash, PartialEq)]
pub struct VariableKeyInfo {
    name: String,
    scope: String, 
}

#[derive(PartialEq, Debug)]
pub enum VariableType {
    Immutable,
    Mutable
}

#[derive(Debug)]
pub struct VariableValueInfo {
    pub value: Value,
    pub variable_type: VariableType
}

impl Env {
    pub fn new() -> Self {
        Self{variable_map: HashMap::new(), scope_stack: vec!["global".to_string()]}
    }

    pub fn enter_scope(&mut self, scope: String) {
        self.scope_stack.push(scope);
    }
    pub fn leave_scope(&mut self, scope: String) {
        self.scope_stack.pop();
    }

    pub fn set(&mut self, name: String, value: Value, variable_type: VariableType) -> Result<(), String> {
        let latest_scope = &self.scope_stack.last();
        if latest_scope.is_none() {
            return Err("missing scope".into());
        }
        let value_info = &self.get(name.clone());
        if value_info.is_some() {
            if value_info.unwrap().variable_type == VariableType::Immutable {
                return Err("Cannot reassign to immutable variable".into());
            }
        }
        self.variable_map.insert(VariableKeyInfo{name, scope: latest_scope.unwrap().clone()}, VariableValueInfo{value, variable_type});
        Ok(())
    }

    pub fn get(&self, name: String) -> Option<&VariableValueInfo> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(variable_key_info)  = self.variable_map.get(&VariableKeyInfo{name: name.to_string(), scope: scope.clone()}) {
                return Some(variable_key_info);
            }
        }
        None
    }
}

