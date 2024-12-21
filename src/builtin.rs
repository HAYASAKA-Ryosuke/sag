use crate::environment::Env;
use crate::parser::Value;

pub fn register_builtins(env: &mut Env) {
    env.register_builtin("print".to_string(), |args: Vec<Value>| {
        for arg in args {
            print!("{} ", arg); // Displayを利用
        }
        println!();
        Value::Void
    });
}
