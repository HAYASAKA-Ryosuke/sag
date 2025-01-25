use crate::parsers::Parser;
use crate::environment::ValueType;

impl Parser {
    pub fn string_to_value_type(&mut self, type_name: String) -> ValueType {
        let scope = self.get_current_scope();
        if let Some(struct_value) = self.get_struct(scope, type_name.clone()) {
            return struct_value;
        }

        match type_name.as_str() {
            "number" => ValueType::Number,
            "string" => ValueType::String,
            "bool" => ValueType::Bool,
            "void" => ValueType::Void,
            _ => panic!("undefined type: {:?}", type_name),
        }
    }
}
