use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::value::Value;
use crate::environment::{Env, ValueType, MethodInfo, EnvVariableType};
use crate::evals::eval;

pub fn struct_node(name: String, fields: HashMap<String, ASTNode>, env: &mut Env) -> Value {
    let mut struct_fields = HashMap::new();
    // fields field_name: StructField
    for (field_name, struct_field) in fields {
        match struct_field {
            ASTNode::StructField { value_type, is_public } => {
                struct_fields.insert(field_name, Value::StructField {
                    value_type,
                    is_public
                });
            },
            _ => panic!("Unexpected struct field: {:?}", struct_field),
        }
    }
    let result = Value::Struct {
        name,
        fields: struct_fields,
        methods: HashMap::new()
    };
    env.register_struct(result.clone());
    result
}

pub fn impl_node(base_struct: Box<ValueType>, methods: Vec<ASTNode>, env: &mut Env) -> Value {
    let mut impl_methods = HashMap::new();
    for method in methods {
        match method {
            ASTNode::Method {
                name,
                arguments,
                body,
                return_type,
                is_mut
            } => {
                let method_info = MethodInfo {
                    arguments,
                    body: Some(*body),
                    return_type,
                    is_mut,
                };
                impl_methods.insert(name, method_info);
            },
            _ => panic!("Unexpected method: {:?}", method),
        }
    }
    let result = Value::Impl {
        base_struct: *base_struct,
        methods: impl_methods,
    };
    env.register_impl(result.clone());
    result
}

pub fn struct_instance_node(name: String, fields: HashMap<String, ASTNode>, env: &mut Env) -> Value {
    let mut struct_fields = HashMap::new();
    for (field_name, field_value) in fields {
        struct_fields.insert(field_name, eval(field_value, env));
    }
    Value::StructInstance {
        name,
        fields: struct_fields,
    }
}

