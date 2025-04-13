use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, EnvVariableType};
use crate::evals::eval;
use crate::evals::runtime_error::RuntimeError;
use fraction::Fraction;

fn extract_arguments(arguments: Box<ASTNode>) -> Vec<ASTNode> {
    match *arguments {
        ASTNode::FunctionCallArgs { args, .. } => args,
        _ => vec![],
    }
}

// number builtin method
fn call_builtin_method_on_number(
    num: Fraction,
    method_name: &str,
    _args: &[ASTNode],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method_name {
        "to_string" => Ok(Value::String(num.to_string())),
        "round" => Ok(Value::Number(num.round().into())),
        "sqrt" => {
            let num_f64 = *num.numer().unwrap() as f64;
            let denom_f64 = *num.denom().unwrap() as f64;
            let fraction_value = num_f64 / denom_f64;
            let sqrt_value = fraction_value.sqrt();
            Ok(Value::Number(sqrt_value.into()))
        },
        _ => Err(RuntimeError::new(
            format!("{} is not a method of number", method_name).as_str(),
            line,
            column,
        )),
    }
}

// list builtin method
fn call_builtin_method_on_list(
    mut list: Vec<Value>,
    method_name: &str,
    args: &[ASTNode],
    caller_ast: &ASTNode,
    env: &mut Env,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method_name {
        "to_string" => Ok(Value::String(format!("{:?}", list))),
        "push" => {
            if args.len() < 1 {
                return Err(RuntimeError::new("push requires an argument", line, column));
            }
            let new_val = eval(args[0].clone(), env)?;
            list.push(new_val);

            if let ASTNode::Variable { name, value_type, .. } = caller_ast {
                let result = env.set(
                    name.to_string(),
                    Value::List(list.clone()),
                    EnvVariableType::Mutable,
                    value_type.clone().unwrap_or(ValueType::Any),
                    false,
                );
                if let Err(e) = result {
                    return Err(RuntimeError::new(e.as_str(), line, column));
                }
            }
            Ok(Value::List(list))
        }
        _ => Err(RuntimeError::new(
            format!("{} is not a method of list", method_name).as_str(),
            line,
            column,
        )),
    }
}

/// Valueに応じた builtin メソッドの呼び出し
fn call_builtin_method(
    value: Value,
    method_name: &str,
    args: &[ASTNode],
    caller_ast: &ASTNode,
    env: &mut Env,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match value {
        Value::Number(num) => {
            call_builtin_method_on_number(num, method_name, args, line, column)
        }
        Value::List(list) => {
            call_builtin_method_on_list(list, method_name, args, caller_ast, env, line, column)
        }
        _ => Err(RuntimeError::new(
            format!("Method {} is not supported for this type", method_name).as_str(),
            line,
            column,
        )),
    }
}

pub fn builtin_method_call_node(
    method_name: String,
    caller: Box<ASTNode>,
    arguments: Box<ASTNode>,
    line: usize,
    column: usize,
    env: &mut Env,
) -> Result<Value, RuntimeError> {
    let args = extract_arguments(arguments);
    let value = eval(*caller.clone(), env)?;
    call_builtin_method(value, &method_name, &args, &caller, env, line, column)
}

