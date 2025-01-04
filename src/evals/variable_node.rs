use crate::parser::{Value};
use crate::environment::Env;

pub fn variable_node(name: String, env: &mut Env) -> Value {
    let value = env.get(name.to_string(), None);
    if value.is_none() {
        panic!("Variable not found: {:?}", name);
    }
    value.unwrap().value.clone()
}
