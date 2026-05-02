use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Plus,
    PlusEqual,

    Minus,
    MinusEqual,

    Slash,
    SlashEqual,

    Asterisk,
    AsteriskEqual,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,

    pub line: usize,
    pub column: usize,
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

            line: 1,
            column: 0,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next().inspect(|&next| match next {
            '\n' => {
                self.line += 1;
                self.column = 0;
            }

            _ => {
                self.column += 1;
            }
        })
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.next_char();
        }
    }

    fn read_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let line = self.line;
        let column = self.column + 1;

        let token_type = match self.next_char()? {
            '+' => match self.peek_char() {
                Some('=') => {
                    self.next_char()?;
                    TokenType::PlusEqual
                }

                _ => TokenType::Plus,
            },

            '-' => match self.peek_char() {
                Some('=') => {
                    self.next_char()?;
                    TokenType::MinusEqual
                }

                _ => TokenType::Minus,
            },

            '/' => match self.peek_char() {
                Some('=') => {
                    self.next_char()?;
                    TokenType::SlashEqual
                }

                _ => TokenType::Slash,
            },

            '*' => match self.peek_char() {
                Some('=') => {
                    self.next_char()?;
                    TokenType::AsteriskEqual
                }

                _ => TokenType::Asterisk,
            },

            value => todo!("unhandled character: {value:?}"),
        };

        Some(Token {
            token_type,
            line,
            column,
        })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let source = r"
            +-/*
            +=
            -=
            /=
            *=
        ";

        let expected = [
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Slash,
            TokenType::Asterisk,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::SlashEqual,
            TokenType::AsteriskEqual,
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            assert_eq!(token.token_type, expected[i]);
        }
    }
}
