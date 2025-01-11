use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, MethodInfo, EnvVariableValueInfo, EnvVariableType};
use crate::evals::eval;

pub fn struct_node(name: String, fields: HashMap<String, ASTNode>, env: &mut Env) -> Value {
    let mut struct_fields = HashMap::new();
    // fields field_name: StructField
    for (field_name, struct_field) in fields {
        match struct_field {
            ASTNode::StructField { value_type, is_public } => {
                struct_fields.insert(field_name, Value::StructField {
                    value_type,
                    is_public
                });
            },
            _ => panic!("Unexpected struct field: {:?}", struct_field),
        }
    }
    let result = Value::Struct {
        name,
        fields: struct_fields,
        methods: HashMap::new()
    };
    env.register_struct(result.clone());
    result
}

pub fn impl_node(base_struct: Box<ValueType>, methods: Vec<ASTNode>, env: &mut Env) -> Value {
    let mut impl_methods = HashMap::new();
    for method in methods {
        match method {
            ASTNode::Method {
                name,
                arguments,
                body,
                return_type,
                is_mut
            } => {
                let method_info = MethodInfo {
                    arguments,
                    body: Some(*body),
                    return_type,
                    is_mut,
                };
                impl_methods.insert(name, method_info);
            },
            _ => panic!("Unexpected method: {:?}", method),
        }
    }
    let result = Value::Impl {
        base_struct: *base_struct,
        methods: impl_methods,
    };
    env.register_impl(result.clone());
    result
}

pub fn method_call_node(method_name: String, caller: String, arguments: Box<ASTNode>, env: &mut Env) -> Value {
            let mut args_vec = vec![];
            match *arguments {
               ASTNode::FunctionCallArgs(arguments) => {
                   args_vec = arguments
               }
               _ => {}
            }
            match env.get(caller.clone(), None) {
                Some(EnvVariableValueInfo{value, value_type, variable_type}) => {
                    let mut local_env = env.clone();
                    let struct_info = match value_type {
                        ValueType::StructInstance { name: struct_name, ..} => {
                            local_env.get_struct(struct_name.to_string()).cloned()
                        },
                        _ => panic!("missing struct: {}", value)
                    };

                    let methods = match struct_info {
                        Some(Value::Struct{ref methods, ..}) => methods,
                        _ => panic!("failed get methods")
                    };
                    match methods.get(&method_name) {
                        Some(MethodInfo{arguments: define_arguments, return_type: _, body, is_mut: _}) => {

                            if *variable_type == EnvVariableType::Immutable {
                                panic!("{} is not mutable", caller);
                            }
                            if args_vec.len() != define_arguments.len() - 1 {
                                panic!("does not match arguments length");
                            }
                            local_env.enter_scope(method_name.to_string());
                            let _ = local_env.set(
                                "self".to_string(),
                                match struct_info {
                                    Some(Value::Struct{..}) => {
                                        value.clone()
                                    },
                                    _ => panic!("failed struct")
                                },
                                EnvVariableType::Mutable,
                                match struct_info {
                                    Some(Value::Struct{name, fields, methods: _}) => {
                                        let mut field_types = HashMap::new();
                                        for (field_name, field_value) in fields {
                                            field_types.insert(field_name.to_string(), field_value.value_type());
                                        }
                                        ValueType::Struct{name: name.to_string(), fields: field_types, methods: methods.clone()}
                                    },
                                    _ => panic!("failed struct")
                                },
                                true
                            );
                            // set struct_instance fields
                            for (field_name, field_value) in match value.clone() {
                                Value::StructInstance { name: _, fields } => fields,
                                _ => panic!("missing struct instance")
                            } {
                                let _ = local_env.set(
                                    field_name.to_string(),
                                    field_value.clone(),
                                    EnvVariableType::Mutable,
                                    field_value.value_type(),
                                    true
                                );
                            }
                            let mut i = 0;
                            for define_arg in define_arguments {
                                if let ASTNode::Variable {name, value_type} = define_arg {
                                    if name == "self" {
                                        continue
                                    }
                                    let arg_value = eval(args_vec[i].clone(), &mut local_env.clone());
                                    let _ = local_env.set(name.to_string(), arg_value, EnvVariableType::Immutable, value_type.clone().unwrap_or(ValueType::Any), true);
                                    i += 1;
                                }
                            }
                            let result = eval(body.clone().unwrap(), &mut local_env);
                            if let Some(self_value) = local_env.get("self".to_string(), None) {
                                if let Value::StructInstance { .. } = self_value.value.clone() {
                                    local_env.set(
                                        caller.to_string(),
                                        self_value.value.clone(),
                                        variable_type.clone(),
                                        value_type.clone(),
                                        false,
                                    ).expect("Failed to update self in global environment");
                                }
                            }
                            env.update_global_env(&local_env);
                            env.leave_scope();
                            result
                        },
                        _ => panic!("call failed method: {}", method_name)
                    }
                },
                None => panic!("missing struct: {}", caller)
            }
}

pub fn struct_instance_node(name: String, fields: HashMap<String, ASTNode>, env: &mut Env) -> Value {
    let mut struct_fields = HashMap::new();
    for (field_name, field_value) in fields {
        struct_fields.insert(field_name, eval(field_value, env));
    }
    Value::StructInstance {
        name,
        fields: struct_fields,
    }
}

