use std::rc::Rc;
use std::cell::RefCell;
use crate::value::Value;
use crate::environment::Env;

pub fn variable_node(name: String, env: Rc<RefCell<Env>>) -> Value {
    let value = env.borrow().get(name.to_string(), None);
    if value.is_none() {
        panic!("Variable not found: {:?}", name);
    }
    value.unwrap().value.clone()
}
