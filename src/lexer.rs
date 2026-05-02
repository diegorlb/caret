use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    EOF,

    PLUS,
    MINUS,
    SLASH,
    ASTERISK,
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,

    line: usize,
    column: usize,
}

pub struct Lexer<'c> {
    chars: Peekable<Chars<'c>>,

    line: usize,
    column: usize,
}

impl<'c> Lexer<'c> {
    #[must_use]
    pub fn new(source: &'c str) -> Self {
        Self {
            chars: source.chars().peekable(),

            line: 0,
            column: 0,
        }
    }

    fn next_char(&mut self) -> char {
        match self.chars.next() {
            Some(next) => {
                self.column += 1;

                next
            }

            None => '0',
        }
    }

    fn peek_char(&mut self) -> &char {
        self.chars.peek().unwrap_or(&'0')
    }

    fn read_token(&mut self) -> Token {
        let token_type = match self.next_char() {
            '0' => TokenType::EOF,

            '+' => TokenType::PLUS,
            '-' => TokenType::MINUS,
            '/' => TokenType::SLASH,
            '*' => TokenType::ASTERISK,

            _ => todo!(""),
        };

        Token {
            token_type,
            line: self.line,
            column: self.column,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_token() {
            Token {
                token_type: TokenType::EOF,
                ..
            } => None,

            token => Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let expected = [
            TokenType::PLUS,
            TokenType::MINUS,
            TokenType::SLASH,
            TokenType::ASTERISK,
        ];

        let lexer = Lexer::new("+-/*");
        for (i, token) in lexer.enumerate() {
            assert_eq!(token.token_type, expected[i]);
        }
    }
}
