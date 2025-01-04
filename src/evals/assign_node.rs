use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, EnvVariableType};
use crate::evals::eval;

pub fn assign_node(name: String, value: Box<ASTNode>, variable_type: EnvVariableType, is_new: bool, env: Arc<Mutex<Env>>) -> Value {
    let value = eval(*value, env.clone());
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
            match env.lock().unwrap().get_struct(name.to_string()) {
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
    let result = env.lock().unwrap().set(
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
