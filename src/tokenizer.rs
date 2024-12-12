struct Tokenizer {
    tokens: Vec<Token>,
    chars: Vec<char>,
    pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Immutable,
    Mutable,
    Print,
    Identifier(String),
    String(String),
    Num(i32),
    Equal,
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    Eof,
}

impl Tokenizer {
    pub fn new(line: &String) -> Self {
        Tokenizer{pos: 0, chars: line.chars().collect(), tokens: vec![]}
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

fn get_digit(tokenizer: &mut Tokenizer) -> i32 {
    let mut num = 0;
    let mut pos = tokenizer.pos;
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '\0' {
            break;
        }
        if is_digit(&c) {
            num = num * 10 + c.to_string().parse::<i32>().unwrap();
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
        if c == '\0' || c == '\n' || c == ' ' {
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

fn is_print(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "print".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false
        }
    }
    true
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

pub fn tokenize(line: &String) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(&line);
    loop {
        let c = tokenizer.get_position_char(tokenizer.pos);
        if is_line_break(&c) {
            tokenizer.tokens.push(Token::Eof);
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
            tokenizer.tokens.push(Token::Num(num));
            continue;
        }

        if is_string(&c) {
            let str = get_string(&mut tokenizer);
            tokenizer.tokens.push(Token::String(str));
            continue
        }

        if is_print(&mut tokenizer) {
            tokenizer.tokens.push(Token::Print);
            tokenizer.pos += 5;
            continue;
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

        match c {
            '+' => tokenizer.tokens.push(Token::Plus),
            '-' => tokenizer.tokens.push(Token::Minus),
            '*' => tokenizer.tokens.push(Token::Mul),
            '/' => tokenizer.tokens.push(Token::Div),
            '(' => tokenizer.tokens.push(Token::LParen),
            ')' => tokenizer.tokens.push(Token::RParen),
            '=' => tokenizer.tokens.push(Token::Equal),
            _ => {
                let value = get_identifier(&mut tokenizer);
                tokenizer.tokens.push(Token::Identifier(value))
            }
        }
        tokenizer.pos += 1;
    }
    tokenizer.tokens.push(Token::Eof);
    tokenizer.tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_basic_arithmetic_operations() {
        assert_eq!(tokenize(&"-1 + 2 * 3/4".to_string()), vec![Token::Minus, Token::Num(1), Token::Plus, Token::Num(2), Token::Mul, Token::Num(3), Token::Div, Token::Num(4), Token::Eof]);
        assert_eq!(tokenize(&"val mut x = 1".to_string()), vec![Token::Mutable , Token::Identifier("x".into()), Token::Equal, Token::Num(1), Token::Eof]);
        assert_eq!(tokenize(&"-1 + 2 \n val x = 1".to_string()), vec![Token::Minus, Token::Num(1), Token::Plus, Token::Num(2), Token::Eof, Token::Immutable , Token::Identifier("x".into()), Token::Equal, Token::Num(1), Token::Eof]);
        assert_eq!(tokenize(&"\"Hello World!!\"".to_string()), vec![Token::String("Hello World!!".into()), Token::Eof]);
    }
}
