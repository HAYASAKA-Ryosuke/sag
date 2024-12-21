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
    env.register_builtin("len".to_string(), |args: Vec<Value>| {
        if args.len() != 1 {
            panic!("len function takes exactly one argument");
        }
        match &args[0] {
            Value::List(l) => Value::Number(l.len().into()),
            Value::Str(s) => Value::Number(s.len().into()),
            _ => panic!("len function takes a list as an argument"),
        }
    });
}
