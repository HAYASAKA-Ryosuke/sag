use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::Env;
use crate::token::TokenKind;
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;

pub fn binary_op(op: TokenKind, left: Box<ASTNode>, right: Box<ASTNode>, line: usize, column: usize, env: &mut Env) -> Result<Value, RuntimeError> {
    let left_val = eval(*left, env)?;
    let right_val = eval(*right, env)?;

    match (left_val.clone(), right_val.clone(), op.clone()) {
        (Value::String(l), Value::String(r), TokenKind::Plus) => Ok(Value::String(l + &r)),
        (Value::Number(l), Value::Number(r), TokenKind::Plus) => Ok(Value::Number(l + r)),
        (Value::Number(l), Value::Number(r), TokenKind::Minus) => Ok(Value::Number(l - r)),
        (Value::Number(l), Value::Number(r), TokenKind::Mul) => Ok(Value::Number(l * r)),
        (Value::Number(l), Value::Number(r), TokenKind::Div) => Ok(Value::Number(l / r)),
        (Value::Number(l), Value::Number(r), TokenKind::Mod) => Ok(Value::Number(l % r)),
        (Value::Number(l), Value::Number(r), TokenKind::Pow) => {
            let a = l.numer().unwrap();
            let b = l.denom().unwrap();
            let c = r.numer().unwrap();
            let d = r.denom().unwrap();

            let raw_numer = a.checked_pow(*c as u32).ok_or(RuntimeError::new("Overflow Numerator", line, column))?;
            let raw_denom = b.checked_pow(*d as u32).ok_or(RuntimeError::new("Overflow Denominator", line, column))?;
            if raw_denom == 0 {
                return Err(RuntimeError::new("Division by zero", line, column));
            }
            Ok(Value::Number((raw_numer, raw_denom).into()))
        },
        (Value::Bool(l), Value::Bool(r), TokenKind::And) => Ok(Value::Bool(l && r)),
        (Value::Bool(l), Value::Bool(r), TokenKind::Or) => Ok(Value::Bool(l || r)),
        (Value::Bool(l), Value::Bool(r), TokenKind::Xor) => Ok(Value::Bool(l && !r || !l && r)),
        (Value::Number(l), Value::Number(r), TokenKind::And) => Ok(Value::Number((l.numer().unwrap() & r.numer().unwrap(), l.denom().unwrap() & r.denom().unwrap()).into())),
        (Value::Number(l), Value::Number(r), TokenKind::Or) => Ok(Value::Number((l.numer().unwrap() | r.numer().unwrap(), l.denom().unwrap() | r.denom().unwrap()).into())),
        (Value::Number(l), Value::Number(r), TokenKind::Xor) => {
            // 分母を揃えて計算
            let a = l.numer().unwrap();
            let b = l.denom().unwrap();
            let c = r.numer().unwrap();
            let d = r.denom().unwrap();
            let ad = a.checked_mul(*d).ok_or(RuntimeError::new("Overflow Numerator", line, column))?;
            let cb = c.checked_mul(*b).ok_or(RuntimeError::new("Overflow Numerator", line, column))?;
            let raw_numer = ad ^ cb;
            let raw_denom = b.checked_mul(*d).ok_or(RuntimeError::new("Overflow Denominator", line, column))?;
            if raw_denom == 0 {
                return Err(RuntimeError::new("Division by zero", line, column));
            }
            Ok(Value::Number((raw_numer, raw_denom).into()))
        },
        _ => Err(RuntimeError::new(format!("Unsupported operation: {:?} {:?} {:?}", left_val.clone(), op, right_val.clone()).as_str(), line, column)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use crate::evals::evals;

    #[test]
    fn add() {
        let mut env = Env::new();
        let input = "1 + 1".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::Number(2.into()));
    }

    #[test]
    fn sub() {
        let mut env = Env::new();
        let input = "1 - 1".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::Number(0.into()));
    }

    #[test]
    fn mul() {
        let mut env = Env::new();
        let input = "2 * 3".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::Number(6.into()));
    }

    #[test]
    fn div() {
        let mut env = Env::new();
        let input = "2 / 3".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::Number((2, 3).into()));
    }

    #[test]
    fn and() {
        let mut env = Env::new();
        for xy in [(true, true), (true, false), (false, true), (false, false)] {
            let input = format!("{} and {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Bool(xy.0 && xy.1));
        }

        for xy in [(1, 1), (1, 0), (0, 1), (0, 0)] {
            let input = format!("{} and {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Number((xy.0 & xy.1, 1).into()));
        }
    }

    #[test]
    fn or() {
        let mut env = Env::new();
        for xy in [(true, true), (true, false), (false, true), (false, false)] {
            let input = format!("{} or {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Bool(xy.0 || xy.1));
        }
        for xy in [(1, 1), (1, 0), (0, 1), (0, 0)] {
            let input = format!("{} or {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Number((xy.0 | xy.1, 1).into()));
        }
    }

    #[test]
    fn xor() {
        let mut env = Env::new();
        for xy in [(true, true), (true, false), (false, true), (false, false)] {
            let input = format!("{} xor {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Bool(xy.0 ^ xy.1));
        }
        for xy in [(1, 1), (1, 0), (0, 1), (0, 0)] {
            let input = format!("{} xor {}", xy.0, xy.1);
            let tokens = tokenize(&input);
            let mut parser = Parser::new(tokens, register_builtins(&mut env));
            let ast = parser.parse_lines();
            let result = evals(ast.unwrap(), &mut env).unwrap();
            assert_eq!(result[0], Value::Number((xy.0 ^ xy.1, 1).into()));
        }
    }

    #[test]
    fn pow() {
        let mut env = Env::new();
        let input = "2 ** 3".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::Number((8, 1).into()));
    }
}
