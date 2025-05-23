use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, EnvVariableType};
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn for_node(variable: String, iterable: Box<ASTNode>, body: Box<ASTNode>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let iterable = eval(*iterable, env)?;
    match iterable {
        Value::List(values) => {
            let scope_name = format!("for-{}", variable.clone());
            for value in values {
                env.enter_scope(scope_name.clone());
                let _ = env.set(variable.clone(), value.clone(), EnvVariableType::Immutable, value.value_type(), true);
                let result = eval(*body.clone(), env)?;
                if let Value::Return(_) = result {
                    env.leave_scope();
                    return Ok(result);
                }
                if let Value::Break = result {
                    env.leave_scope();
                    return Ok(Value::Void);
                }
                if let Value::Continue = result {
                    env.leave_scope();
                    continue;
                }
            }
            env.leave_scope();
            Ok(Value::Void)
        }
        _ => Err(RuntimeError::new(format!("Unexpected iterable: {:?}", iterable).as_str(), line, column)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::evals::evals;
    use crate::builtin::register_builtins;

    #[test]
    fn test_for() {
        let input = r#"
        val mut sum = 0
        for i in [0, 1, 2, 3] {
            sum = sum + i
        }
        sum
        "#;
        let tokens = tokenize(&input.to_string());
        let builtin = register_builtins(&mut Env::new());
        let asts = Parser::new(tokens, builtin).parse_lines().unwrap();
        let mut env = Env::new();
        register_builtins(&mut env);
        let result = evals(asts, &mut env).unwrap();
        assert_eq!(result, vec![
            Value::Number(Fraction::from(0)),
            Value::Void,
            Value::Number(Fraction::from(6)),
        ]);
    }

    #[test]
    fn test_for_break() {
        let input = r#"
        val mut value = 0
        for i in [0, 1, 2, 3] {
            if (i == 2) {
                value = i
                break
            }
        }
        value
        "#;
        let tokens = tokenize(&input.to_string());
        let builtin = register_builtins(&mut Env::new());
        let asts = Parser::new(tokens, builtin).parse_lines().unwrap();
        let mut env = Env::new();
        register_builtins(&mut env);
        let result = evals(asts, &mut env).unwrap();
        assert_eq!(result, vec![
            Value::Number(Fraction::from(0)),
            Value::Void,
            Value::Number(Fraction::from(2)),
        ]);
    }
    #[test]
    fn test_for_continue() {
        let input = r#"
        val mut value = 0
        for i in range(10) {
            if (i > 2) {
                continue
            }
            value = i
        }
        value
        "#;
        let tokens = tokenize(&input.to_string());
        let builtin = register_builtins(&mut Env::new());
        let asts = Parser::new(tokens, builtin).parse_lines().unwrap();
        let mut env = Env::new();
        register_builtins(&mut env);
        let result = evals(asts, &mut env).unwrap();
        assert_eq!(result, vec![
            Value::Number(Fraction::from(0)),
            Value::Void,
            Value::Number(Fraction::from(2)),
        ]);
    }
}
