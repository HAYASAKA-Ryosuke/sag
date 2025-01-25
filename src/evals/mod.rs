pub mod prefix_op;
pub mod struct_node;
pub mod function_node;
pub mod comparison_op;
pub mod if_node;
pub mod assign_node;
pub mod lambda_node;
pub mod variable_node;
pub mod binary_op;
pub mod for_node;
pub mod import_node;
pub mod method_call_node;
pub mod runtime_error;
pub mod match_node;

use crate::environment::Env;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::token::TokenKind;
use crate::evals::runtime_error::RuntimeError;

pub fn evals(asts: Vec<ASTNode>, env: &mut Env) -> Result<Vec<Value>, RuntimeError> {
    let mut values = vec![];
    for ast in asts {
        values.push(eval(ast, env)?);
    }
    Ok(values)
}

pub fn eval(ast: ASTNode, env: &mut Env) -> Result<Value, RuntimeError> {
    match ast {
        ASTNode::Import {
            module_name,
            symbols,
            line,
            column,
        } => import_node::import_node(module_name, symbols, line, column, env),
        ASTNode::Public { node, line, column } => import_node::public_node(node, line, column, env),
        ASTNode::Literal{value, ..} => Ok(value.clone()),
        ASTNode::PrefixOp { op, expr, line, column } => prefix_op::prefix_op(op, expr, line, column, env),
        ASTNode::Struct {
            name,
            fields,
            line,
            column,
        } => struct_node::struct_node(name, fields, line, column, env),
        ASTNode::Impl {
            base_struct,
            methods,
            line,
            column,
        } => {
            struct_node::impl_node(base_struct, methods, line, column, env)
        }
        ASTNode::MethodCall { method_name, caller, arguments, builtin, line, column } => {
            if builtin {
                method_call_node::builtin_method_call_node(method_name, caller, arguments, line, column, env)
            } else {
                match caller {
                    _ => method_call_node::method_call_node(method_name, caller, arguments, line, column, env)
                }
            }
        }
        ASTNode::StructInstance {
            name,
            fields,
            line,
            column,
        } => {
            struct_node::struct_instance_node(name, fields, line, column, env)
        }
        ASTNode::StructFieldAssign { instance, field_name: updated_field_name, value: updated_value_ast, line, column } => {
            struct_node::struct_field_assign_node(instance, updated_field_name, updated_value_ast, line, column, env)
        }
        ASTNode::StructFieldAccess { instance, field_name, line, column } => {
            struct_node::struct_field_access_node(instance, field_name, line, column, env)
        }
        ASTNode::Function {
            name,
            arguments,
            body,
            return_type,
            line,
            column,
        } => {
            function_node::function_node(name, arguments, body, return_type, line, column, env)
        }
        ASTNode::Lambda { arguments, body, .. } => Ok(Value::Lambda {
            arguments,
            body: body.clone(),
            env: env.clone(),
        }),
        ASTNode::Block{nodes: statements, line, column} => {
            function_node::block_node(statements, line, column, env)
        }
        ASTNode::Return{expr: value, line: _, column: _} => {
            Ok(Value::Return(Box::new(eval(*value, env)?)))
        }
        ASTNode::Eq { left, right, line, column } => {
            comparison_op::comparison_op_node(TokenKind::Eq, left, right, line, column, env)
        }
        ASTNode::Gte { left, right, line, column, } => {
            comparison_op::comparison_op_node(TokenKind::Gte, left, right, line, column, env)
        }
        ASTNode::Gt { left, right, line, column, } => {
            comparison_op::comparison_op_node(TokenKind::Gt, left, right, line, column, env)
        }
        ASTNode::Lte { left, right, line, column, } => {
            comparison_op::comparison_op_node(TokenKind::Lte, left, right, line, column, env)
        }
        ASTNode::Lt { left, right, line, column, } => {
            comparison_op::comparison_op_node(TokenKind::Lt, left, right, line, column, env)
        }
        ASTNode::For {
            variable,
            iterable,
            body,
            line,
            column
        } => {
            for_node::for_node(variable, iterable, body, line, column, env)
        }
        ASTNode::Match {
            expression,
            cases,
            line,
            column
        } => {
            match_node::match_node(expression, cases, line, column, env)
        }
        ASTNode::OptionSome { value, line: _, column: _ } => {
            let value = eval(*value, env)?;
            Ok(Value::Option(Some(value.into())))
        }
        ASTNode::OptionNone { line: _, column: _ } => {
            Ok(Value::Option(None))
        }
        ASTNode::If {
            condition,
            then,
            else_,
            value_type: _,
            line,
            column
        } => {
            if_node::if_node(condition, then, else_, line, column, env)
        }
        ASTNode::Assign {
            name,
            value,
            variable_type,
            value_type,
            is_new,
            line,
            column
        } => {
            assign_node::assign_node(name, value, value_type, variable_type, is_new, line, column, env)
        }
        ASTNode::LambdaCall { lambda, arguments, line, column } => {
            lambda_node::lambda_call_node(lambda, arguments, line, column, env)
        }
        ASTNode::FunctionCall { name, arguments, line, column } => {
            function_node::function_call_node(name, arguments, line, column, env)
        }
        ASTNode::Variable {
            name,
            value_type,
            line,
            column,
        } => {
            variable_node::variable_node(name, value_type, line, column, env)
        }
        ASTNode::BinaryOp { left, op, right, line, column } => {
            binary_op::binary_op(op, left, right, line, column, env)
        }
        ASTNode::CommentBlock{..} => Ok(Value::Void),
        _ => Err(RuntimeError::new(format!("Unsupported ast node: {:?}", ast).as_str(), 0, 0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use fraction::Fraction;

    #[test]
    fn test_assign_expression_value() {
        let mut env = Env::new();
        let input = r#"
        val mut x = 5
        val mut y = x + 5
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Number(Fraction::from(10)));
    }
    #[test]
    fn test_assign_overwrite_mutable_variable() {
        let mut env = Env::new();
        let input = r#"
        val mut x = 10
        x = 20
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Number(Fraction::from(20)));
    }
    #[test]
    fn test_assign_to_immutable_variable() {
        let input = r#"
        val x = 200
        x = 300
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        assert!(ast.is_err());
        assert_eq!(ast.err().unwrap().message, "It is an immutable variable and cannot be reassigned: \"x\"");
    }

    #[test]
    fn test_unsupported_prefix_operation() {
        let mut env = Env::new();
        let input = r#"
        +5
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse();
        assert!(ast.is_err());
        assert_eq!(ast.err().unwrap().message, "unexpected token: Plus");

    }

    #[test]
    fn test_unsupported_binary_operation() {
        let mut env = Env::new();
        let input = r#"
        5 * "hello"
        "#.to_string();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines();
        let results = evals(ast.unwrap(), &mut env);
        assert!(results.is_err());
    }

    #[test]
    fn test_list() {
        let input = r#"
        val mut x = [1, 2, 3]
        x.push(4)
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::List(vec![
                Value::Number(Fraction::from(1)),
                Value::Number(Fraction::from(2)),
                Value::Number(Fraction::from(3)),
                Value::Number(Fraction::from(4)),
        ]));
    }

    #[test]
    #[should_panic(expected = "does not match arguments length")]
    fn test_function_call_argument_mismatch() {
        let input = r#"
        fun foo(x: number): number {
            return x
        }
        foo(1, 2)
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env);
        assert!(results.is_err());
        assert_eq!(results.err().unwrap().message, "Function foo does not match arguments length");
    }

    #[test]
    fn test_scope_management_in_function() {
        let input = r#"
        fun add_and_return(a: number): number {
            val mut local_var = 10
            return a + local_var
        }
        add_and_return(5)
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Number(Fraction::from(15)));
        // スコープ外でローカル変数が見つからないことを確認
        let local_var_check = env.get(&"local_var".to_string(), None);
        assert!(local_var_check.is_none());
    }

    #[test]
    fn test_scope_and_global_variable() {
        let input = r#"
        val mut z = 3
        fun f1(x: number, y: number): number {
            z = 2
            val mut d = 3
            z = d = 4
            return x + y + z
        }
        fun f2(x: number, y: number): number {
            return x + y + z
        }
        fun f3(): number {
            return 1
        }
        f1(2, 0)
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6
        let tokens = tokenize(&"f2(2, 0)".to_string());
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(6))); // 2 + 0 + z(4) = 6

        let tokens = tokenize(&"f3()".to_string());
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse().unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(result, Value::Number(Fraction::from(1)));
    }

    #[test]
    fn test_global_variable_and_functions() {
        let input = r#"
        val mut z = 3
        fun f1(x: number, y: number): number {
            z = 2
            val mut d = 3
            z = d = 4
            return x + y + z
        }
        |2, 0| -> f1
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Number(Fraction::from(6))); // 2 + 0 + 4 = 6
    }

    #[test]
    fn test_option_type() {
        let input = r#"
        val mut x:Option<number> = None
        x = Some(5)
        val mut y = None
        x
        "#.to_string();
        let mut env = Env::new();
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens, register_builtins(&mut env));
        let ast = parser.parse_lines().unwrap();
        let results = evals(ast, &mut env).unwrap();
        assert_eq!(*results.last().unwrap(), Value::Option(Some(Box::new(Value::Number(Fraction::from(5))))));
    }

    #[test]
    fn test_comparison_operations() {
        let mut env = Env::new();
        let ast = ASTNode::Eq {
            left: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            right: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            line: 0,
            column: 0,
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env).unwrap());

        let ast = ASTNode::Gte {
            left: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            right: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            line: 0,
            column: 0,
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env).unwrap());

        let ast = ASTNode::Gt {
            left: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            right: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            line: 0,
            column: 0,
        };
        assert_eq!(Value::Bool(false), eval(ast, &mut env).unwrap());

        let ast = ASTNode::Lte {
            left: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            right: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            line: 0,
            column: 0,
        };
        assert_eq!(Value::Bool(true), eval(ast, &mut env).unwrap());

        let ast = ASTNode::Lt {
            left: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            right: Box::new(ASTNode::Literal{value: Value::Number(Fraction::from(1)), line: 0, column: 0}),
            line: 0,
            column: 0,
        };
        assert_eq!(Value::Bool(false), eval(ast, &mut env).unwrap());
    }
}
