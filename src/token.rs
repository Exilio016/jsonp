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

use std::iter::Peekable;
use std::str::Chars;

pub enum Token {
    OpenBracket,
    CloseBracket,
    OpenSquareBracket,
    CloseSquareBracket,
    Quotion,
    Character(char),
    End,
    Colon,
    Comma,
}

pub struct Tokenizer<'a> {
    cursor: usize,
    iterator: Peekable<Chars<'a>>,
}

fn match_token(c: char) -> Token {
    return match c {
        '{' => Token::OpenBracket,
        '}' => Token::CloseBracket,
        '[' => Token::OpenSquareBracket,
        ']' => Token::CloseSquareBracket,
        '"' => Token::Quotion,
        ':' => Token::Colon,
        ',' => Token::Comma,
        _ => Token::Character(c),
    };
}

impl<'a> Tokenizer<'a> {
    pub fn new(json: &str) -> Tokenizer {
        Tokenizer {
            cursor: 0,
            iterator: json.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        let c: char;
        let next = self.iterator.next();
        match next {
            Some(character) => c = character,
            None => return Token::End,
        }
        let token = match_token(c);
        self.cursor += 1;
        return token;
    }

    pub fn peek_token(&mut self) -> Token {
        let c: char;
        let next = self.iterator.peek();
        match next {
            Some(character) => c = *character,
            None => return Token::End,
        }
        let token = match_token(c);
        return token;
    }
}
