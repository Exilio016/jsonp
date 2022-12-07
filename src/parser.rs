use crate::json_element::JsonElement;
use crate::token::Token;
use crate::token::Tokenizer;

use std::collections::HashMap;

macro_rules! unbox_token_char {
    ($t:expr, $e:expr, $c:ident) => {
        match $t {
            Token::Character(c) => $c = c,
            _ => return Err($e),
        }
    };
}

macro_rules! expect_char {
    ($token: expr, $c: expr) => {
        let charac: char;
        unbox_token_char!($token, ParseError::new("Expected a $c"), charac);
        if charac != $c {
            return Err(ParseError::new("Expected a $c"));
        }
    };
}

macro_rules! peek_character_or_return {
    ($string: ident, $self: ident, $c: ident) => {
        let token = $self.tokenizer.peek_token();
        match token {
            Token::Character(nc) => $c = nc,
            _ => return Ok($string.parse::<f64>().unwrap()),
        };
    };
    ($string: ident, $self: ident, $c: ident, $error: expr) => {
        let token = $self.tokenizer.peek_token();
        match token {
            Token::Character(nc) => $c = nc,
            _ => return Err($error),
        };
    };
}
macro_rules! add_c_to_str_and_peek_character {
    ($string: ident, $self: ident, $c: ident) => {
        $string.push($c);
        $self.tokenizer.next_token();
        peek_character_or_return!($string, $self, $c);
    };
    ($string: ident, $self: ident, $c: ident, $error: expr) => {
        $string.push($c);
        $self.tokenizer.next_token();
        peek_character_or_return!($string, $self, $c, $error);
    };
}

pub struct Parser {
    tokenizer: Tokenizer,
}

pub struct ParseError {
    pub details: String,
}

impl ParseError {
    fn new(msg: &str) -> ParseError {
        return ParseError {
            details: msg.to_string(),
        };
    }
}

type BoxResult<T> = Result<T, ParseError>;
type JsonObject = HashMap<String, JsonElement>;

impl Parser {
    fn consume_whitespace(&mut self) {
        loop {
            let token = self.tokenizer.peek_token();
            match token {
                Token::Character(c) => match c {
                    ' ' | '\n' | '\r' | '\t' => {
                        self.tokenizer.next_token();
                    }
                    _ => break,
                },
                _ => break,
            }
        }
    }

    fn token_to_char(token: Token) -> BoxResult<char> {
        let c: char;
        match token {
            Token::Character(character) => c = character,
            Token::CloseBracket => c = '}',
            Token::OpenBracket => c = '{',
            Token::CloseSquareBracket => c = ']',
            Token::OpenSquareBracket => c = '[',
            Token::Colon => c = ':',
            Token::Comma => c = ',',
            Token::Quotion => c = '"',
            Token::End => return Err(ParseError::new("Json ended without closing string")),
        }
        return Ok(c);
    }

    fn parse_string(&mut self) -> BoxResult<String> {
        let mut string = String::new();
        let mut c: char;
        let mut token: Token;
        self.tokenizer.next_token();

        loop {
            token = self.tokenizer.next_token();
            match Parser::token_to_char(token) {
                Ok(character) => c = character,
                Err(e) => return Err(e),
            }
            match c {
                '"' => break,
                '\\' => {
                    token = self.tokenizer.next_token();
                    match Parser::token_to_char(token) {
                        Ok(character) => c = character,
                        Err(e) => return Err(e),
                    }

                    match c {
                        '"' => string.push('\"'),
                        '\\' => string.push('\\'),
                        '/' => string.push('/'),
                        'b' => string.push(0x08 as char),
                        'n' => string.push('\n'),
                        'r' => string.push('\r'),
                        't' => string.push('\t'),
                        'u' => {
                            //FIXME parse unicode character
                        }
                        _ => {
                            return Err(ParseError::new(&format!("Unknown escaped character {c}")))
                        }
                    }
                }
                _ => string.push(c),
            }
        }

        return Ok(string);
    }

    fn parse_object(&mut self) -> BoxResult<JsonObject> {
        self.tokenizer.next_token();
        self.consume_whitespace();
        let mut c = self.tokenizer.peek_token();
        let mut map: JsonObject = HashMap::new();
        if matches!(c, Token::CloseBracket) {
            self.tokenizer.next_token();
            return Ok(map);
        } else {
            loop {
                self.consume_whitespace();
                let name_or_error = self.parse_string();
                match name_or_error {
                    Ok(name) => {
                        self.consume_whitespace();
                        c = self.tokenizer.next_token();
                        if !matches!(c, Token::Colon) {
                            return Err(ParseError::new("Expected a colon"));
                        }
                        self.consume_whitespace();
                        let value_or_error = self.parse_value();
                        match value_or_error {
                            Ok(value) => {
                                map.insert(name, value);
                            }
                            Err(e) => return Err(e),
                        }
                        self.consume_whitespace();
                    }
                    Err(e) => return Err(e),
                }
                c = self.tokenizer.peek_token();
                if !matches!(c, Token::Comma) {
                    break;
                } else {
                    self.tokenizer.next_token();
                }
            }
            c = self.tokenizer.next_token();
            match c {
                Token::CloseBracket => return Ok(map),
                _ => return Err(ParseError::new("Expecting a '}'")),
            }
        }
    }