pub fn struct_field_assign_node(instance: Box<ASTNode>, updated_field_name: String, updated_value_ast: Box<ASTNode>, env: &mut Env) -> Value {
    match *instance {
        ASTNode::StructFieldAccess { instance, field_name: _  } => {
            match *instance {
                ASTNode::Variable { name: variable_name, value_type } => {
                    match value_type {
                        Some(ValueType::Struct{name, fields, ..}) if variable_name == "self" => {
                            match env.get_struct(&name) {
                                Some(Value::Struct { fields: _, methods, .. }) => {
                                    let scope = env.get_current_scope();
                                    match methods.get(&scope) {
                                        Some(MethodInfo {arguments, ..}) => {
                                            let first_argument = arguments.first();
                                            if first_argument.is_none() {
                                                panic!("missing self argument");
                                            }
                                            match first_argument.unwrap() {
                                                ASTNode::Variable { name: self_argument, value_type: self_type } => {
                                                    if self_argument != "self" || *self_type != Some(ValueType::MutSelfType) {
                                                        panic!("{} is not mut self argument", scope);
                                                    }
                                                },
                                                _ => panic!("missing self argument"),
                                            }
                                        },
                                        _ => panic!("missing method info")
                                    }
                                },
                                _ => panic!("Unexpected value type"),
                            };
                            let obj = env.get(&variable_name, None);
                            if obj.is_none() {
                                panic!("Variable not found: {:?}", variable_name);
                            }
                            let mut struct_fields = HashMap::new();
                            match obj.unwrap().value.clone() {
                                Value::StructInstance { .. } => {
                                    let instance_value = obj.unwrap().value.clone();
                                    let updated_value = match instance_value {
                                        Value::StructInstance { name, fields } => {
                                            let mut updated_fields = fields.clone();
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            *updated_fields.entry(updated_field_name.to_string()).or_insert(updated_value.clone()) = updated_value.clone();
                                            Value::StructInstance{name, fields: updated_fields}
                                        },
                                        _ => panic!("missing struct instance value: {:?}", instance_value)
                                    };
                                    env.set(variable_name.to_string(), updated_value.clone(), EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false).expect("update variable");
                                    updated_value
                                },
                                Value::Struct { name: _, fields: obj_fields, .. } => {
                                    for (field_name, field_value) in obj_fields {
                                        if field_name == updated_field_name {
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            if field_value.value_type() != updated_value.value_type() {
                                                panic!("Struct field type mismatch: {}.{}:{:?} = {:?}", variable_name, field_name, field_value.value_type(), updated_value.value_type());
                                            }
                                            struct_fields.insert(field_name, updated_value);
                                        } else {
                                            struct_fields.insert(field_name, field_value);
                                        }
                                    }
                                    let env_updated_result = env.set(variable_name.to_string(), Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields.clone(),
                                    }, EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false);
                                    if env_updated_result.is_err() {
                                        panic!("{}", env_updated_result.unwrap_err());
                                    }
                                    Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields,
                                    }
                                },
                                _ => panic!("Unexpected value type: {:?}", obj),
                            }
                        }
                        Some(ValueType::StructInstance { name, fields }) => {
                            let obj = env.get(&variable_name, Some(&ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }));
                            if obj.is_none() {
                                panic!("Variable not found: {:?}", variable_name);
                            }
                            let mut struct_fields = HashMap::new();
                            match obj.unwrap().value.clone() {
                                Value::StructInstance { name: _, fields: obj_fields } => {
                                    for (field_name, field_value) in obj_fields {
                                        if field_name == updated_field_name {
                                            let updated_value = eval(*updated_value_ast.clone(), env);
                                            if field_value.value_type() != updated_value.value_type() {
                                                panic!("Struct field type mismatch: {}.{}:{:?} = {:?}", variable_name, field_name, field_value.value_type(), updated_value.value_type());
                                            }
                                            struct_fields.insert(field_name, updated_value);
                                        } else {
                                            struct_fields.insert(field_name, field_value);
                                        }
                                    }
                                    let env_updated_result = env.set(variable_name.to_string(), Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields.clone(),
                                    }, EnvVariableType::Mutable, ValueType::StructInstance { name: name.to_string(), fields: fields.clone() }, false);
                                    if env_updated_result.is_err() {
                                        panic!("{}", env_updated_result.unwrap_err());
                                    }
                                    Value::StructInstance {
                                        name: variable_name.to_string(),
                                        fields: struct_fields,
                                    }
                                },
                                _ => panic!("Unexpected value type"),
                            }
                        },
                        _ => panic!("Unexpected value type"),
                    }
                },
                _ => panic!("Unexpected value type"),
            }
        },
        _ => panic!("Unexpected value type"),
    }
}

