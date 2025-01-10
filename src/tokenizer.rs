use crate::token::Token;
use fraction::Fraction;

struct Tokenizer {
    tokens: Vec<Token>,
    chars: Vec<char>,
    pos: usize,
    nesting_count: usize,
}

impl Tokenizer {
    pub fn new(line: &String) -> Self {
        Tokenizer {
            pos: 0,
            chars: line.chars().collect(),
            tokens: vec![],
            nesting_count: 0,
        }
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

fn is_tab(c: &char) -> bool {
    *c == '\t'
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
                num = num
                    + Fraction::from(c.to_string().parse::<i32>().unwrap()) / Fraction::from(10);
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
        if c == '\0'
            || c == '\n'
            || c == ' '
            || c == ':'
            || c == ','
            || c == '('
            || c == ')'
            || c == '{'
            || c == '}'
            || c == '='
            || c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '%'
            || c == '.'
            || c == '|'
            || c == '<'
            || c == '>'
            || c == '\\'
            || c == '['
            || c == ']'
            || c == '\t'
        {
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

fn is_function_call_args(c: &char) -> bool {
    *c == '|'
}

fn is_line_comment(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "//".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_comment_block(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "```".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_import(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "import ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_from(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "from ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn get_line_comment_string(tokenizer: &mut Tokenizer) -> String {
    let mut comment = String::new();
    let mut pos = tokenizer.pos + 1;
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '\0' || c == '\n' {
            tokenizer.pos = pos;
            break;
        }
        comment += &c.to_string();
        pos += 1;
    }
    comment
}

fn get_comment_string(tokenizer: &mut Tokenizer) -> String {
    let mut comment = String::new();
    let mut pos = tokenizer.pos + 3;
    let mut back_quote_count = 0;
    let mut before_c = '\0';
    loop {
        let c = tokenizer.get_position_char(pos);
        if c == '\0' {
            tokenizer.pos = pos;
            break;
        }
        if back_quote_count == 3 {
            pos += 2;
            tokenizer.pos = pos;
            break;
        }
        if c == '`' && back_quote_count == 0 {
            back_quote_count += 1;
            before_c = c;
            pos += 1;
            continue;
        }
        if c == '`' && before_c == '`' {
            back_quote_count += 1;
            continue;
        }
        back_quote_count = 0;
        before_c = c;
        comment += &c.to_string();
        pos += 1;
    }
    comment
}

fn is_immutable(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "val ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_mutable(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "val mut ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_function(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "fun ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_return(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "return ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_match(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "match ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_right_arrow(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "->".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_right_rocket(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "=>".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_public_struct(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "pub struct ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_pub(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "pub ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_struct(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "struct ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_impl(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "impl ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_for(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "for ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_in(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "in ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_eq(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "==".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_lte(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "<=".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_lt(c: char) -> bool {
    c == '<'
}

fn is_gte(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in ">=".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_gt(c: char) -> bool {
    c == '>'
}

fn is_if(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "if ".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
        }
    }
    true
}

fn is_else(tokenizer: &mut Tokenizer) -> bool {
    for (i, c) in "else".chars().enumerate() {
        if c != tokenizer.get_position_char(i + tokenizer.pos) {
            return false;
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
            break;
        }
        if is_space(&c) {
            tokenizer.pos += 1;
            continue;
        }
        if is_tab(&c) {
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

        if is_line_comment(&mut tokenizer) {
            let _comment = get_line_comment_string(&mut tokenizer);
            //tokenizer.tokens.push(Token::CommentLine(comment));
            tokenizer.pos += 1;
            continue;
        }

        if is_comment_block(&mut tokenizer) {
            let _comment = get_comment_string(&mut tokenizer);
            //tokenizer.tokens.push(Token::CommentBlock(comment));
            continue;
        }

        if is_function(&mut tokenizer) {
            tokenizer.tokens.push(Token::Function);
            tokenizer.pos += 3;
            continue;
        }

        if is_import(&mut tokenizer) {
            tokenizer.tokens.push(Token::Import);
            tokenizer.pos += 6;
            continue;
        }

        if is_from(&mut tokenizer) {
            tokenizer.tokens.push(Token::From);
            tokenizer.pos += 5;
            continue;
        }

        if is_match(&mut tokenizer) {
            tokenizer.tokens.push(Token::Match);
            tokenizer.pos += 5;
            continue;
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

        if is_public_struct(&mut tokenizer) {
            tokenizer.tokens.push(Token::PublicStruct);
            tokenizer.pos += 10;
            continue;
        }

        if is_struct(&mut tokenizer) {
            tokenizer.tokens.push(Token::PrivateStruct);
            tokenizer.pos += 6;
            continue;
        }

        if is_impl(&mut tokenizer) {
            tokenizer.tokens.push(Token::Impl);
            tokenizer.pos += 4;
            continue;
        }

        if is_pub(&mut tokenizer) {
            tokenizer.tokens.push(Token::Pub);
            tokenizer.pos += 3;
            continue;
        }

        if is_for(&mut tokenizer) {
            tokenizer.tokens.push(Token::For);
            tokenizer.pos += 3;
            continue;
        }

        if is_in(&mut tokenizer) {
            tokenizer.tokens.push(Token::In);
            tokenizer.pos += 2;
            continue;
        }

        if is_right_rocket(&mut tokenizer) {
            tokenizer.tokens.push(Token::RRocket);
            tokenizer.pos += 2;
            continue;
        }

        if is_if(&mut tokenizer) {
            tokenizer.tokens.push(Token::If);
            tokenizer.pos += 2;
            continue;
        }

        if is_else(&mut tokenizer) {
            tokenizer.tokens.push(Token::Else);
            tokenizer.pos += 4;
            continue;
        }

        if is_eq(&mut tokenizer) {
            tokenizer.tokens.push(Token::Eq);
            tokenizer.pos += 2;
            continue;
        }

        if is_lte(&mut tokenizer) {
            tokenizer.tokens.push(Token::Lte);
            tokenizer.pos += 2;
            continue;
        }

        if is_lt(c) {
            tokenizer.tokens.push(Token::Lt);
            tokenizer.pos += 1;
            continue;
        }

        if is_gte(&mut tokenizer) {
            tokenizer.tokens.push(Token::Gte);
            tokenizer.pos += 2;
            continue;
        }

        if is_gt(c) {
            tokenizer.tokens.push(Token::Gt);
            tokenizer.pos += 1;
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

        if is_function_call_args(&c) {
            tokenizer.tokens.push(Token::Pipe);
            tokenizer.pos += 1;
            continue;
        }

        match c {
            '+' => tokenizer.tokens.push(Token::Plus),
            '-' => tokenizer.tokens.push(Token::Minus),
            '*' => tokenizer.tokens.push(Token::Mul),
            '/' => tokenizer.tokens.push(Token::Div),
            '%' => tokenizer.tokens.push(Token::Mod),
            '(' => tokenizer.tokens.push(Token::LParen),
            ')' => tokenizer.tokens.push(Token::RParen),
            '[' => tokenizer.tokens.push(Token::LBrancket),
            ']' => tokenizer.tokens.push(Token::RBrancket),
            '.' => tokenizer.tokens.push(Token::Dot),
            '\\' => tokenizer.tokens.push(Token::BackSlash),
            '{' => {
                tokenizer.nesting_count += 1;
                tokenizer.tokens.push(Token::LBrace)
            }
            '}' => {
                tokenizer.nesting_count -= 1;
                tokenizer.tokens.push(Token::RBrace);
                if tokenizer.nesting_count == 0 {
                    tokenizer.tokens.push(Token::Eof);
                }
            }
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
        assert_eq!(
            tokenize(&"-1 + 2 * 3/4 % 3".to_string()),
            vec![
                Token::Minus,
                Token::Number(Fraction::from(1)),
                Token::Plus,
                Token::Number(Fraction::from(2)),
                Token::Mul,
                Token::Number(Fraction::from(3)),
                Token::Div,
                Token::Number(Fraction::from(4)),
                Token::Mod,
                Token::Number(Fraction::from(3)),
                Token::Eof
            ]
        );
    }
    #[test]
    fn test_variable_definition() {
        assert_eq!(
            tokenize(&"val mut x = 1".to_string()),
            vec![
                Token::Mutable,
                Token::Identifier("x".into()),
                Token::Equal,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );
        assert_eq!(
            tokenize(&"val x: num = 1".to_string()),
            vec![
                Token::Immutable,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Identifier("num".into()),
                Token::Equal,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_multiline() {
        assert_eq!(
            tokenize(&"-1 + 2\n val x = 1".to_string()),
            vec![
                Token::Minus,
                Token::Number(Fraction::from(1)),
                Token::Plus,
                Token::Number(Fraction::from(2)),
                Token::Eof,
                Token::Immutable,
                Token::Identifier("x".into()),
                Token::Equal,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(
            tokenize(&"\"Hello World!!\"".to_string()),
            vec![Token::String("Hello World!!".into()), Token::Eof]
        );
    }

    #[test]
    fn test_function() {
        assert_eq!(
            tokenize(&"fun foo = (x:number, y: number): number {\n return x + y \n}".to_string()),
            vec![
                Token::Function,
                Token::Identifier("foo".into()),
                Token::Equal,
                Token::LParen,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Comma,
                Token::Identifier("y".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::RParen,
                Token::Colon,
                Token::Identifier("number".into()),
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Identifier("x".into()),
                Token::Plus,
                Token::Identifier("y".into()),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
    }
    #[test]
    fn test_call_function() {
        assert_eq!(
            tokenize(&"(x, y) -> foo".to_string()),
            vec![
                Token::LParen,
                Token::Identifier("x".into()),
                Token::Comma,
                Token::Identifier("y".into()),
                Token::RParen,
                Token::RArrow,
                Token::Identifier("foo".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_decimal_point() {
        assert_eq!(
            tokenize(&"1.5".to_string()),
            vec![Token::Number(Fraction::from(1.5)), Token::Eof]
        );
    }

    #[test]
    fn test_list() {
        assert_eq!(
            tokenize(&"[1, 2, 3]".to_string()),
            vec![
                Token::LBrancket,
                Token::Number(Fraction::from(1)),
                Token::Comma,
                Token::Number(Fraction::from(2)),
                Token::Comma,
                Token::Number(Fraction::from(3)),
                Token::RBrancket,
                Token::Eof
            ]
        );
        assert_eq!(
            tokenize(&"[\"Hello\", \"World\"]".to_string()),
            vec![
                Token::LBrancket,
                Token::String("Hello".into()),
                Token::Comma,
                Token::String("World".into()),
                Token::RBrancket,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_call_functions() {
        assert_eq!(
            tokenize(&"1 -> f1 -> f2".to_string()),
            vec![
                Token::Number(Fraction::from(1)),
                Token::RArrow,
                Token::Identifier("f1".into()),
                Token::RArrow,
                Token::Identifier("f2".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_lambda() {
        assert_eq!(
            tokenize(&"val inc = \\|x: number| => x + 1".to_string()),
            vec![
                Token::Immutable,
                Token::Identifier("inc".into()),
                Token::Equal,
                Token::BackSlash,
                Token::Pipe,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Pipe,
                Token::RRocket,
                Token::Identifier("x".into()),
                Token::Plus,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            tokenize(&"if x == 1 {\n return 1\n }".to_string()),
            vec![
                Token::If,
                Token::Identifier("x".into()),
                Token::Eq,
                Token::Number(Fraction::from(1)),
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(1)),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_else() {
        assert_eq!(
            tokenize(&"if x == 1 {\n return 1\n } else {\n return 0 \n}".to_string()),
            vec![
                Token::If,
                Token::Identifier("x".into()),
                Token::Eq,
                Token::Number(Fraction::from(1)),
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(1)),
                Token::Eof,
                Token::RBrace,
                Token::Eof,
                Token::Else,
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(0)),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_else_if() {
        assert_eq!(
            tokenize(&"if x == 1 {\n return 1\n } else if x == 2 {\n return 2 \n} else {\n return 0 \n}".to_string()),
            vec![
                Token::If,
                Token::Identifier("x".into()),
                Token::Eq,
                Token::Number(Fraction::from(1)),
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(1)),
                Token::Eof,
                Token::RBrace,
                Token::Eof,
                Token::Else,
                Token::If,
                Token::Identifier("x".into()),
                Token::Eq,
                Token::Number(Fraction::from(2)),
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(2)),
                Token::Eof,
                Token::RBrace,
                Token::Eof,
                Token::Else,
                Token::LBrace,
                Token::Eof,
                Token::Return,
                Token::Number(Fraction::from(0)),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_funtion_call_front() {
        assert_eq!(
            tokenize(&"f1()".to_string()),
            vec![
                Token::Identifier("f1".into()),
                Token::LParen,
                Token::RParen,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_comparison_operations() {
        assert_eq!(
            tokenize(&"1 == 1".to_string()),
            vec![
                Token::Number(Fraction::from(1)),
                Token::Eq,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );

        assert_eq!(
            tokenize(&"2 > 1".to_string()),
            vec![
                Token::Number(Fraction::from(2)),
                Token::Gt,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );

        assert_eq!(
            tokenize(&"3 >= 3".to_string()),
            vec![
                Token::Number(Fraction::from(3)),
                Token::Gte,
                Token::Number(Fraction::from(3)),
                Token::Eof
            ]
        );

        assert_eq!(
            tokenize(&"1 < 2".to_string()),
            vec![
                Token::Number(Fraction::from(1)),
                Token::Lt,
                Token::Number(Fraction::from(2)),
                Token::Eof
            ]
        );

        assert_eq!(
            tokenize(&"4 <= 4".to_string()),
            vec![
                Token::Number(Fraction::from(4)),
                Token::Lte,
                Token::Number(Fraction::from(4)),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_struct() {
        assert_eq!(
            tokenize(&"struct Point {\n x: number,\n y: number\n }".to_string()),
            vec![
                Token::PrivateStruct,
                Token::Identifier("Point".into()),
                Token::LBrace,
                Token::Eof,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Comma,
                Token::Eof,
                Token::Identifier("y".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
        assert_eq!(
            tokenize(&"pub struct Point {\n pub x: number,\n y: number\n }".to_string()),
            vec![
                Token::PublicStruct,
                Token::Identifier("Point".into()),
                Token::LBrace,
                Token::Eof,
                Token::Pub,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Comma,
                Token::Eof,
                Token::Identifier("y".into()),
                Token::Colon,
                Token::Identifier("number".into()),
                Token::Eof,
                Token::RBrace,
                Token::Eof
            ]
        );
    }
    #[test]
    fn test_struct_instance() {
        assert_eq!(
            tokenize(&"Point { x: 1, y: 2 }".to_string()),
            vec![
                Token::Identifier("Point".into()),
                Token::LBrace,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Number(Fraction::from(1)),
                Token::Comma,
                Token::Identifier("y".into()),
                Token::Colon,
                Token::Number(Fraction::from(2)),
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_assign_struct() {
        assert_eq!(
            tokenize(&"val point = Point { x: 1, y: 2 }".to_string()),
            vec![
                Token::Immutable,
                Token::Identifier("point".into()),
                Token::Equal,
                Token::Identifier("Point".into()),
                Token::LBrace,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Number(Fraction::from(1)),
                Token::Comma,
                Token::Identifier("y".into()),
                Token::Colon,
                Token::Number(Fraction::from(2)),
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_struct_field_access() {
        assert_eq!(
            tokenize(&"point.x".to_string()),
            vec![
                Token::Identifier("point".into()),
                Token::Dot,
                Token::Identifier("x".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_impl() {
        assert_eq!(
            tokenize(&"impl Point {\n fun x = (self: Point) {\n self.x\n }\n }".to_string()),
            vec![Token::Impl, Token::Identifier("Point".into()), Token::LBrace, Token::Eof, Token::Function, Token::Identifier("x".into()), Token::Equal, Token::LParen, Token::Identifier("self".into()), Token::Colon, Token::Identifier("Point".into()), Token::RParen, Token::LBrace, Token::Eof, Token::Identifier("self".into()), Token::Dot, Token::Identifier("x".into()), Token::Eof, Token::RBrace, Token::Eof, Token::RBrace, Token::Eof],
        )
    }

    #[test]
    fn test_comment_block() {
        assert_eq!(
            tokenize(&"```# Title\n## title1```".to_string()),
            vec![Token::Eof]
        );
    }

    #[test]
    fn test_commnet_line() {
        assert_eq!(
            tokenize(&"// comment".to_string()),
            vec![Token::Eof]
        );
    }

    #[test]
    fn test_add_tab() {
        assert_eq!(
            tokenize(&"1 + 2\t+ 3".to_string()),
            vec![
                Token::Number(Fraction::from(1)),
                Token::Plus,
                Token::Number(Fraction::from(2)),
                Token::Plus,
                Token::Number(Fraction::from(3)),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            tokenize(&"x[]".to_string()),
            vec![Token::Identifier("x".into()), Token::LBrancket, Token::RBrancket, Token::Eof]
        );
    }

    #[test]
    fn test_for() {
        assert_eq!(
            tokenize(&"for x in [1, 2, 3]".to_string()),
            vec![
                Token::For,
                Token::Identifier("x".into()),
                Token::In,
                Token::LBrancket,
                Token::Number(Fraction::from(1)),
                Token::Comma,
                Token::Number(Fraction::from(2)),
                Token::Comma,
                Token::Number(Fraction::from(3)),
                Token::RBrancket,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_import() {
        assert_eq!(
            tokenize(&"import foo1,foo2, foo3 from Foo".to_string()),
            vec![
                Token::Import,
                Token::Identifier("foo1".into()),
                Token::Comma,
                Token::Identifier("foo2".into()),
                Token::Comma,
                Token::Identifier("foo3".into()),
                Token::From,
                Token::Identifier("Foo".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_export() {
        assert_eq!(
            tokenize(&"pub foo1 = 1".to_string()),
            vec![
                Token::Pub,
                Token::Identifier("foo1".into()),
                Token::Equal,
                Token::Number(Fraction::from(1)),
                Token::Eof
            ]
        );
    }
}
