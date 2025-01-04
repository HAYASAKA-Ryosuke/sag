use crate::environment::Env;
use crate::value::Value;
use fraction::Fraction;

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
    env.register_builtin("range".to_string(), |args: Vec<Value>| {
        if let [Value::Number(start), Value::Number(end)] = args.as_slice() {
            Value::List(((*start.numer().unwrap() as i64)..(*end.numer().unwrap() as i64)).map(|x| Value::Number(Fraction::from(x))).collect())
        } else if let [Value::Number(end)] = args.as_slice() {
            Value::List((0..(*end.numer().unwrap() as i64)).map(|x| Value::Number(Fraction::from(x))).collect())
        } else if let [Value::Number(start), Value::Number(end), Value::Number(step)] = args.as_slice() {
            Value::List(((*start.numer().unwrap() as i64..*end.numer().unwrap() as i64).step_by(*step.numer().unwrap() as usize)).map(|x| Value::Number(Fraction::from(x))).collect())
        } else {
            panic!("range function takes 1, 2 or 3 arguments")
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn register_builtins(env: &mut Env) {
    use crate::wasm::CONSOLE_OUTPUT;

    env.register_builtin("print".to_string(), |args: Vec<Value>| {
        let output = args
            .iter()
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