pub fn struct_field_access_node(instance: Box<ASTNode>, field_name: String, env: &mut Env) -> Value {
    let struct_obj = match *instance {
        ASTNode::Variable { name: variable_name, value_type } => {
            match value_type {
                Some(ValueType::Struct { .. }) if variable_name == "self" => {
                    let obj = env.get(&variable_name, None);
                    if obj.is_none() {
                        panic!("Variable not found: {:?}", variable_name);
                    }
                    obj.unwrap().value.clone()
                },
                Some(ValueType::StructInstance { name, fields }) => {
                    let obj = env.get(&variable_name, Some(&ValueType::StructInstance { name: name.to_string(), fields }));
                    if obj.is_none() {
                        panic!("Variable not found: {:?}", variable_name);
                    }
                    obj.unwrap().value.clone()
                },
                _ => panic!("Unexpected value type"),
            }
        },
        _ => panic!("Unexpected value type"),
    };
    match struct_obj {
        Value::Struct {fields, ..} => {
            // selfのケース
            if !fields.contains_key(&field_name) {
                panic!("Field not found: {:?}", field_name);
            }
            fields.get(&field_name).unwrap().clone()
        }
        Value::StructInstance { name: _, fields } => {
            if !fields.contains_key(&field_name) {
                panic!("Field not found: {:?}", field_name);
            }
            fields.get(&field_name).unwrap().clone()
        },
        _ => panic!("Unexpected value: {:?}", struct_obj),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parsers::Parser;
    use crate::builtin::register_builtins;
    use fraction::Fraction;
    use crate::evals::evals;


    #[test]
    fn test_mutset_impl() {
        let input = r#"
            struct Foo {
              value: number,
            }
            
            impl Foo {
              fun set(mut self, num: number) {
                self.value = num
              }
            }
            
            val mut foo = Foo{value: 1}
            foo.set(3)
            foo.value
        "#;

        let tokens = tokenize(&input.to_string());
        let mut env = Env::new();
        let builtins = register_builtins(&mut env);
        let asts = Parser::new(tokens, builtins).parse_lines();
        let result = evals(asts, &mut env);
        assert_eq!(result.last(), Some(&Value::Number(Fraction::from(3))));
    }

    #[test]
    #[should_panic(expected = "set is not mut self argument")]
    fn test_not_mut_set_impl() {
        let input = r#"
            struct Foo {
              value: number,
            }
            
            impl Foo {
              fun set(self, num: number) {
                self.value = num
              }
            }
            
            val mut foo = Foo{value: 1}
            foo.set(3)
            foo.value
        "#;

        let tokens = tokenize(&input.to_string());
        let mut env = Env::new();
        let builtins = register_builtins(&mut env);
        let asts = Parser::new(tokens.to_vec(), builtins).parse_lines();
        evals(asts, &mut env);
    }

    #[test]
    #[should_panic(expected = "foo is not mutable")]
    fn test_not_mut_instance_impl() {
        let input = r#"
            struct Foo {
              value: number,
            }
            
            impl Foo {
              fun set(self, num: number) {
                self.value = num
              }
            }
            
            val foo = Foo{value: 1}
            foo.set(3)
            foo.value
        "#;

        let tokens = tokenize(&input.to_string());
        let mut env = Env::new();
        let builtin = register_builtins(&mut env);
        let asts = Parser::new(tokens.to_vec(), builtin).parse_lines();
        evals(asts, &mut env);
    }

    #[test]
    fn test_struct_impl() {
        let input = r#"
struct Point {
  x: number,
  y: number
}

impl Point {
  fun move(mut self, dx: number, dy: number) {
      self.x = self.x + dx
      self.y = self.y + dy
  }
}

impl Point {
  fun clear(mut self) {
      self.x = 0
      self.y = 0
  }
}

val x = 8
val y = 3
val mut point = Point{x: x, y: y}
point.move(5, 2)
point.clear()
"#;

        let tokens = tokenize(&input.to_string());
        let mut env = Env::new();
        let builtin = register_builtins(&mut env);
        let asts = Parser::new(tokens, builtin).parse_lines();
        let result = evals(asts, &mut env);
        let base_struct = Value::Struct {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("y".into(), Value::StructField{
                    value_type: ValueType::Number,
                    is_public: false
                }),
                ("x".into(), Value::StructField{
                    value_type: ValueType::Number,
                    is_public: false
                })
            ]),
            methods: HashMap::new(),
        };
        assert_eq!(result.first(), Some(base_struct.clone()).as_ref());
        assert_eq!(result.get(6), Some(Value::Void).as_ref());
    }

    #[test]
    #[should_panic(expected = "Struct field type mismatch: point.x:Number = String")]
    fn test_struct_other_type_assign() {
        let asts = vec![
            ASTNode::Struct {
                name: "Point".into(),
                fields: HashMap::from_iter(vec![
                    ("x".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    }),
                    ("y".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    })
                ])
            },
            ASTNode::Assign {
                name: "point".into(),
                variable_type: EnvVariableType::Mutable,
                is_new: true,
                value_type: ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                    ("x".into(), ValueType::Number),
                    ("y".into(), ValueType::Number)
                ])},
                value: Box::new(ASTNode::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                        ("y".into(), ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    ]),
                }),
            },
            ASTNode::StructFieldAssign {
                instance: Box::new(ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable {
                        name: "point".into(),
                        value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                            ("x".into(), ValueType::Number),
                            ("y".into(), ValueType::Number)
                        ])})
                    }),
                    field_name: "x".into()
                }),
                value: Box::new(ASTNode::Literal(Value::String("hello".into()))),
                field_name: "x".into()
            },
        ];
        let mut env = Env::new();
        evals(asts, &mut env);
    }

    #[test]
    fn test_struct_access() {
        let asts = vec![
            ASTNode::Struct {
                name: "Point".into(),
                fields: HashMap::from_iter(vec![
                    ("x".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    }),
                    ("y".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    })
                ])
            },
            ASTNode::Assign {
                name: "point".into(),
                variable_type: EnvVariableType::Mutable,
                is_new: true,
                value_type: ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                    ("x".into(), ValueType::Number),
                    ("y".into(), ValueType::Number)
                ])},
                value: Box::new(ASTNode::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                        ("y".into(), ASTNode::Literal(Value::Number(Fraction::from(2)))),
                    ]),
                }),
            },
            ASTNode::StructFieldAccess {
                instance: Box::new(ASTNode::Variable {
                    name: "point".into(),
                    value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                        ("x".into(), ValueType::Number),
                        ("y".into(), ValueType::Number)
                    ])})
                }),
                field_name: "x".into()
            },
            ASTNode::StructFieldAssign {
                instance: Box::new(ASTNode::StructFieldAccess {
                    instance: Box::new(ASTNode::Variable {
                        name: "point".into(),
                        value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                            ("x".into(), ValueType::Number),
                            ("y".into(), ValueType::Number)
                        ])})
                    }),
                    field_name: "x".into()
                }),
                value: Box::new(ASTNode::Literal(Value::Number(Fraction::from(3)))),
                field_name: "x".into()
            },
            ASTNode::StructFieldAccess {
                instance: Box::new(ASTNode::Variable {
                    name: "point".into(),
                    value_type: Some(ValueType::StructInstance{name: "Point".into(), fields: HashMap::from_iter(vec![
                        ("x".into(), ValueType::Number),
                        ("y".into(), ValueType::Number)
                    ])})
                }),
                field_name: "x".into()
            },
        ];
        let mut env = Env::new();
        let result = evals(asts, &mut env);
        assert_eq!(result[4], Value::Number(Fraction::from(3)));
    }

    #[test]
    fn test_assign_struct() {
        let mut env = Env::new();
        let ast = vec![
            ASTNode::Struct {
                name: "Point".into(),
                fields: HashMap::from_iter(vec![
                    ("y".into(), ASTNode::StructField {
                        value_type: ValueType::String,
                        is_public: false
                    }),
                    ("x".into(), ASTNode::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    })
                ]),
            },
            ASTNode::Assign {
                name: "point".into(),
                value: Box::new(ASTNode::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ASTNode::Literal(Value::Number(Fraction::from(1)))),
                        ("y".into(), ASTNode::Literal(Value::String("hello".into())))
                    ])
                }),
                variable_type: EnvVariableType::Immutable,
                value_type: ValueType::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), ValueType::Number),
                        ("y".into(), ValueType::String)
                    ])
                },
                is_new: true
            }
        ];
        let result = evals(ast, &mut env);
        assert_eq!(
            result,
            vec![
                Value::Struct {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("y".into(), Value::StructField {
                            value_type: ValueType::String,
                            is_public: false
                        }),
                        ("x".into(), Value::StructField {
                            value_type: ValueType::Number,
                            is_public: false
                        })
                    ]),
                    methods: HashMap::new()
                },
                Value::StructInstance {
                    name: "Point".into(),
                    fields: HashMap::from_iter(vec![
                        ("x".into(), Value::Number(Fraction::from(1))),
                        ("y".into(), Value::String("hello".into()))
                    ])
                }
            ]
        );
    }

    #[test]
    fn test_struct() {
        let mut env = Env::new();
        let ast = ASTNode::Struct {
            name: "Point".into(),
            fields: HashMap::from_iter(vec![
                ("x".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: true }),
                ("y".into(), ASTNode::StructField {
                    value_type: ValueType::Number,
                    is_public: false
                })
            ])
        };
        let result = eval(ast, &mut env);
        assert_eq!(
            result,
            Value::Struct {
                name: "Point".into(),
                methods: HashMap::new(),
                fields: HashMap::from_iter(vec![
                    ("x".into(), Value::StructField {
                        value_type: ValueType::Number,
                        is_public: true
                    }),
                    ("y".into(), Value::StructField {
                        value_type: ValueType::Number,
                        is_public: false
                    })
                ])
            }
        );
        assert_eq!(env.get_struct(&"Point".to_string()).is_some(), true);
        assert_eq!(env.get_struct(&"DummuStruct".to_string()).is_some(), false);
    }
}
