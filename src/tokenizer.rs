use fraction::Fraction;

struct Tokenizer {
    tokens: Vec<Token>,
    chars: Vec<char>,
    pos: usize,
    nesting_count: usize
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Immutable,
    Mutable,
    Colon,
    Identifier(String),
    String(String),
    Number(Fraction),
    Void,
    Equal,
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Eof,
    Function,
    FunctionCallArgs,
    Return,
    Comma,
    RArrow,
    Match,
}

impl Tokenizer {
    pub fn new(line: &String) -> Self {
        Tokenizer{pos: 0, chars: line.chars().collect(), tokens: vec![], nesting_count: 0}
    }

    pub fn get_position_char(&self, pos: usize) -> char {
        if pos >= self.chars.len() {
            return '\0';
        }
        self.chars[pos]
    }
}

fn is_space(c: &char) -> bool {
    *c == ' '
}

fn is_digit(c: &char) -> bool {
    *c >= '0' && *c <= '9'
}

fn get_digit(tokenizer: &mut Tokenizer) -> Fraction {
    let mut num = Fraction::from(0);
    let mut pos = tokenizer.pos;
    let mut is_decimal_point = false;
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '\0' {
            break;
        }
        if is_digit(&c) {
            if is_decimal_point {
                num = num + Fraction::from(c.to_string().parse::<i32>().unwrap()) / Fraction::from(10);
            } else {
                num = num * 10 + Fraction::from(c.to_string().parse::<i32>().unwrap());
            }
            pos += 1;
        } else if c == '.' {
            is_decimal_point = true;
            pos += 1;
        } else {
            break;
        }
    }
    tokenizer.pos = pos;
    num
}

fn is_string(c: &char) -> bool {
    *c == '"'
}

fn get_identifier(tokenizer: &mut Tokenizer) -> String {
    let mut identifier = String::new();
    let mut pos = tokenizer.pos;
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '\0' || c == '\n' || c == ' ' || c == ':' || c == ',' || c == '(' || c == ')' {
            break;
        }
        identifier += &c.to_string();
        pos += 1;
    }
    tokenizer.pos = pos;
    identifier
}

fn get_string(tokenizer: &mut Tokenizer) -> String {
    let mut str = String::new();
    let mut pos = tokenizer.pos + 1;
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '"' {
            pos += 1;
            tokenizer.pos = pos;
            break;
        }
        if c == '\0' {
            break;
        }
        str += &c.to_string();
        pos += 1;
    }
    str
}

fn is_line_break(c: &char) -> bool {
    *c == '\n'
}

fn is_colon(c: &char) -> bool {
    *c == ':'
}

fn is_comma(c: &char) -> bool {
    *c == ','
}

fn is_semicoron(c: &char) -> bool {
    *c == ';'
}

