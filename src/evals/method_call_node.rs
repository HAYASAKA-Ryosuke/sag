use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, MethodInfo, EnvVariableType, EnvVariableValueInfo, FunctionInfo};
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn builtin_method_call_node(method_name: String, caller: Box<ASTNode>, arguments: Box<ASTNode>, _line: usize, _column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    match *caller.clone() {
        ASTNode::FunctionCall { name, arguments, line, column } => {
            let arguments = match *arguments { ASTNode::FunctionCallArgs{args: arguments, ..} => arguments,
                _ => vec![],
            };
            let function_reutrn_type = match env.get_function(&name) {
                Some(FunctionInfo{return_type, ..}) => return_type,
                _ => return Err(RuntimeError::new(format!("missing function: {:?}", name).as_str(), line, column)),
            };
            match function_reutrn_type {
                ValueType::Number => {
                    match method_name.as_str() {
                        "to_string" => {
                            let value = eval(*caller.clone(), env)?;
                            Ok(match value {
                                Value::Number(value) => Value::String(value.to_string()),
                                _ => Value::Void,
                            })
                        },
                        "round" => {
                            let value = eval(*caller.clone(), env)?;
                            Ok(match value {
                                Value::Number(value) => Value::Number(value.round()),
                                _ => Value::Void,
                            })
                        },
                        _ => Err(RuntimeError::new(format!("{} is not a method of number", method_name).as_str(), line, column)),
                    }
                },
                ValueType::List(_) => {
                    match method_name.as_str() {
                        "to_string" => {
                            let value = eval(*caller.clone(), env)?;
                            Ok(match value {
                                Value::Number(value) => Value::String(value.to_string()),
                                _ => Value::Void,
                            })
                        },
                        "push" => {
                            let mut list = match eval(*caller.clone(), env)? {
                                Value::List(list) => list,
                                _ => vec![],
                            };
                            let value = eval(arguments[0].clone(), env)?;
                            list.push(value);
                            Ok(Value::List(list))
                        },
                        _ => Err(RuntimeError::new(format!("{} is not a method of list", method_name).as_str(), line, column)),
                    }
                }
                _ => Err(RuntimeError::new(format!("{} is not a method of function", method_name).as_str(), line, column)),
            }
        }
        ASTNode::MethodCall { method_name: _, caller: _, arguments: _, builtin: _, line, column } => {
            let method_name = method_name.clone();
            let arguments = match *arguments { ASTNode::FunctionCallArgs{args: arguments, ..} => arguments,
                _ => vec![],
            };
            match method_name.as_str() {
                "to_string" => {
                    let value = eval(arguments[0].clone(), env)?;
                    match value {
                        Value::Number(value) => Ok(Value::String(value.to_string())),
                        _ => Err(RuntimeError::new(format!("{} is not a method of number", method_name).as_str(), line, column)),
                    }
                },
                "round" => {
                    let value = eval(arguments[0].clone(), env)?;
                    match value {
                        Value::Number(value) => Ok(Value::Number(value.round())),
                        _ => Err(RuntimeError::new(format!("{} is not a method of number", method_name).as_str(), line, column)),
                    }
                },
                "push" => {
                    let mut list = match eval(arguments[0].clone(), env)? {
                        Value::List(list) => list,
                        _ => return Err(RuntimeError::new(format!("{} is not a method of list", method_name).as_str(), line, column)),
                    };
                    let value = eval(arguments[1].clone(), env)?;
                    list.push(value);
                    Ok(Value::List(list))
                },
                _ => Ok(Value::Void),
            }
        }
        ASTNode::Literal{value: Value::Number(_), line, column} => {
            let method_name = method_name.clone();
            let arguments = match *arguments {
                ASTNode::FunctionCallArgs{args: arguments, ..} => arguments,
                _ => vec![],
            };
            match method_name.as_str() {
                "to_string" => {
                    let value = eval(arguments[0].clone(), env)?;
                    Ok(match value {
                        Value::Number(value) => Value::String(value.to_string()),
                        _ => Value::Void,
                    })
                },
                "round" => {
                    let value = eval(arguments[0].clone(), env)?;
                    Ok(match value {
                        Value::Number(value) => Value::Number(value.round()),
                        _ => Value::Void,
                    })
                },
                _ => Err(RuntimeError::new(format!("{} is not a method of number", method_name).as_str(), line, column)),
            }
        }
        ASTNode::Variable { ref name, ref value_type, .. } => {
            match method_name.as_str() {
                "to_string" => {
                    let value = eval(*caller.clone(), env)?;
                    Ok(match value {
                        Value::Number(value) => Value::String(value.to_string()),
                        _ => Value::Void,
                    })
                },
                "round" => {
                    let value = eval(*caller.clone(), env)?;
                    Ok(match value {
                        Value::Number(value) => Value::Number(value.round()),
                        _ => Value::Void,
                    })
                },
                "push" => {
                    let variable_info = env.get(&name, value_type.as_ref()).unwrap().clone();
                    let mut variable = match variable_info.value.clone() {
                        Value::List(list) => list,
                        _ => vec![],
                    };
                    let value = match *arguments {
                        ASTNode::FunctionCallArgs{args: arguments, ..} => eval(arguments[0].clone(), env)?,
                        _ => Value::Void,
                    };
                    variable.push(value);
                    let _ = env.set(name.to_string(), Value::List(variable.clone()), variable_info.variable_type.clone(), variable_info.value_type.clone(), false);
                    Ok(Value::List(variable))
                }
                _ => Ok(Value::Void),
            }
        }
        _ => Ok(Value::Void),
    }
}

