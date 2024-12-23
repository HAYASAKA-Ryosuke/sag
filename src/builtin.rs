use crate::environment::Env;
use crate::parser::Value;

#[cfg(not(target_arch = "wasm32"))]
pub fn register_builtins(env: &mut Env) {
    env.register_builtin("print".to_string(), |args: Vec<Value>| {
        for arg in args {
            print!("{} ", arg);
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
            Value::String(s) => Value::Number(s.len().into()),
            _ => panic!("len function takes a list as an argument"),
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn register_builtins(env: &mut Env) {
    use crate::wasm::CONSOLE_OUTPUT;
    
    env.register_builtin("print".to_string(), |args: Vec<Value>| {
        let output = args.iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<_>>()
            .join(" ");
        
        CONSOLE_OUTPUT.with(|console| {
            let mut console = console.borrow_mut();
            if !console.is_empty() {
                console.push('\n');
            }
            console.push_str(&output);
        });
        
        Value::Void
    });
    
    env.register_builtin("len".to_string(), |args: Vec<Value>| {
        if args.len() != 1 {
            panic!("len function takes exactly one argument");
        }
        match &args[0] {
            Value::List(l) => Value::Number(l.len().into()),
            Value::String(s) => Value::Number(s.len().into()),
            _ => panic!("len function takes a list as an argument"),
        }
    });
}
