use std::sync::{Arc, Mutex};
use crate::value::Value;
use crate::environment::Env;

pub fn variable_node(name: String, env: Arc<Mutex<Env>>) -> Value {
    let value = env.lock().unwrap().get(name.to_string(), None);
    if value.is_none() {
        panic!("Variable not found: {:?}", name);
    }
    value.unwrap().value.clone()
}
