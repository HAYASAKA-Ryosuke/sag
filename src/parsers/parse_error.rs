use crate::token::Token;


#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(message: &str, token: &Token) -> Self {
        Self {
            message: message.to_string(),
            line: token.line,
            column: token.column,
        }
    }

    pub fn display_with_source(&self, source: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let error_line = lines.get(self.line - 1).unwrap_or(&"");
        eprintln!("Error: {}", self.message);
        eprintln!(" --> line {}, column {}", self.line, self.column);
        eprintln!(" | {}", error_line);
        eprint!(" | ");
        for _ in 1..self.column {
            eprint!(" ");
        }
        eprintln!("^");
    }
}
