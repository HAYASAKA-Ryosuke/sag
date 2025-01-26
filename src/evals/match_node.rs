use crate::ast::ASTNode;
use crate::value::Value;
use crate::evals::eval;
use crate::environment::Env;
use crate::evals::runtime_error::RuntimeError;

pub fn match_node(expression: Box<ASTNode>, cases: Vec<(ASTNode, ASTNode)>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let expression_value = eval(*expression, env)?;
    for (pattern, body) in cases.clone() {
        match pattern {
            ASTNode::Literal{value, ..} => {
                if value == expression_value {
                    let result = eval(body, env)?;
                    return Ok(result);
                }
            }
            ASTNode::OptionSome{ref value, ..} => {
                if let Value::Option(Some(ref some_value)) = expression_value {
                    match value.as_ref() {
                        ASTNode::Literal{value, ..} => {
                            if value == some_value.as_ref() {
                                let result = eval(body, env)?;
                                return Ok(result);
                            }
                        },
                        ASTNode::Variable{name, ..} => {
                            let _ = env.set(name.clone(), some_value.as_ref().clone(), crate::environment::EnvVariableType::Mutable, some_value.value_type().clone(), true);
                            let result = eval(body, env)?;
                            return Ok(result);
                        },
                        _ => {
                            return Err(RuntimeError::new("Unsupported pattern", line, column));
                        }
                    }
                    
                }
            }
            ASTNode::OptionNone{..} => {
                if let Value::Option(None) = expression_value {
                    let result = eval(body, env)?;
                    return Ok(result);
                }
            }
            ASTNode::ResultSuccess{ref value, ..} => {
                if let Value::Result(Ok(ref some_value)) = expression_value {
                    match value.as_ref() {
                        ASTNode::Literal{value, ..} => {
                            if value == some_value.as_ref() {
                                let result = eval(body, env)?;
                                return Ok(result);
                            }
                        },
                        ASTNode::Variable{name, ..} => {
                            let _ = env.set(name.clone(), some_value.as_ref().clone(), crate::environment::EnvVariableType::Mutable, some_value.value_type().clone(), true);
                            let result = eval(body, env)?;
                            return Ok(result);
                        },
                        _ => {
                            return Err(RuntimeError::new("Unsupported pattern", line, column));
                        }
                    }
                    
                }
            }
            ASTNode::ResultFailure{ref value, ..} => {
                if let Value::Result(Err(ref some_value)) = expression_value {
                    match value.as_ref() {
                        ASTNode::Literal{value, ..} => {
                            if value == some_value.as_ref() {
                                let result = eval(body, env)?;
                                return Ok(result);
                            }
                        },
                        ASTNode::Variable{name, ..} => {
                            let _ = env.set(name.clone(), some_value.as_ref().clone(), crate::environment::EnvVariableType::Mutable, some_value.value_type().clone(), true);
                            let result = eval(body, env)?;
                            return Ok(result);
                        },
                        _ => {
                            return Err(RuntimeError::new("Unsupported pattern", line, column));
                        }
                    }
                    
                }
            }
            _ => {}
        }
    }
    for (pattern, body) in cases {
        match pattern {
            ASTNode::Variable{name, ..} if name == "_" => {
                return Ok(eval(body, env)?);
            },
            _ => {}
        }
    }
    Err(RuntimeError::new("No match found", line, column))
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
    fn test_pattern_matching() {
        let input = r#"
        match 1 {
            1 => {return 2}
            _ => {return 3}
        }
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(2)));

        let input = r#"
        match 2 {
            1 => { return 2 }
            _ => { return 3 }
        }
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(3)));
        let input = r#"
        match Some(2) {
            Some(2) => { return 2 }
            _ => { return 3 }
        }
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(2)));
        let input = r#"
        val x:Option<number> = Some(2)
        match (x) {
            Some(v) => { return (v + 10) }
            None => { return 3 }
            _ => { return 4 }
        }
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let result = evals(ast, &mut env).unwrap();
        assert_eq!(result[1], Value::Number(Fraction::from(12)));
        let input = r#"
        match None {
            Some(2) => { return 2 }
            None => { return 3 }
            _ => { return 4 }
        }
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(3)));
    }
}
