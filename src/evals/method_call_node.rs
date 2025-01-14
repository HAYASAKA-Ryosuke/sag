use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, MethodInfo, EnvVariableType, EnvVariableValueInfo};
use crate::evals::eval;

pub fn builtin_method_call_node(method_name: String, caller: Box<ASTNode>, arguments: Box<ASTNode>, env: &mut Env) -> Value {
    match *caller {
        ASTNode::MethodCall { method_name: _, caller: _, arguments: _, builtin: _ } => {
            let method_name = method_name.clone();
            let arguments = match *arguments {
                ASTNode::FunctionCallArgs(arguments) => arguments,
                _ => vec![],
            };
            match method_name.as_str() {
                "to_string" => {
                    let value = eval(arguments[0].clone(), env);
                    match value {
                        Value::Number(value) => Value::String(value.to_string()),
                        _ => Value::Void,
                    }
                },
                "round" => {
                    let value = eval(arguments[0].clone(), env);
                    match value {
                        Value::Number(value) => Value::Number(value.round()),
                        _ => Value::Void,
                    }
                },
                "push" => {
                    let mut list = match eval(arguments[0].clone(), env) {
                        Value::List(list) => list,
                        _ => vec![],
                    };
                    let value = eval(arguments[1].clone(), env);
                    list.push(value);
                    Value::List(list)
                },
                _ => Value::Void,
            }
        }
        ASTNode::Literal(Value::Number(_)) => {
            let method_name = method_name.clone();
            let arguments = match *arguments {
                ASTNode::FunctionCallArgs(arguments) => arguments,
                _ => vec![],
            };
            match method_name.as_str() {
                "to_string" => {
                    let value = eval(arguments[0].clone(), env);
                    match value {
                        Value::Number(value) => Value::String(value.to_string()),
                        _ => Value::Void,
                    }
                },
                "round" => {
                    let value = eval(arguments[0].clone(), env);
                    match value {
                        Value::Number(value) => Value::Number(value.round()),
                        _ => Value::Void,
                    }
                },
                "push" => {
                    let mut list = match *caller {
                        ASTNode::Variable { name, value_type: _ } => {
                            let variable = env.get(&name, None).unwrap();
                            match variable.value.clone() {
                                Value::List(list) => list,
                                _ => vec![],
                            }
                        },
                        _ => vec![],
                    };
                    let value = eval(arguments[1].clone(), env);
                    list.push(value);
                    Value::List(list)
                },
                _ => Value::Void,
            }
        }
        ASTNode::Variable { ref name, ref value_type } => {
            match method_name.as_str() {
                "push" => {
                    let variable_info = env.get(&name, value_type.as_ref()).unwrap().clone();
                    let mut variable = match variable_info.value.clone() {
                        Value::List(list) => list,
                        _ => vec![],
                    };
                    let value = match *arguments {
                        ASTNode::FunctionCallArgs(arguments) => eval(arguments[0].clone(), env),
                        _ => Value::Void,
                    };
                    variable.push(value);
                    let _ = env.set(name.to_string(), Value::List(variable.clone()), variable_info.variable_type.clone(), variable_info.value_type.clone(), false);
                    Value::List(variable)
                }
                _ => Value::Void,
            }
        }
        _ => Value::Void,
    }
}

pub fn method_call_node(method_name: String, caller: Box<ASTNode>, arguments: Box<ASTNode>, env: &mut Env) -> Value {
            let mut args_vec = vec![];
            match *arguments {
               ASTNode::FunctionCallArgs(arguments) => {
                   args_vec = arguments
               }
               _ => {}
            }
            let caller_name = match *caller {
                ASTNode::Variable { name, value_type: _ } => {
                    name
                },
                _ => panic!("Unexpected caller: {:?}", caller),
            };
            match env.get(&caller_name, None) {
                Some(EnvVariableValueInfo{value, value_type, variable_type}) => {
                    let mut local_env = env.clone();
                    let struct_info = match value_type {
                        ValueType::StructInstance { name: struct_name, ..} => {
                            local_env.get_struct(&struct_name).cloned()
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
                                panic!("{} is not mutable", caller_name);
                            }
                            if args_vec.len() != define_arguments.len() - 1 {
                                panic!("does not match arguments length");
                            }
                            local_env.enter_scope(method_name);
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
                            if let Some(self_value) = local_env.get(&"self".to_string(), None) {
                                if let Value::StructInstance { .. } = self_value.value.clone() {
                                    local_env.set(
                                        caller_name.to_string(),
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
                None => panic!("missing struct: {}", caller_name)
            }
}
