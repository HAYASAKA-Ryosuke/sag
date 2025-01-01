use crate::environment::{Env, EnvVariableType, FunctionInfo, ValueType, MethodInfo, EnvVariableValueInfo};
use crate::parser::{ASTNode, Value};
use crate::tokenizer::Token;
use std::collections::HashMap;

pub fn evals(asts: Vec<ASTNode>, env: &mut Env) -> Vec<Value> {
    let mut values = vec![];
    for ast in asts {
        values.push(eval(ast, env));
    }
    values
}

pub fn eval(ast: ASTNode, env: &mut Env) -> Value {
    match ast {
        ASTNode::Literal(value) => value.clone(),
        ASTNode::PrefixOp { op, expr } => {
            let value = eval(*expr, env);
            match (op.clone(), value) {
                (Token::Minus, Value::Number(v)) => Value::Number(-v),
                _ => panic!("Unexpected prefix op: {:?}", op),
            }
        }
        ASTNode::Struct {
            name,
            fields,
            is_public
        } => {
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
                methods: HashMap::new(),
                is_public
            };
            env.register_struct(result.clone());
            result
        }
        ASTNode::Impl {
            base_struct,
            methods,
        } => {
            let mut impl_methods = HashMap::new();
            for method in methods {
                match method {
                    ASTNode::Method {
                        name,
                        arguments,
                        body,
                        return_type,
                    } => {
                        let method_info = MethodInfo {
                            arguments,
                            body: Some(*body),
                            return_type,
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
        ASTNode::MethodCall { method_name, caller, arguments } => {
            let mut args_vec = vec![];
            match *arguments {
               ASTNode::FunctionCallArgs(arguments) => {
                   args_vec = arguments
               }
               _ => {}
            }
            match env.get(caller.clone(), None) {
                Some(EnvVariableValueInfo{value, value_type, variable_type}) => {
                    let local_env = env.clone();
                    let struct_info = match value {
                        Value::StructInstance { name: struct_name, ..} => {
                            local_env.get_struct(struct_name.to_string())
                        },
                        _ => panic!("missing struct: {}", value)
                    };
                    let methods = match struct_info {
                        Some(Value::Struct{methods, ..}) => methods,
                        _ => panic!("failed get methods")
                    };

                    match methods.get(&method_name) {
                        Some(MethodInfo{arguments: define_arguments, return_type, body}) => {

                            // self分加味
                            if (args_vec.len() + 1) != define_arguments.len() {
                                panic!("does not match arguments length");
                            }
                            let mut local_env = env.clone();
                            local_env.enter_scope(value.to_string());
                            let _ = local_env.set(
                                "self".to_string(),
                                match struct_info {
                                    Some(Value::Struct{name, fields, methods, is_public}) => {
                                        value.clone()
                                    },
                                    _ => panic!("failed struct")
                                },
                                EnvVariableType::Mutable,
                                match struct_info {
                                    Some(Value::Struct{name, fields, methods, is_public}) => {
                                        let mut field_types = HashMap::new();
                                        for (field_name, field_value) in fields {
                                            field_types.insert(field_name.to_string(), field_value.value_type());
                                        }
                                        ValueType::Struct{name: name.to_string(), fields: field_types, is_public: is_public.clone()}
                                    },
                                    _ => panic!("failed struct")
                                },
                                true
                            );
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
                                if let Value::StructInstance { name, fields } = self_value.value.clone() {
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
        ASTNode::StructInstance {
            name,
            fields,
        } => {
            let mut struct_fields = HashMap::new();
            for (field_name, field_value) in fields {
                struct_fields.insert(field_name, eval(field_value, env));
            }
            Value::StructInstance {
                name,
                fields: struct_fields,
            }
        }
        ASTNode::StructFieldAssign { instance, field_name: updated_field_name, value: updated_value_ast } => {
            match *instance {
                ASTNode::StructFieldAccess { instance, field_name: _  } => {
                    match *instance {
                        ASTNode::Variable { name: variable_name, value_type } => {
                            match value_type {
                                Some(ValueType::Struct{name, fields, ..}) if variable_name == "self" => {
                                    let obj = env.get(variable_name.to_string(), None);
                                    if obj.is_none() {
                                        panic!("Variable not found: {:?}", variable_name);
                                    }
                                    let mut struct_fields = HashMap::new();
                                    match obj.unwrap().value.clone() {
                                        Value::StructInstance { .. } => {
                                            let instance_value = obj.unwrap().value.clone();
                                            println!("instance value: {:?}", instance_value);
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
                                        Value::Struct { name: _, fields: obj_fields, methods: obj_methods, .. } => {
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
        ASTNode::StructFieldAccess { instance, field_name } => {
            let struct_obj = match *instance {
                ASTNode::Variable { name: variable_name, value_type } => {
                    match value_type {
                        Some(ValueType::Struct { name, fields, is_public }) if variable_name == "self" => {
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
                Value::Struct {name: obj_name, fields, ..} => {
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
        ASTNode::Function {
            name,
            arguments,
            body,
            return_type,
        } => {
            let function_info = FunctionInfo {
                arguments,
                body: Some(*body),
                return_type,
                builtin: None,
            };
            env.register_function(name, function_info);
            Value::Function
        }
        ASTNode::Lambda { arguments, body } => Value::Lambda {
            arguments,
            body: body.clone(),
            env: env.clone(),
        },
        ASTNode::Block(statements) => {
            for statement in statements {
                if let Value::Return(v) = eval(statement, env) {
                    return *v;
                }
            }
            Value::Void
        }
        ASTNode::Return(value) => {
            let result = eval(*value, env);
            Value::Return(Box::new(result))
        }
        ASTNode::Eq { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l == r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::Gte { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l >= r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::Gt { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l > r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::Lte { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l <= r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::Lt { left, right } => {
            let left_value = eval(*left, env);
            let right_value = eval(*right, env);
            match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Bool(l < r),
                _ => panic!("Unsupported operation"),
            }
        }
        ASTNode::If {
            condition,
            then,
            else_,
            value_type: _,
        } => {
            let condition = eval(*condition, env);
            match condition {
                Value::Bool(true) => eval(*then, env),
                Value::Bool(false) => {
                    if let Some(else_) = else_{
                        eval(*else_, env)
                    } else {
                        Value::Void
                    }
                }
                _ => panic!("Unexpected value type"),
            }
        }
        ASTNode::Assign {
            name,
            value,
            variable_type,
            value_type: _,
            is_new,
        } => {
            let value = eval(*value, env);
            let value_type = match value {
                Value::Number(_) => ValueType::Number,
                Value::String(_) => ValueType::String,
                Value::Bool(_) => ValueType::Bool,
                Value::Function => ValueType::Function,
                Value::Lambda { .. } => ValueType::Lambda,
                Value::Void => ValueType::Void,
                Value::Return(ref value) => {
                    if let Value::Void = **value {
                        ValueType::Void
                    } else {
                        value.value_type()
                    }
                },
                Value::List(ref elements) => {
                    if elements.len() == 0 {
                        ValueType::List(Box::new(ValueType::Any))
                    } else {
                        let first_element = elements.first().unwrap();
                        let value_type = first_element.value_type();
                        for e in elements {
                            if e.value_type() != value_type {
                                panic!("List value type mismatch");
                            }
                        }
                        ValueType::List(Box::new(value_type))
                    }
                },
                Value::StructInstance { ref name, fields: ref instance_fields } => {
                    match env.get_struct(name.to_string()) {
                        Some(Value::Struct { name: _, fields, is_public: _, methods }) => {
                            for (field_name, value_type) in instance_fields {
                                if !fields.contains_key(&field_name.to_string()) {
                                    panic!("Struct field not found: {:?}", field_name);
                                }
                                if fields.get(&field_name.to_string()).unwrap().value_type() != value_type.value_type() {
                                    panic!("Struct field type mismatch: {:?}", field_name);
                                }
                            }
                        },
                        _ => panic!("Unexpected value type"),
                    };
                    let mut field_types = HashMap::new();
                    for (field_name, field_value) in instance_fields {
                        field_types.insert(field_name.clone(), field_value.value_type());
                    }
                    ValueType::StructInstance { name: name.to_string(), fields: field_types }
                },
                _ => panic!("Unsupported value type, {:?}", value),
            };
            let result = env.set(
                name.to_string(),
                value.clone(),
                variable_type,
                value_type,
                is_new,
            );
            if result.is_err() {
                panic!("{}", result.unwrap_err());
            }
            value
        }
        ASTNode::LambdaCall { lambda, arguments } => {
            let mut params_vec = vec![];
            let lambda = match *lambda {
                ASTNode::Lambda { arguments, body } => (arguments, body),
                _ => panic!("Unexpected value type"),
            };
            for arg in &lambda.0 {
                params_vec.push(match arg {
                    ASTNode::Variable { name, value_type } => (name, value_type),
                    _ => panic!("illigal param: {:?}", lambda.0),
                });
            }

            let mut args_vec = vec![];

            for arg in arguments {
                match arg {
                    ASTNode::FunctionCallArgs(arguments) => {
                        args_vec = arguments;
                    }
                    _ => {
                        args_vec.push(arg);
                    }
                }
            }
            if args_vec.len() != lambda.0.len() {
                panic!("does not match arguments length");
            }

            let mut local_env = env.clone();

            local_env.enter_scope("lambda".to_string());

            for (param, arg) in params_vec.iter().zip(&args_vec) {
                let arg_value = eval(arg.clone(), env);
                let name = param.0.to_string();
                let value_type = param.1.clone();
                let _ = local_env.set(
                    name,
                    arg_value,
                    EnvVariableType::Immutable,
                    value_type.unwrap_or(ValueType::Any),
                    true,
                );
            }

            let result = eval(*lambda.1, &mut local_env);

            env.update_global_env(&local_env);

            env.leave_scope();
            result
        }
        ASTNode::FunctionCall { name, arguments } => {
            if env.get_function(name.to_string()).is_some()
                || env.get_builtin(name.to_string()).is_some()
            {
                let function = match env.get_function(name.to_string()) {
                    Some(function) => function.clone(),
                    None => {
                        let builtin = env.get_builtin(name.to_string());
                        if builtin.is_some() {
                            builtin.unwrap().clone()
                        } else {
                            panic!("Function is missing: {:?}", name)
                        }
                    }
                };
                let mut params_vec = vec![];
                for arg in &function.arguments {
                    params_vec.push(match arg {
                        ASTNode::Variable { name, value_type } => (name, value_type),
                        _ => panic!("illigal param: {:?}", function.arguments),
                    });
                }

                let args_vec = match *arguments {
                    ASTNode::FunctionCallArgs(arguments) => arguments,
                    _ => panic!("illigal arguments: {:?}", arguments),
                };

                if let Some(func) = function.builtin {
                    let result = func(args_vec.iter().map(|arg| eval(arg.clone(), env)).collect());
                    return result;
                };

                if args_vec.len() != function.arguments.len() {
                    panic!("does not match arguments length");
                }

                let mut local_env = env.clone();

                local_env.enter_scope(name.to_string());

                for (param, arg) in params_vec.iter().zip(&args_vec) {
                    let arg_value = eval(arg.clone(), env);
                    let name = param.0.to_string();
                    let value_type = param.1.clone();
                    let _ = local_env.set(
                        name,
                        arg_value,
                        EnvVariableType::Immutable,
                        value_type.unwrap_or(ValueType::Any),
                        true,
                    );
                }


                let result = eval(function.body.unwrap(), &mut local_env);
                env.update_global_env(&local_env);

                env.leave_scope();
                if let Value::Return(v) = result {
                    *v
                } else {
                    result
                }
            } else if env
                .get(name.to_string(), Some(&ValueType::Lambda))
                .is_some()
            {
                let lambda = match env.get(name.to_string(), None).unwrap().value.clone() {
                    Value::Lambda {
                        arguments,
                        body,
                        env: lambda_env,
                    } => (arguments, body, lambda_env),
                    _ => panic!("Unexpected value type"),
                };

                let mut params_vec = vec![];
                for arg in &lambda.0 {
                    params_vec.push(match arg {
                        ASTNode::Variable { name, value_type } => (name, value_type),
                        _ => panic!("illigal param: {:?}", lambda.0),
                    });
                }

                let args_vec = match *arguments {
                    ASTNode::FunctionCallArgs(arguments) => arguments,
                    _ => panic!("illigal arguments: {:?}", arguments),
                };

                if args_vec.len() != lambda.0.len() {
                    panic!("does not match arguments length");
                }

                let mut local_env = env.clone();

                local_env.enter_scope(name.to_string());

                for (param, arg) in params_vec.iter().zip(&args_vec) {
                    let arg_value = eval(arg.clone(), env);
                    let name = param.0.to_string();
                    let value_type = param.1.clone();
                    let _ = local_env.set(
                        name,
                        arg_value,
                        EnvVariableType::Immutable,
                        value_type.unwrap_or(ValueType::Any),
                        true,
                    );
                }

                let result = eval(*lambda.1, &mut local_env);

                env.update_global_env(&local_env);

                env.leave_scope();
                result
            } else {
                panic!("Function is missing: {:?}", name)
            }
        }
        ASTNode::Variable {
            name,
            value_type: _,
        } => {
            let value = env.get(name.to_string(), None);
            if value.is_none() {
                panic!("Variable not found: {:?}", name);
            }
            value.unwrap().value.clone()
        }
        ASTNode::BinaryOp { left, op, right } => {
            let left_val = eval(*left, env);
            let right_val = eval(*right, env);

            match (left_val.clone(), right_val.clone(), op.clone()) {
                (Value::String(l), Value::String(r), Token::Plus) => Value::String(l + &r),
                (Value::Number(l), Value::Number(r), Token::Plus) => Value::Number(l + r),
                (Value::Number(l), Value::Number(r), Token::Mul) => Value::Number(l * r),
                (Value::Number(l), Value::Number(r), Token::Div) => Value::Number(l / r),
                _ => panic!("Unsupported operation: {:?} {:?} {:?}", left_val.clone(), op, right_val.clone()),
            }
        }
        _ => panic!("Unsupported ast node: {:?}", ast),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parser::Parser;
    use crate::environment::EnvVariableType;
    use crate::builtin::register_builtins;
    use fraction::Fraction;

    #[test]
    fn test_four_basic_arithmetic_operations() {
        let mut env = Env::new();
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::PrefixOp {
                op: Token::Minus,
                expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            }),
            op: Token::Plus,
            right: Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                op: Token::Mul,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            }),
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
    }
    #[test]
    fn test_assign() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
        assert_eq!(
            Value::Number(Fraction::from(5)),
            env.get("x".to_string(), None).unwrap().value
        );
        assert_eq!(
            EnvVariableType::Mutable,
            env.get("x".to_string(), None).unwrap().variable_type
        );
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: false,
        };
        assert_eq!(Value::Number(Fraction::from(5)), eval(ast, &mut env));
        assert_eq!(
            Value::Number(Fraction::from(5)),
            env.get("x".to_string(), None).unwrap().value
        );
        assert_eq!(
            EnvVariableType::Immutable,
            env.get("x".to_string(), None).unwrap().variable_type
        );
    }
    #[test]
    fn test_assign_expression_value() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "y".to_string(),
            value: Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(10)))),
                op: Token::Plus,
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(20)))),
            }),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        assert_eq!(Value::Number(Fraction::from(30)), eval(ast, &mut env));
        assert_eq!(
            env.get("y".to_string(), None).unwrap().value,
            Value::Number(Fraction::from(30))
        );
    }
    #[test]
    fn test_assign_overwrite_mutable_variable() {
        let mut env = Env::new();

        let ast1 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(50)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(ast1, &mut env);

        // 再代入
        let ast2 = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(100)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: false,
        };

        // 環境に新しい値が登録されていること
        assert_eq!(eval(ast2, &mut env), Value::Number(Fraction::from(100)));
        assert_eq!(
            env.get("z".to_string(), None).unwrap().value,
            Value::Number(Fraction::from(100))
        );
    }
    #[test]
    #[should_panic(expected = "Cannot reassign to immutable variable")]
    fn test_assign_to_immutable_variable() {
        let mut env = Env::new();

        // Immutable 変数の初期値を設定
        let ast1 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(200)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(ast1, &mut env);

        // 再代入しようとしてエラー
        let ast2 = ASTNode::Assign {
            name: "w".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(300)))),
            variable_type: EnvVariableType::Immutable,
            value_type: ValueType::Number,
            is_new: false,
        };
        eval(ast2, &mut env);
    }
    #[test]
    fn test_register_function_and_function_call() {
        let mut env = Env::new();
        let ast = ASTNode::Function {
            name: "foo".into(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![ASTNode::Return(Box::new(
                ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "y".into(),
                        value_type: Some(ValueType::Number),
                    }),
                },
            ))])),
            return_type: ValueType::Number,
        };
        eval(ast, &mut env);
        let ast = ASTNode::FunctionCall {
            name: "foo".into(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(1))),
                ASTNode::Literal(Value::Number(Fraction::from(2))),
            ])),
        };
        let result = eval(ast, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(3)));
    }

    #[test]
    #[should_panic(expected = "Unexpected prefix op: Plus")]
    fn test_unsupported_prefix_operation() {
        let mut env = Env::new();
        let ast = ASTNode::PrefixOp {
            op: Token::Plus,
            expr: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
        };
        eval(ast, &mut env);
    }

    #[test]
    #[should_panic(expected = "Unsupported operation")]
    fn test_unsupported_binary_operation() {
        let mut env = Env::new();
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::Literal(Value::String("hello".to_string()))),
            op: Token::Mul,
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(5)))),
        };
        eval(ast, &mut env);
    }

    #[test]
    fn test_list() {
        let mut env = Env::new();
        let ast = ASTNode::Assign {
            name: "x".to_string(),
            value: Box::new(ASTNode::Literal(Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::List(Box::new(ValueType::Number)),
            is_new: true,
        };
        assert_eq!(
            Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]),
            eval(ast, &mut env)
        );
        assert_eq!(
            Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
            ]),
            env.get("x".to_string(), None).unwrap().value
        );
    }

    #[test]
    #[should_panic(expected = "does not match arguments length")]
    fn test_function_call_argument_mismatch() {
        let mut env = Env::new();
        let ast_function = ASTNode::Function {
            name: "bar".to_string(),
            arguments: vec![ASTNode::Variable {
                name: "x".into(),
                value_type: Some(ValueType::Number),
            }],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::Variable {
                name: "x".into(),
                value_type: Some(ValueType::Number),
            }))),
            return_type: ValueType::Number,
        };
        eval(ast_function, &mut env);

        // 引数の数が合わない関数呼び出し
        let ast_call = ASTNode::FunctionCall {
            name: "bar".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(5))),
                ASTNode::Literal(Value::Number(Fraction::from(10))), // 余分な引数
            ])),
        };
        eval(ast_call, &mut env);
    }

    #[test]
    fn test_scope_management_in_function() {
        let mut env = Env::new();

        // 関数定義
        let ast_function = ASTNode::Function {
            name: "add_and_return".to_string(),
            arguments: vec![ASTNode::Variable {
                name: "a".into(),
                value_type: Some(ValueType::Number),
            }],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "local_var".into(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(10)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: true,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "a".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "local_var".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };

        eval(ast_function, &mut env);

        // 関数呼び出し
        let ast_call = ASTNode::FunctionCall {
            name: "add_and_return".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![ASTNode::Literal(
                Value::Number(Fraction::from(5)),
            )])),
        };

        // 結果の確認
        let result = eval(ast_call, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(15)));

        // スコープ外でローカル変数が見つからないことを確認
        let local_var_check = env.get("local_var".to_string(), None);
        assert!(local_var_check.is_none());
    }

    #[test]
    fn test_scope_and_global_variable() {
        let mut env = Env::new();

        // グローバル変数 z を定義
        let global_z = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(global_z, &mut env);

        // f1 関数の定義
        let f1 = ASTNode::Function {
            name: "f1".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Assign {
                    name: "d".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,

                    is_new: true,
                },
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Assign {
                        name: "d".to_string(),
                        value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4)))),
                        variable_type: EnvVariableType::Mutable,
                        value_type: ValueType::Number,
                        is_new: false,
                    }),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number),
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Variable {
                            name: "y".into(),
                            value_type: Some(ValueType::Number),
                        }),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "z".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };
        eval(f1, &mut env);

        // f2 関数の定義
        let f2 = ASTNode::Function {
            name: "f2".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::Variable {
                        name: "x".into(),
                        value_type: Some(ValueType::Number),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "y".into(),
                        value_type: Some(ValueType::Number),
                    }),
                }),
                op: Token::Plus,
                right: Box::new(ASTNode::Variable {
                    name: "z".into(),
                    value_type: Some(ValueType::Number),
                }),
            }))),
            return_type: ValueType::Number,
        };
        eval(f2, &mut env);

        // f3 関数の定義
        let f3 = ASTNode::Function {
            name: "f3".to_string(),
            arguments: vec![],
            body: Box::new(ASTNode::Return(Box::new(ASTNode::Literal(Value::Number(
                Fraction::from(1),
            ))))),
            return_type: ValueType::Number,
        };
        eval(f3, &mut env);

        // f1 の呼び出し
        let call_f1 = ASTNode::FunctionCall {
            name: "f1".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result_f1 = eval(call_f1, &mut env);
        assert_eq!(result_f1, Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6

        // f2 の呼び出し (f1 の影響で z = 4)
        let call_f2 = ASTNode::FunctionCall {
            name: "f2".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result_f2 = eval(call_f2, &mut env);
        assert_eq!(result_f2, Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6

        // f3 の呼び出し
        let call_f3 = ASTNode::FunctionCall {
            name: "f3".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![])),
        };
        let result_f3 = eval(call_f3, &mut env);
        assert_eq!(result_f3, Value::Number(Fraction::from(1)));
    }

    #[test]
    fn test_global_variable_and_functions() {
        let mut env = Env::new();

        // グローバル変数の定義
        let global_z = ASTNode::Assign {
            name: "z".to_string(),
            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
            variable_type: EnvVariableType::Mutable,
            value_type: ValueType::Number,
            is_new: true,
        };
        eval(global_z, &mut env);

        // f1関数の定義
        let f1 = ASTNode::Function {
            name: "f1".to_string(),
            arguments: vec![
                ASTNode::Variable {
                    name: "x".into(),
                    value_type: Some(ValueType::Number),
                },
                ASTNode::Variable {
                    name: "y".into(),
                    value_type: Some(ValueType::Number),
                },
            ],
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Assign {
                    name: "d".to_string(),
                    value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: true,
                },
                ASTNode::Assign {
                    name: "z".to_string(),
                    value: Box::new(ASTNode::Assign {
                        name: "d".to_string(),
                        value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(4)))),
                        variable_type: EnvVariableType::Mutable,
                        value_type: ValueType::Number,
                        is_new: false,
                    }),
                    variable_type: EnvVariableType::Mutable,
                    value_type: ValueType::Number,
                    is_new: false,
                },
                ASTNode::Return(Box::new(ASTNode::BinaryOp {
                    left: Box::new(ASTNode::BinaryOp {
                        left: Box::new(ASTNode::Variable {
                            name: "x".into(),
                            value_type: Some(ValueType::Number),
                        }),
                        op: Token::Plus,
                        right: Box::new(ASTNode::Variable {
                            name: "y".into(),
                            value_type: Some(ValueType::Number),
                        }),
                    }),
                    op: Token::Plus,
                    right: Box::new(ASTNode::Variable {
                        name: "z".into(),
                        value_type: Some(ValueType::Number),
                    }),
                })),
            ])),
            return_type: ValueType::Number,
        };
        eval(f1, &mut env);

        // f1の呼び出し
        let call_f1 = ASTNode::FunctionCall {
            name: "f1".to_string(),
            arguments: Box::new(ASTNode::FunctionCallArgs(vec![
                ASTNode::Literal(Value::Number(Fraction::from(2))),
                ASTNode::Literal(Value::Number(Fraction::from(0))),
            ])),
        };
        let result = eval(call_f1, &mut env);
        assert_eq!(result, Value::Number(Fraction::from(6))); // 2 + 0 + 4 = 6
    }

    #[test]
    fn test_if() {
        let mut env = Env::new();
        let ast = ASTNode::If {
            condition: Box::new(ASTNode::Eq {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
            }),
            then: Box::new(ASTNode::Block(vec![
                ASTNode::Literal(Value::Number(Fraction::from(1)))
            ])),
            else_: None,
            value_type: ValueType::Void
        };
        assert_eq!(Value::Void, eval(ast, &mut env));
    }

    #[test]
    fn test_if_return() {
        let mut env = Env::new();
        let ast = ASTNode::If {
            condition: Box::new(ASTNode::Eq {
                left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
                right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
            }),
            then: Box::new(ASTNode::Block(vec![
                ASTNode::Literal(Value::Number(Fraction::from(1)))
            ])),
            else_: None,
            value_type: ValueType::Void
        };
        assert_eq!(Value::Void, eval(ast, &mut env));
    }

    #[test]
    fn test_comparison_operations() {
        let mut env = Env::new();
        let ast = ASTNode::Eq {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env));

        let ast = ASTNode::Gte {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env));

        let ast = ASTNode::Gt {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        };
        assert_eq!(Value::Bool(false), eval(ast, &mut env));

        let ast = ASTNode::Lte {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env));

        let ast = ASTNode::Lt {
            left: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1)))),
            right: Box::new(ASTNode::Literal(Value::Number(Fraction::from(1))))
        };
        assert_eq!(Value::Bool(false), eval(ast, &mut env));

    }
    #[test]
    fn test_struct() {
        let mut env = Env::new();
        let ast = ASTNode::Struct {
            name: "Point".into(),
            is_public: true,
            fields: HashMap::from_iter(vec![
                ("x".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: true }),
                ("y".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: false
                })
            ])
        };
        let result = eval(ast, &mut env);
        assert_eq!(
            result,
            Value::Struct {
                name: "Point".into(),
                is_public: true,
                methods: HashMap::new(),
                fields: HashMap::from_iter(vec![
                    ("x".into(), Value::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    }),
                    ("y".into(), Value::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    })
                ])
            }
        );
        assert_eq!(env.get_struct("Point".to_string()).is_some(), true);
        assert_eq!(env.get_struct("DummuStruct".to_string()).is_some(), false);
    }

    #[test]
    fn test_assign_struct() {
        let mut env = Env::new();
        let ast = vec![
            ASTNode::Struct {
                name: "Point".into(),
                fields: HashMap::from_iter(vec![
                    ("y".into(), ASTNode::StructField {
                        value_type: ValueType::String,
                        is_public: false
                    }),
                    ("x".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    })
                ]),
                is_public: false
            },
            ASTNode::Assign {
                name: "point".into(),
                value: Box::new(ASTNode::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                        ("y".into(), ASTNode::Literal(Value::String("hello".into())))
                    ])
                }),
                variable_type: EnvVariableType::Immutable,
                value_type: ValueType::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ValueType::Number),
                        ("y".into(), ValueType::String)
                    ])
                },
                is_new: true
            }
        ];
        let result = evals(ast, &mut env);
        assert_eq!(
            result,
            vec![
                Value::Struct {
                    name: "Point".into(),
                    is_public: false,
                    fields: HashMap::from_iter(vec![
                        ("y".into(), Value::StructField {
                            value_type: ValueType::String,
                            is_public: false
                        }),
                        ("x".into(), Value::StructField {
                            value_type: ValueType::Number,
                            is_public: false
                        })
                    ]),
                    methods: HashMap::new()
                },
                Value::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), Value::Number(Fraction::from(1))),
                        ("y".into(), Value::String("hello".into()))
                    ])
                }
            ]
        );
    }
    #[test]
    fn test_struct_access() {
        let asts = vec![
            ASTNode::Struct {
                name: "Point".into(),
                is_public: true,
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
            },
            ASTNode::Assign {
                name: "point".into(),
                variable_type: EnvVariableType::Mutable,
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
        ];
        let mut env = Env::new();
        let result = evals(asts, &mut env);
        assert_eq!(result[4], Value::Number(Fraction::from(3)));
    }

    #[test]
    #[should_panic(expected = "Struct field type mismatch: point.x:Number = String")]
    fn test_struct_other_type_assign() {
        let asts = vec![
            ASTNode::Struct {
                name: "Point".into(),
                is_public: true,
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
            },
            ASTNode::Assign {
                name: "point".into(),
                variable_type: EnvVariableType::Mutable,
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
                value: Box::new(ASTNode::Literal(Value::String("hello".into()))),
                field_name: "x".into()
            },
        ];
        let mut env = Env::new();
        evals(asts, &mut env);
    }

    #[test]
    fn test_struct_test() {
        let input = r#"
struct Point {
  x: number,
  y: number
}

impl Point {
  fun move(dx: number, dy: number) {
      self.x = self.x + dx
      self.y = self.y + dy
  }
}

impl Point {
  fun clear() {
      self.x = 0
      self.y = 0
  }
}

val x = 8
val y = 3
val mut point = Point{x: x, y: y}
point.move(5, 2)
point.clear()
"#;

    let tokens = tokenize(&input.to_string());
    let asts = Parser::new(tokens.to_vec()).parse_lines();
    let mut env = Env::new();
    register_builtins(&mut env);
    let result = evals(asts, &mut env);
    let base_struct = Value::Struct {
        name: "Point".into(),
        fields: HashMap::from_iter(vec![
            ("y".into(), Value::StructField{
                value_type: ValueType::Number,
                is_public: false
            }),
            ("x".into(), Value::StructField{
                value_type: ValueType::Number,
                is_public: false
            })
        ]),
        methods: HashMap::new(),
        is_public: false
    };
    let base_struct_type = ValueType::Struct {
        name: "Point".into(),
        fields: HashMap::from_iter(vec![
            ("y".into(), ValueType::StructField{
                value_type: Box::new(ValueType::Number),
                is_public: false
            }),
            ("x".into(), ValueType::StructField{
                value_type: Box::new(ValueType::Number),
                is_public: false
            })
        ]),
        is_public: false
    };
    assert_eq!(result, vec![
        base_struct.clone(),
        Value::Impl {
            base_struct: base_struct_type.clone(),
            methods: HashMap::from_iter(vec![
                ("move".into(), MethodInfo{
                    arguments: vec![
                        ASTNode::Variable{
                            name: "self".into(),
                            value_type: None
                        },
                        ASTNode::Variable{
                            name: "dx".into(),
                            value_type: Some(ValueType::Number)
                        },
                        ASTNode::Variable{
                            name: "dy".into(),
                            value_type: Some(ValueType::Number)
                        },
                    ],
                    return_type: ValueType::Void,
                    body: Some(ASTNode::Block(vec![
                            ASTNode::StructFieldAssign {
                                instance: Box::new(ASTNode::StructFieldAccess {
                                    instance: Box::new(ASTNode::Variable {
                                        name: "self".into(),
                                        value_type: Some(base_struct_type.clone()),
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
                            },
                            ASTNode::StructFieldAssign {
                                instance: Box::new(ASTNode::StructFieldAccess {
                                    instance: Box::new(ASTNode::Variable {
                                        name: "self".into(),
                                        value_type: Some(base_struct_type.clone()),
                                    }),
                                    field_name: "y".into()
                                }),
                                field_name: "y".into(),
                                value: Box::new(ASTNode::BinaryOp {
                                    left: Box::new(ASTNode::StructFieldAccess {
                                        instance: Box::new(ASTNode::Variable {
                                            name: "self".into(),
                                            value_type: Some(base_struct_type.clone())
                                        }),
                                        field_name: "y".into()
                                    }),
                                    op: Token::Plus,
                                    right: Box::new(ASTNode::Variable {
                                        name: "dy".into(),
                                        value_type: Some(ValueType::Number)
                                    })
                                })
                            },
                    ]))
                })
            ])
        },
        Value::Impl {
            base_struct: base_struct_type.clone(),
            methods: HashMap::from_iter(vec![
                ("clear".into(), MethodInfo{
                    arguments: vec![ASTNode::Variable{name: "self".into(), value_type: None}],
                    return_type: ValueType::Void,
                    body: Some(ASTNode::Block(vec![
                        ASTNode::StructFieldAssign {
                            instance: Box::new(ASTNode::StructFieldAccess {
                                instance:  Box::new(ASTNode::Variable {
                                    name: "self".into(),
                                    value_type: Some(base_struct_type.clone())
                                }),
                                field_name: "x".into()
                            }),
                            field_name: "x".into(),
                            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(0))))
                        },
                        ASTNode::StructFieldAssign {
                            instance: Box::new(ASTNode::StructFieldAccess {
                                instance:  Box::new(ASTNode::Variable {
                                    name: "self".into(),
                                    value_type: Some(base_struct_type.clone())
                                }),
                                field_name: "y".into()
                            }),
                            field_name: "y".into(),
                            value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(0))))
                        },
                    ]))
                })
            ])
        },
        Value::Number(Fraction::from(8)),
        Value::Number(Fraction::from(3)),
        Value::StructInstance {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("x".into(), Value::Number(Fraction::from(8))),
                ("y".into(), Value::Number(Fraction::from(3))),
            ])
        },
        Value::Void,
        Value::Void,
    ]);
    }
}
