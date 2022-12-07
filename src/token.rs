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

pub struct Tokenizer {
    cursor: usize,
    iterator: Peekable<Chars<'static>>,
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

impl Tokenizer {
    pub fn new(json: &'static str) -> Tokenizer {
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