    fn parse_array(&mut self) -> BoxResult<Vec<JsonElement>> {
        self.tokenizer.next_token();
        self.consume_whitespace();
        let mut c = self.tokenizer.peek_token();
        let mut array: Vec<JsonElement> = Vec::new();
        if matches!(c, Token::CloseSquareBracket) {
            self.tokenizer.next_token();
            return Ok(array);
        } else {
            loop {
                self.consume_whitespace();
                let value_or_error = self.parse_value();
                match value_or_error {
                    Ok(value) => {
                        array.push(value);
                    }
                    Err(e) => return Err(e),
                }
                self.consume_whitespace();
                c = self.tokenizer.peek_token();
                if !matches!(c, Token::Comma) {
                    break;
                } else {
                    self.tokenizer.next_token();
                }
            }
        }
        return Ok(array);
    }

    fn parse_number(&mut self) -> BoxResult<f64> {
        let token = self.tokenizer.peek_token();
        let mut string = String::new();
        match token {
            Token::Character(mut c) => {
                if c == '0' {
                    add_c_to_str_and_peek_character!(string, self, c);
                    if c != '.' && c != 'e' && c != 'E' {
                        return Err(ParseError::new("Expected a '.' or 'e' or 'E'"));
                    }
                } else if c == '-' {
                    add_c_to_str_and_peek_character!(
                        string,
                        self,
                        c,
                        ParseError::new("Expected a number after a '-'")
                    );
                    if c == '0' {
                        add_c_to_str_and_peek_character!(string, self, c);
                        if c != '.' && c != 'e' && c != 'E' {
                            return Err(ParseError::new("Expected a '.' or 'e' or 'E'"));
                        }
                    }
                } else {
                    add_c_to_str_and_peek_character!(string, self, c);
                }
                while c >= '0' && c <= '9' {
                    add_c_to_str_and_peek_character!(string, self, c);
                }
                if c == '.' {
                    add_c_to_str_and_peek_character!(string, self, c);
                    if c < '0' || c > '9' {
                        return Err(ParseError::new("Expected a digit"));
                    }
                    while c >= '0' && c <= '9' {
                        add_c_to_str_and_peek_character!(string, self, c);
                    }
                }
                if c == 'E' || c == 'e' {
                    add_c_to_str_and_peek_character!(string, self, c);
                    if c == '-' || c == '+' {
                        add_c_to_str_and_peek_character!(string, self, c);
                    }
                    if c < '0' || c > '9' {
                        return Err(ParseError::new("Expected a digit"));
                    }
                    while c >= '0' && c <= '9' {
                        add_c_to_str_and_peek_character!(string, self, c);
                    }
                }
                return Ok(string.parse::<f64>().unwrap());
            }
            _ => {
                self.tokenizer.next_token();
                return Err(ParseError::new("Expected a number"));
            }
        }
    }

    fn parse_boolean(&mut self) -> BoxResult<bool> {
        let token = self.tokenizer.next_token();
        match token {
            Token::Character(c) => {
                if c == 't' {
                    expect_char!(self.tokenizer.next_token(), 'r');
                    expect_char!(self.tokenizer.next_token(), 'u');
                    expect_char!(self.tokenizer.next_token(), 'e');
                    return Ok(true);
                }
                if c == 'f' {
                    expect_char!(self.tokenizer.next_token(), 'a');
                    expect_char!(self.tokenizer.next_token(), 'l');
                    expect_char!(self.tokenizer.next_token(), 's');
                    expect_char!(self.tokenizer.next_token(), 'e');
                    return Ok(false);
                }
                return Err(ParseError::new("Expected a 'f' or 't'"));
            }
            _ => return Err(ParseError::new("Expected a 'f' or 't'")),
        }
    }
    fn parse_null(&mut self) -> BoxResult<i8> {
        expect_char!(self.tokenizer.next_token(), 'n');
        expect_char!(self.tokenizer.next_token(), 'u');
        expect_char!(self.tokenizer.next_token(), 'l');
        expect_char!(self.tokenizer.next_token(), 'l');
        return Ok(0);
    }

    fn parse_value(&mut self) -> BoxResult<JsonElement> {
        let token = self.tokenizer.peek_token();

        match token {
            Token::OpenBracket => {
                let obj = self.parse_object();
                return match obj {
                    Ok(o) => Ok(JsonElement::Object(o)),
                    Err(e) => Err(e),
                };
            }
            Token::OpenSquareBracket => {
                let array = self.parse_array();
                return match array {
                    Ok(o) => Ok(JsonElement::Array(o)),
                    Err(e) => Err(e),
                };
            }
            Token::Quotion => {
                let string = self.parse_string();
                return match string {
                    Ok(s) => Ok(JsonElement::Str(s)),
                    Err(e) => Err(e),
                };
            }
            Token::Character(c) => match c {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => {
                    let num = self.parse_number();
                    match num {
                        Ok(value) => return Ok(JsonElement::Number(value)),
                        Err(e) => return Err(e),
                    }
                }
                't' | 'f' => {
                    let boolean = self.parse_boolean();
                    match boolean {
                        Ok(value) => return Ok(JsonElement::Boolean(value)),
                        Err(e) => return Err(e),
                    }
                }
                'n' => {
                    let value = self.parse_null();
                    match value {
                        Err(e) => return Err(e),
                        Ok(_) => return Ok(JsonElement::Null),
                    }
                }
                _ => return Err(ParseError::new("Expected true, false or null")),
            },
            _ => {
                return Err(ParseError::new("Invalid json value"));
            }
        };
    }

    pub fn parse(json: &'static str) -> BoxResult<JsonElement> {
        let mut parser = Parser {
            tokenizer: Tokenizer::new(json),
        };
        parser.consume_whitespace();
        let element = parser.parse_value();
        parser.consume_whitespace();
        return element;
    }
}
