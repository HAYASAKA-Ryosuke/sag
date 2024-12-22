use wasm_bindgen::prelude::*;
use crate::tokenizer::tokenize;
use crate::parser::Parser;
use crate::environment::Env;
use crate::eval::evals;
use crate::parser::Value;
use std::cell::RefCell;

thread_local! {
    static CONSOLE_OUTPUT: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn evaluate(input: &str) -> String {
    CONSOLE_OUTPUT.with(|output| output.borrow_mut().clear());

    let tokens = tokenize(&input.to_string());
    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse_lines();
    let mut env = Env::new();
    register_wasm_builtins(&mut env);
    let result = evals(ast_nodes, &mut env);
    
    let output = CONSOLE_OUTPUT.with(|output| output.borrow().clone());
    let result_str = format!("{}", result.last().unwrap_or(&Value::Void));
    format!("__ConsoleOutput__{}__Result__{}", output.trim_end(), result_str)
}

fn register_wasm_builtins(env: &mut Env) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_basic_arithmetic() {
        let result = evaluate("1 + 2");
        assert_eq!(result, "Number(Rational(Plus, Ratio { numer: 3, denom: 1 }))");
    }

    #[test]
    fn test_evaluate_global_variable_and_functions() {
        let input = r#"
val mut z = 3

fun f1 = (x: number, y: number): number {
    z = 2
    val mut d = 3
    z = d = 4
    return x + y + z
}

|2, 0| -> f1
"#;
        let result = evaluate(input);
        assert_eq!(result, "Number(Rational(Plus, Ratio { numer: 6, denom: 1 }))");
    }

    #[test]
    fn test_evaluate_multiple_functions() {
        let input = r#"
val mut z = 3

fun f1 = (x: number, y: number): number {
    z = 2
    val mut d = 3
    z = d = 4
    return x + y + z
}

fun f2 = (x: number, y: number): number {
    return x + y + z
}

fun f3 = (): number {
    return 1
}

fun f4 = (): number {
    return 2 + 3 / 4
}

|2, 0| -> f1
|2, 0| -> f2
|| -> f3
|| -> f4
"#;
        let result = evaluate(input);
        assert_eq!(result, "Number(Rational(Plus, Ratio { numer: 11, denom: 4 }))");
    }
}