/// 構造体メソッド呼び出しの本体
pub fn method_call_node(
    method_name: String,
    caller: Box<ASTNode>,
    arguments: Box<ASTNode>,
    line: usize,
    column: usize,
    env: &mut Env,
) -> Result<Value, RuntimeError> {
    // 引数リストの取り出し
    let args_vec = match *arguments {
        ASTNode::FunctionCallArgs { args, .. } => args,
        _ => vec![],
    };
    // caller が変数であることを確認し、変数名を取得する
    let caller_name = match *caller {
        ASTNode::Variable { name, .. } => name,
        _ => {
            return Err(RuntimeError::new(
                format!("Unexpected caller: {:?}", caller).as_str(),
                line,
                column,
            ))
        }
    };

    // 環境から caller の変数情報を取得
    let variable_info = env.get(&caller_name, None).ok_or_else(|| {
        RuntimeError::new(
            format!("missing struct: {:?}", caller_name).as_str(),
            line,
            column,
        )
    })?;

    // 構造体情報の取得
    let mut local_env = env.clone();
    let struct_info = match &variable_info.value_type {
        ValueType::StructInstance { name: struct_name, .. } => {
            local_env.get_struct(struct_name).cloned()
        }
        ValueType::Struct { name: struct_name, .. } => {
            local_env.get_struct(struct_name).cloned()
        }
        _ => {
            return Err(RuntimeError::new(
                format!("missing struct: {:?}", variable_info.value).as_str(),
                line,
                column,
            ))
        }
    };

    let methods = match &struct_info {
        Some(Value::Struct { methods, .. }) => methods,
        _ => {
            return Err(RuntimeError::new(
                format!("failed get methods: {:?}", struct_info).as_str(),
                line,
                column,
            ))
        }
    };

    // 対象のメソッド情報を取得する
    let method_info = methods.get(&method_name).ok_or_else(|| {
        RuntimeError::new(
            format!("call failed method: {:?}", method_name).as_str(),
            line,
            column,
        )
    })?;

    // 変更可能な変数であることの確認
    if variable_info.variable_type == EnvVariableType::Immutable {
        return Err(RuntimeError::new(
            format!("{} is not mutable", caller_name).as_str(),
            line,
            column,
        ));
    }
    if args_vec.len() != method_info.arguments.len() - 1 {
        return Err(RuntimeError::new(
            format!("does not match arguments length: {:?}", args_vec).as_str(),
            line,
            column,
        ));
    }

    // ローカル環境にスコープを追加して、self の設定や引数の割り当てを行う
    local_env.enter_scope(method_name.clone());
    let self_value = variable_info.value.clone();
    let result = local_env.set(
        "self".to_string(),
        self_value.clone(),
        EnvVariableType::Mutable,
        // self の型情報は構造体定義から組み立てる
        match &struct_info {
            Some(Value::Struct { name, fields, methods: _ }) => {
                let mut field_types = HashMap::new();
                for (field_name, field_value) in fields {
                    field_types.insert(field_name.to_string(), field_value.value_type());
                }
                ValueType::Struct {
                    name: name.to_string(),
                    fields: field_types,
                    methods: methods.clone(),
                }
            }
            _ => ValueType::Any,
        },
        true,
    );
    if let Err(e) = result {
        return Err(RuntimeError::new(e.as_str(), line, column));
    }
    // struct インスタンスのフィールドもローカル環境にセットする
    if let Value::StructInstance { fields, .. } = self_value {
        for (field_name, field_value) in fields {
            let result = local_env.set(
                field_name.to_string(),
                field_value.clone(),
                EnvVariableType::Mutable,
                field_value.value_type(),
                true,
            );
            if let Err(e) = result {
                return Err(RuntimeError::new(e.as_str(), line, column));
            }
        }
    } else {
        return Err(RuntimeError::new(
            format!("missing struct instance: {:?}", variable_info.value).as_str(),
            line,
            column,
        ));
    }

    // 定義された引数に対して評価して割り当てる
    for (i, define_arg) in method_info.arguments.iter().enumerate() {
        // self はすでにセット済みなのでスキップ
        if let ASTNode::Variable { name, value_type, .. } = define_arg {
            if name == "self" {
                continue;
            }
            let arg_value = eval(args_vec[i - 1].clone(), &mut local_env.clone())?;
            let result = local_env.set(
                name.to_string(),
                arg_value,
                EnvVariableType::Immutable,
                value_type.clone().unwrap_or(ValueType::Any),
                true,
            );
            if let Err(e) = result {
                return Err(RuntimeError::new(e.as_str(), line, column));
            }
        }
    }

    // メソッド本体の評価
    let result = eval(method_info.body.clone().unwrap(), &mut local_env)?;
    // Returnに包まれている場合は中身を取り出す
    let unwrapped_result = match result {
        Value::Return(inner) => *inner,
        other => other,
    };

    // メソッド呼び出し後、self の変更があればグローバル環境に反映する
    if let Some(self_var) = local_env.get(&"self".to_string(), None) {
        if let Value::StructInstance { .. } = self_var.value.clone() {
            let result = local_env.set(
                caller_name.to_string(),
                self_var.value.clone(),
                variable_info.variable_type.clone(),
                variable_info.value_type.clone(),
                false,
            );
            if let Err(e) = result {
                return Err(RuntimeError::new(e.as_str(), line, column));
            }
        }
    }
    env.update_global_env(&local_env);
    env.leave_scope();
    Ok(unwrapped_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use crate::evals::evals;

    #[test]
    fn test_to_string_method_call_node() {
        let mut env = Env::new();
        let input = "1.to_string()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[0], Value::String("1".to_string()));
    }

    #[test]
    fn test_round_method_call_node() {
        let mut env = Env::new();
        let input = "(1.5).round()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse();
        let result = eval(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result, Value::Number(2.into()));
    }

    #[test]
    fn test_sqrt_method_call_node() {
        let mut env = Env::new();
        let input = "(2 + 2).sqrt()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse();
        let result = eval(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result, Value::Number(2.into()));
    }

    #[test]
    fn test_new_method_call_node() {
        let mut env = Env::new();
        let input = r#"
        struct Point {
            x: number,
            y: number,
        }
        impl Point {
          fun new(self, x: number, y: number): Point {
            return Point { x: x, y: y }
          }
          fun get_x(self): number {
            return self.x
          }
        }
        val mut p = Point{x: 1, y: 2}
        val mut p2 = p.new(3, 4)
        p2.get_x()
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env);
        assert_eq!(result.unwrap().last(), Some(&Value::Number(3.into())));
    }
    #[test]
    fn test_push_method_call_node() {
        let mut env = Env::new();
        
        let input = "val mut xs = []\nxs.push(1)\n".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[1], Value::List(vec![Value::Number(1.into())]));
    }

    #[test]
    fn test_push_method_call_node_with_variable() {
        let mut env = Env::new();
        let input = "val mut xs = [1,2]\nval x = 3\nxs.push(x)\n".to_string();
        let tokens = tokenize(&input);
        let builtin = register_builtins(&mut env);
        let mut parser = Parser::new(tokens, builtin);
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[2], Value::List(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())]));
    }
    #[test]
    fn method_chaining_with_round_and_to_string() {
        let mut env = Env::new();
        let input = "fun add(x: number): number {\n return x + 1\n}\n add(1.5).round().to_string()".to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let result = evals(ast.unwrap(), &mut env).unwrap();
        assert_eq!(result[1], Value::String("3".to_string()));
    }
}