pub fn method_call_node(method_name: String, caller: Box<ASTNode>, arguments: Box<ASTNode>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
            let mut args_vec = vec![];
            match *arguments {
               ASTNode::FunctionCallArgs{args: arguments, ..} => {
                   args_vec = arguments
               }
               _ => {}
            }
            let caller_name = match *caller {
                ASTNode::Variable { name, value_type: _, .. } => {
                    name
                },
                _ => return Err(RuntimeError::new(format!("Unexpected caller: {:?}", caller).as_str(), line, column)),
            };
            match env.get(&caller_name, None) {
                Some(EnvVariableValueInfo{value, value_type, variable_type}) => {
                    let mut local_env = env.clone();
                    let struct_info = match value_type {
                        ValueType::StructInstance { name: struct_name, ..} => {
                            local_env.get_struct(&struct_name).cloned()
                        },
                        _ => return Err(RuntimeError::new(format!("missing struct: {:?}", value).as_str(), line, column)),
                    };

                    let methods = match struct_info {
                        Some(Value::Struct{ref methods, ..}) => methods,
                        _ => return Err(RuntimeError::new(format!("failed get methods: {:?}", struct_info).as_str(), line, column)),
                    };
                    match methods.get(&method_name) {
                        Some(MethodInfo{arguments: define_arguments, return_type: _, body, is_mut: _}) => {

                            if *variable_type == EnvVariableType::Immutable {
                                return Err(RuntimeError::new(format!("{} is not mutable", caller_name).as_str(), line, column));
                            }
                            if args_vec.len() != define_arguments.len() - 1 {
                                return Err(RuntimeError::new(format!("does not match arguments length: {:?}", args_vec).as_str(), line, column));
                            }
                            local_env.enter_scope(method_name);
                            let _ = local_env.set(
                                "self".to_string(),
                                match struct_info {
                                    Some(Value::Struct{..}) => {
                                        value.clone()
                                    },
                                    _ => return Err(RuntimeError::new(format!("failed struct: {:?}", struct_info).as_str(), line, column)),
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
                                    _ => return Err(RuntimeError::new(format!("failed struct: {:?}", struct_info).as_str(), line, column)),
                                },
                                true
                            );
                            // set struct_instance fields
                            for (field_name, field_value) in match value.clone() {
                                Value::StructInstance { name: _, fields } => fields,
                                _ => return Err(RuntimeError::new(format!("missing struct instance: {:?}", value).as_str(), line, column)),
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
                                if let ASTNode::Variable {name, value_type, ..} = define_arg {
                                    if name == "self" {
                                        continue
                                    }
                                    let arg_value = eval(args_vec[i].clone(), &mut local_env.clone())?;
                                    let _ = local_env.set(name.to_string(), arg_value, EnvVariableType::Immutable, value_type.clone().unwrap_or(ValueType::Any), true);
                                    i += 1;
                                }
                            }
                            let result = eval(body.clone().unwrap(), &mut local_env)?;
                            if let Some(self_value) = local_env.get(&"self".to_string(), None) {
                                if let Value::StructInstance { .. } = self_value.value.clone() {
                                    let set_result = local_env.set(
                                        caller_name.to_string(),
                                        self_value.value.clone(),
                                        variable_type.clone(),
                                        value_type.clone(),
                                        false,
                                    );
                                    if set_result.is_err() {
                                        return Err(RuntimeError::new(format!("Failed to update self in global environment").as_str(), line, column));
                                    }
                                }
                            }
                            env.update_global_env(&local_env);
                            env.leave_scope();
                            Ok(result)
                        },
                        _ => Err(RuntimeError::new(format!("call failed method: {:?}", method_name).as_str(), line, column)),
                    }
                },
                None => Err(RuntimeError::new(format!("missing struct: {:?}", caller_name).as_str(), line, column)),
            }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use crate::evals::evals;

    #[test]
    fn test_to_string_method_call_node() {
        let mut env = Env::new();
        let input = "1.to_string()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::String("1".to_string()));
    }

    #[test]
    fn test_round_method_call_node() {
        let mut env = Env::new();
        let input = "(1.5).round()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse();
        let result = eval(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result, Value::Number(2.into()));
    }
    #[test]
    fn test_push_method_call_node() {
        let mut env = Env::new();
        
        let input = "val mut xs = []\nxs.push(1)\n".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[1], Value::List(vec![Value::Number(1.into())]));
    }

    #[test]
    fn test_push_method_call_node_with_variable() {
        let mut env = Env::new();
        let input = "val mut xs = [1,2]\nval x = 3\nxs.push(x)\n".to_string();
        let tokens = tokenize(&input);
        let builtin = register_builtins(&mut env);
        let mut parser = Parser::new(tokens, builtin);
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[2], Value::List(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())]));
    }
    #[test]
    fn method_chaining_with_round_and_to_string() {
        let mut env = Env::new();
        let input = "fun add(x: number): number {\n return x + 1\n}\n add(1.5).round().to_string()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[1], Value::String("3".to_string()));
    }
}
