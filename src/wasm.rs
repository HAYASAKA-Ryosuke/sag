use wasm_bindgen::prelude::*;
use crate::tokenizer::tokenize;
use crate::parser::Parser;
use crate::environment::Env;
use crate::eval::evals;

#[wasm_bindgen]
pub fn evaluate(input: &str) -> String {
    let tokens = tokenize(&input.to_string());
    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse_lines();
    let mut env = Env::new();
    let result = evals(ast_nodes, &mut env);
    format!("{:?}", result.last().unwrap())
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

args(2, 0) -> f1
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

args(2, 0) -> f1
args(2, 0) -> f2
args() -> f3
args() -> f4
"#;
        let result = evaluate(input);
        assert_eq!(result, "Number(Rational(Plus, Ratio { numer: 11, denom: 4 }))");
    }
}
