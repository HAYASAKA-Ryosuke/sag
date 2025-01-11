use crate::ast::ASTNode;
use crate::parsers::Parser;
use crate::value::Value;
use crate::environment::ValueType;
use std::collections::HashMap;

impl Parser {
    pub fn infer_type(&self, ast: &ASTNode) -> Result<ValueType, String> {
        match ast {
            ASTNode::Literal(ref v) => match v {
                Value::Number(_) => Ok(ValueType::Number),
                Value::String(_) => Ok(ValueType::String),
                Value::Bool(_) => Ok(ValueType::Bool),
                Value::Void => Ok(ValueType::Void),
                Value::Struct { name, fields, methods } => {
                    let field_types = fields.iter().map(|(name, field)| {
                        if let Value::StructField { value_type, is_public: _ } = field {
                            (name.clone(), value_type.clone())
                        } else {
                            panic!("invalid struct field")
                        }
                    }).collect::<HashMap<_,_>>();
                    Ok(ValueType::Struct { name: name.clone(), fields: field_types.clone(), methods: methods.clone() })
                },
                Value::List(values) => {
                    let mut value_type = ValueType::Any;
                    for value in values {
                        let value_type_ = self.infer_type(&ASTNode::Literal(value.clone()))?;
                        if value_type == ValueType::Any {
                            value_type = value_type_;
                        } else if value_type != value_type_ {
                            return Err("type mismatch in list".to_string());
                        }
                    }
                    Ok(value_type)
                },
                _ => Ok(ValueType::Any),
            },
            ASTNode::Lambda { .. } => Ok(ValueType::Lambda),
            ASTNode::PrefixOp { op: _, expr } => {
                let value_type = self.infer_type(&expr)?;
                Ok(value_type)
            }

            ASTNode::StructInstance { name, fields } => {
                let mut field_types = HashMap::new();
                for (field_name, field_value) in fields.iter() {
                    field_types.insert(field_name.clone(), self.infer_type(field_value)?);
                }
                Ok(ValueType::StructInstance {
                    name: name.clone(),
                    fields: field_types,
                })
            }
            ASTNode::FunctionCall { name, arguments: _ } => {
                let function = self.get_function(self.get_current_scope(), name.clone());
                if function.is_none() {
                    return Err(format!("undefined function: {:?}", name));
                }
                let value_type = function.unwrap();
                Ok(value_type.clone())
            }
            ASTNode::BinaryOp { left, op, right } => {
                let left_type = self.infer_type(&left)?;
                let right_type = self.infer_type(&right)?;

                match (&left_type, &right_type) {
                    (ValueType::Number, ValueType::Number) => Ok(ValueType::Number),
                    (ValueType::Number, ValueType::String) => Ok(ValueType::String),
                    (ValueType::String, ValueType::Number) => Ok(ValueType::String),
                    (ValueType::Bool, ValueType::Bool) => Ok(ValueType::Bool),
                    _ => Err(
                        format!("type mismatch: {:?} {:?} {:?}", left_type, op, right_type).into(),
                    ),
                }
            },
            ASTNode::Variable { name, value_type } => {
                if let Some(value_type) = value_type {
                    Ok(value_type.clone())
                } else {
                    let scope = self.get_current_scope();
                    match self.find_variables(scope, name.clone()) {
                        Some((value_type, _)) => Ok(value_type.clone()),
                        None => Err(format!("undefined variable: {:?}", name).into()),
                    }
                }
            }
            _ => Ok(ValueType::Any),
        }
    }
}