fn is_immutable(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "val".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_mutable(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "val mut".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_function(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "fun".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_function_call_args(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "args".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_return(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "return".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_match(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "match".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

fn is_right_arrow(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "->".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
}

pub fn tokenize(line: &String) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(&line);
    loop {
        let c = tokenizer.get_position_char(tokenizer.pos);
        if is_line_break(&c) || is_semicoron(&c) {
            if tokenizer.tokens.last() != Some(&Token::Eof) {
                tokenizer.tokens.push(Token::Eof);
            }
            tokenizer.pos += 1;
            continue;
        }
        if c == '\0' {
            break
        }
        if is_space(&c) {
            tokenizer.pos += 1;
            continue;
        }
        if is_digit(&c) {
            let num = get_digit(&mut tokenizer);
            tokenizer.tokens.push(Token::Number(num));
            continue;
        }

        if is_string(&c) {
            let str = get_string(&mut tokenizer);
            tokenizer.tokens.push(Token::String(str));
            continue
        }

        if is_mutable(&mut tokenizer) {
            tokenizer.tokens.push(Token::Mutable);
            tokenizer.pos += 7;
            continue;
        }

        if is_immutable(&mut tokenizer) {
            tokenizer.tokens.push(Token::Immutable);
            tokenizer.pos += 3;
            continue;
        }

        if is_function(&mut tokenizer) {
            tokenizer.tokens.push(Token::Function);
            tokenizer.pos += 3;
            continue;
        }
        if is_function_call_args(&mut tokenizer) {
            tokenizer.tokens.push(Token::FunctionCallArgs);
            tokenizer.pos += 4;
            continue;
        }

        if is_match(&mut tokenizer) {
            tokenizer.tokens.push(Token::Match);
            tokenizer.pos += 5;
            continue
        }

        if is_return(&mut tokenizer) {
            tokenizer.tokens.push(Token::Return);
            tokenizer.pos += 6;
            continue;
        }

        if is_right_arrow(&mut tokenizer) {
            tokenizer.tokens.push(Token::RArrow);
            tokenizer.pos += 2;
            continue;
        }

        if is_colon(&c) {
            tokenizer.tokens.push(Token::Colon);
            tokenizer.pos += 1;
            continue;
        }

        if is_comma(&c) {
            tokenizer.tokens.push(Token::Comma);
            tokenizer.pos += 1;
            continue;
        }

        match c {
            '+' => tokenizer.tokens.push(Token::Plus),
            '-' => tokenizer.tokens.push(Token::Minus),
            '*' => tokenizer.tokens.push(Token::Mul),
            '/' => tokenizer.tokens.push(Token::Div),
            '(' => tokenizer.tokens.push(Token::LParen),
            ')' => tokenizer.tokens.push(Token::RParen),
            '{' => {
                tokenizer.nesting_count += 1;
                tokenizer.tokens.push(Token::LBrace)
            },
            '}' => {
                tokenizer.nesting_count -= 1;
                tokenizer.tokens.push(Token::RBrace);
                if tokenizer.nesting_count == 0 {
                    tokenizer.tokens.push(Token::Eof);
                }
            },
            '=' => tokenizer.tokens.push(Token::Equal),
            _ => {
                let value = get_identifier(&mut tokenizer);
                tokenizer.tokens.push(Token::Identifier(value));
                continue;
            }
        }
        tokenizer.pos += 1;
    }
    if tokenizer.tokens.last() != Some(&Token::Eof) {
        tokenizer.tokens.push(Token::Eof);
    }
    tokenizer.tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_basic_arithmetic_operations() {
        assert_eq!(tokenize(&"-1 + 2 * 3/4".to_string()), vec![Token::Minus, Token::Number(Fraction::from(1)), Token::Plus, Token::Number(Fraction::from(2)), Token::Mul, Token::Number(Fraction::from(3)), Token::Div, Token::Number(Fraction::from(4)), Token::Eof]);
    }
    #[test]
    fn test_variable_definition() {
        assert_eq!(tokenize(&"val mut x = 1".to_string()), vec![Token::Mutable , Token::Identifier("x".into()), Token::Equal, Token::Number(Fraction::from(1)), Token::Eof]);
        assert_eq!(tokenize(&"val x: num = 1".to_string()), vec![Token::Immutable, Token::Identifier("x".into()), Token::Colon, Token::Identifier("num".into()), Token::Equal, Token::Number(Fraction::from(1)), Token::Eof]);
    }

    #[test]
    fn test_multiline() {
        assert_eq!(tokenize(&"-1 + 2\n val x = 1".to_string()), vec![Token::Minus, Token::Number(Fraction::from(1)), Token::Plus, Token::Number(Fraction::from(2)), Token::Eof, Token::Immutable , Token::Identifier("x".into()), Token::Equal, Token::Number(Fraction::from(1)), Token::Eof]);
    }

    #[test]
    fn test_string() {
        assert_eq!(tokenize(&"\"Hello World!!\"".to_string()), vec![Token::String("Hello World!!".into()), Token::Eof]);
    }

    #[test]
    fn test_function() {
        assert_eq!(tokenize(&"fun foo = (x:number, y: number): number {\n return x + y \n}".to_string()), vec![Token::Function, Token::Identifier("foo".into()), Token::Equal, Token::LParen, Token::Identifier("x".into()), Token::Colon, Token::Identifier("number".into()), Token::Comma, Token::Identifier("y".into()), Token::Colon, Token::Identifier("number".into()), Token::RParen, Token::Colon, Token::Identifier("number".into()), Token::LBrace, Token::Eof, Token::Return, Token::Identifier("x".into()), Token::Plus, Token::Identifier("y".into()), Token::Eof, Token::RBrace, Token::Eof]);
    }
    #[test]
    fn test_call_function() {
        assert_eq!(tokenize(&"(x, y) -> foo".to_string()), vec![Token::LParen, Token::Identifier("x".into()), Token::Comma, Token::Identifier("y".into()), Token::RParen, Token::RArrow, Token::Identifier("foo".into()), Token::Eof]);
    }

    #[test]
    fn test_decimal_point() {
        assert_eq!(tokenize(&"1.5".to_string()), vec![Token::Number(Fraction::from(1.5)), Token::Eof]);
    }
}
