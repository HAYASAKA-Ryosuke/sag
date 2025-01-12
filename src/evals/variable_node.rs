use crate::value::Value;
use crate::environment::{Env, ValueType};

pub fn variable_node(name: String, _value_type: Option<ValueType>, env: &mut Env) -> Value {
    let value = env.get(name.to_string(), None);
    if value.is_none() {
        panic!("Variable not found: {:?}", name)
    } else {
        value.unwrap().value.clone()
    }
}
