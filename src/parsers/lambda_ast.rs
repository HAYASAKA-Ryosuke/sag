use crate::ast::ASTNode;
use crate::token::Token;
use crate::parsers::Parser;
use crate::environment::EnvVariableType;

impl Parser {
    pub fn parse_lambda(&mut self) -> ASTNode {
        self.consume_token();
        let mut arguments = vec![];

        self.enter_scope("lambda".to_string());
        match self.get_current_token() {
            Some(Token::Pipe) => {
                self.consume_token();
                while let Some(token) = self.get_current_token() {
                    if token == Token::Pipe {
                        self.consume_token();
                        break;
                    }
                    if self.get_current_token() == Some(Token::Comma) {
                        self.consume_token();
                        continue;
                    }
                    if let Token::Identifier(argument) = token {
                        self.consume_token();
                        self.extract_token(Token::Colon);
                        let value_type =
                            if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                                Some(self.string_to_value_type(type_name))
                            } else {
                                None
                            };
                        arguments.push(ASTNode::Variable {
                            name: argument.clone(),
                            value_type: value_type.clone(),
                        });
                        self.register_variables(
                            "lambda".to_string(),
                            &argument,
                            &value_type.unwrap(),
                            &EnvVariableType::Immutable,
                        );
                        self.consume_token();
                        continue;
                    }
                }
            }
            Some(Token::Identifier(argument)) => {
                self.consume_token();
                self.extract_token(Token::Colon);
                let value_type =
                    if let Some(Token::Identifier(type_name)) = self.get_current_token() {
                        Some(self.string_to_value_type(type_name))
                    } else {
                        None
                    };
                arguments.push(ASTNode::Variable {
                    name: argument.clone(),
                    value_type,
                });
                self.consume_token();
            }
            _ => {}
        };

        self.extract_token(Token::RRocket);

        let result = match self.get_current_token() {
            Some(Token::LBrace) => {
                let statement = self.parse_block();
                ASTNode::Lambda {
                    arguments,
                    body: Box::new(statement),
                }
            }
            _ => {
                let statement = self.parse_expression(0);
                ASTNode::Lambda {
                    arguments,
                    body: Box::new(statement),
                }
            }
        };
        self.leave_scope();
        result
    }

    pub fn parse_lambda_call(&mut self, left: ASTNode) -> ASTNode {
        self.consume_token();
        let lambda = self.parse_lambda();
        let arguments = match left {
            ASTNode::FunctionCallArgs(arguments) => arguments,
            _ => vec![left],
        };
        ASTNode::LambdaCall {
            lambda: Box::new(lambda),
            arguments,
        }
    }

    pub fn is_lambda_call(&mut self) -> bool {
        self.pos += 1;
        let next_token = self.get_current_token();
        self.pos -= 1;
        next_token == Some(Token::BackSlash)
    }
}
