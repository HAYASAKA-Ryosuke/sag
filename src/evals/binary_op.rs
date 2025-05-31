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