pub fn struct_field_assign_node(instance: Box<ASTNode>, updated_field_name: String, updated_value_ast: Box<ASTNode>, env: &mut Env) -> Value {
    match *instance {
        ASTNode::StructFieldAccess { instance, field_name: _  } => {
            match *instance {
                ASTNode::Variable { name: variable_name, value_type } => {
                    match value_type {
                        Some(ValueType::Struct{name, fields, ..}) if variable_name == "self" => {
                            match env.get_struct(name.to_string()) {
                                Some(Value::Struct { fields: _, methods, .. }) => {
                                    let scope = env.get_current_scope();
                                    match methods.get(&scope) {
                                        Some(MethodInfo {arguments, ..}) => {
                                            let first_argument = arguments.first();
                                            if first_argument.is_none() {
                                                panic!("missing self argument");
                                            }
                                            match first_argument.unwrap() {
                                                ASTNode::Variable { name: self_argument, value_type: self_type } => {
                                                    if self_argument != "self" || *self_type != Some(ValueType::MutSelfType) {
                                                        panic!("{} is not mut self argument", scope);
                                                    }
                                                },
                                                _ => panic!("missing self argument"),
                                            }
                                        },
                                        _ => panic!("missing method info")
                                    }
                                },
                                _ => panic!("Unexpected value type"),
                            };
                            let obj = env.get(variable_name.to_string(), None);
                            if obj.is_none() {
                                panic!("Variable not found: {:?}", variable_name);
                            }
                            let mut struct_fields = HashMap::new();
                            match obj.unwrap().value.clone() {
                                Value::StructInstance { .. } => {
                                    let instance_value = obj.unwrap().value.clone();
                                    let updated_value = match instance_value {
                                        Value::StructInstance { name, fields } => {
                                            let mut updated_fields = fields.clone();
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            *updated_fields.entry(updated_field_name.to_string()).or_insert(updated_value.clone()) = updated_value.clone();
                                            Value::StructInstance{name, fields: updated_fields}
                                        },
                                        _ => panic!("missing struct instance value: {:?}", instance_value)
                                    };
                                    env.set(variable_name.to_string(), updated_value.clone(), EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false).expect("update variable");
                                    updated_value
                                },
                                Value::Struct { name: _, fields: obj_fields, .. } => {
                                    for (field_name, field_value) in obj_fields {
                                        if field_name == updated_field_name {
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            if field_value.value_type() != updated_value.value_type() {
                                                panic!("Struct field type mismatch: {}.{}:{:?} = {:?}", variable_name, field_name, field_value.value_type(), updated_value.value_type());
                                            }
                                            struct_fields.insert(field_name, updated_value);
                                        } else {
                                            struct_fields.insert(field_name, field_value);
                                        }
                                    }
                                    let env_updated_result = env.set(variable_name.to_string(), Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields.clone(),
                                    }, EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false);
                                    if env_updated_result.is_err() {
                                        panic!("{}", env_updated_result.unwrap_err());
                                    }
                                    Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields,
                                    }
                                },
                                _ => panic!("Unexpected value type: {:?}", obj),
                            }
                        }
                        Some(ValueType::StructInstance { name, fields }) => {
                            let obj = env.get(variable_name.to_string(), Some(&ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }));
                            if obj.is_none() {
                                panic!("Variable not found: {:?}", variable_name);
                            }
                            let mut struct_fields = HashMap::new();
                            match obj.unwrap().value.clone() {
                                Value::StructInstance { name: _, fields: obj_fields } => {
                                    for (field_name, field_value) in obj_fields {
                                        if field_name == updated_field_name {
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            if field_value.value_type() != updated_value.value_type() {
                                                panic!("Struct field type mismatch: {}.{}:{:?} = {:?}", variable_name, field_name, field_value.value_type(), updated_value.value_type());
                                            }
                                            struct_fields.insert(field_name, updated_value);
                                        } else {
                                            struct_fields.insert(field_name, field_value);
                                        }
                                    }
                                    let env_updated_result = env.set(variable_name.to_string(), Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields.clone(),
                                    }, EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false);
                                    if env_updated_result.is_err() {
                                        panic!("{}", env_updated_result.unwrap_err());
                                    }
                                    Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields,
                                    }
                                },
                                _ => panic!("Unexpected value type"),
                            }
                        },
                        _ => panic!("Unexpected value type"),
                    }
                },
                _ => panic!("Unexpected value type"),
            }
        },
        _ => panic!("Unexpected value type"),
    }
}

pub fn struct_field_access_node(instance: Box<ASTNode>, field_name: String, env: &mut Env) -> Value {
    let struct_obj = match *instance {
        ASTNode::Variable { name: variable_name, value_type } => {
            match value_type {
                Some(ValueType::Struct { .. }) if variable_name == "self" => {
                    let obj = env.get(variable_name.to_string(), None);
                    if obj.is_none() {
                        panic!("Variable not found: {:?}", variable_name);
                    }
                    obj.unwrap().value.clone()
                },
                Some(ValueType::StructInstance { name, fields }) => {
                    let obj = env.get(variable_name.to_string(), Some(&ValueType::StructInstance { name: name.to_string(), fields }));
                    if obj.is_none() {
                        panic!("Variable not found: {:?}", variable_name);
                    }
                    obj.unwrap().value.clone()
                },
                _ => panic!("Unexpected value type"),
            }
        },
        _ => panic!("Unexpected value type"),
    };
    match struct_obj {
        Value::Struct {fields, ..} => {
            // selfのケース
            if !fields.contains_key(&field_name) {
                panic!("Field not found: {:?}", field_name);
            }
            fields.get(&field_name).unwrap().clone()
        }
        Value::StructInstance { name: _, fields } => {
            if !fields.contains_key(&field_name) {
                panic!("Field not found: {:?}", field_name);
            }
            fields.get(&field_name).unwrap().clone()
        },
        _ => panic!("Unexpected value: {:?}", struct_obj),
    }
}
