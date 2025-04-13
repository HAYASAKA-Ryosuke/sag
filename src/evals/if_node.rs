use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::Env;
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn if_node(condition: Box<ASTNode>, then: Box<ASTNode>, else_: Option<Box<ASTNode>>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let condition = eval(*condition, env)?;
    match condition {
        Value::Bool(true) => eval(*then, env),
        Value::Bool(false) => {
            if let Some(else_) = else_{
                eval(*else_, env)
            } else {
                Ok(Value::Void)
            }
        }
        _ => Err(RuntimeError::new(format!("Condition must be a boolean: {}", condition).as_str(), line, column)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use crate::evals::evals;

    #[test]
    fn test_if() {
        let mut env = Env::new();
        let input = r#"
        if (1 == 1) {
            1
        } else {
            2
        }
        "#;
        let tokens = tokenize(&input.to_string());
        let builtin = register_builtins(&mut Env::new());
        let asts = Parser::new(tokens, builtin).parse_lines().unwrap();
        let result = evals(asts, &mut env).unwrap();
        assert_eq!(result, vec![
            Value::Number(Fraction::from(1)),
        ]);
    }
}
