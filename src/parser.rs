/* Copyright 2022 Bruno Flavio Ferreira
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*   https://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*/

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

macro_rules! parse_hex_digit {
    ($self: ident, $c: ident, $hex: ident, $index: expr) => {
        let token = $self.tokenizer.next_token();
        $c = Parser::token_to_char(token)?;
        if !is_hex_digit($c) {
            return Err(ParseError::new("Expected a hex digit after a \\u"));
        }
        $hex = $hex | (hex_char_to_u32($c) << ($index * 4))
    };
}
pub struct Parser {
    tokenizer: Tokenizer,
}

pub struct ParseError {
    pub details: String,
}

impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        return ParseError {
            details: msg.to_string(),
        };
    }
}

type BoxResult<T> = Result<T, ParseError>;
type JsonObject = HashMap<String, JsonElement>;

fn is_hex_digit(c: char) -> bool {
    return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}

fn hex_char_to_u32(c: char) -> u32 {
    if c >= '0' && c <= '9' {
        return (c as u32 - '0' as u32) as u32;
    }
    if c >= 'a' && c <= 'f' {
        return (c as u32 - 'a' as u32) as u32;
    }
    return (c as u32 - 'A' as u32) as u32;
}

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
            c = Parser::token_to_char(token)?;
            match c {
                '"' => break,
                '\\' => {
                    token = self.tokenizer.next_token();
                    c = Parser::token_to_char(token)?;
                    match c {
                        '"' => string.push('\"'),
                        '\\' => string.push('\\'),
                        '/' => string.push('/'),
                        'b' => string.push(0x08 as char),
                        'n' => string.push('\n'),
                        'r' => string.push('\r'),
                        't' => string.push('\t'),
                        'u' => {
                            let mut hex: u32 = 0;
                            parse_hex_digit!(self, c, hex, 3);
                            parse_hex_digit!(self, c, hex, 2);
                            parse_hex_digit!(self, c, hex, 1);
                            parse_hex_digit!(self, c, hex, 0);
                            match char::from_u32(hex) {
                                Some(uc) => string.push(uc),
                                None => {
                                    return Err(ParseError::new(&format!(
                                        "Invalid unicode character {:x}",
                                        hex
                                    )))
                                }
                            }
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
                let name = name_or_error?;
                
                self.consume_whitespace();
                c = self.tokenizer.next_token();
                if !matches!(c, Token::Colon) {
                    return Err(ParseError::new("Expected a colon"));
                }
                
                self.consume_whitespace();
                let value = self.parse_value();
                map.insert(name, value?);
                self.consume_whitespace();
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
                let value = self.parse_value();
                array.push(value?);
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
                return Ok(JsonElement::Object(obj?));
            }
            Token::OpenSquareBracket => {
                let array = self.parse_array();
                return Ok(JsonElement::Array(array?));
            }
            Token::Quotion => {
                let string = self.parse_string();
                return Ok(JsonElement::Str(string?));
            }
            Token::Character(c) => match c {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => {
                    let num = self.parse_number();
                    return Ok(JsonElement::Number(num?));
                }
                't' | 'f' => {
                    let boolean = self.parse_boolean();
                    return Ok(JsonElement::Boolean(boolean?));
                }
                'n' => {
                    self.parse_null()?;
                    return Ok(JsonElement::Null);
                }
                _ => return Err(ParseError::new("Expected true, false or null")),
            },
            _ => {
                return Err(ParseError::new("Invalid json value"));
            }
        };
    }

    pub fn parse (json: &str) -> BoxResult<JsonElement> {
        let mut parser = Parser {
            tokenizer: Tokenizer::new(json),
        };
        parser.consume_whitespace();
        let element = parser.parse_value();
        parser.consume_whitespace();
        return element;
    }
}
