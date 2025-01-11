use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, EnvVariableType};
use crate::evals::eval;

pub fn for_node(variable: String, iterable: Box<ASTNode>, body: Box<ASTNode>, env: &mut Env) -> Value {
    let iterable = eval(*iterable, env);
    match iterable {
        Value::List(values) => {
            for value in values {
                let scope_name = format!("for-{}-{}", variable.clone(), value.clone());
                env.enter_scope(scope_name.clone());
                let _ = env.set(variable.clone(), value.clone(), EnvVariableType::Immutable, value.value_type(), true);
                eval(*body.clone(), env);
                env.leave_scope();
            }
            Value::Void
        }
        _ => panic!("Unexpected iterable: {:?}", iterable),
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
        let asts = Parser::new(tokens.to_vec()).parse_lines();
        let mut env = Env::new();
        register_builtins(&mut env);
        let result = evals(asts, &mut env);
        assert_eq!(result, vec![
            Value::Number(Fraction::from(0)),
            Value::Void,
            Value::Number(Fraction::from(6)),
        ]);
    }

}
