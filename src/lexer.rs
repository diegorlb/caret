use std::{iter::Peekable, str::Chars};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordType {
    True,
    False,
    Fn,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Equal,         // =
    EqualEqual,    // ==
    Bang,          // !
    BangEqual,     // !=
    Plus,          // +
    PlusEqual,     // +=
    Minus,         // -
    MinusEqual,    // -=
    Slash,         // /
    SlashEqual,    // /=
    Asterisk,      // *
    AsteriskEqual, // *=
    LeftParen,     // (
    RightParen,    // )
    LeftBracket,   // [
    RightBracket,  // ]
    LeftBrace,     // {
    RightBrace,    // }
    Comma,         // ,
    Colon,         // :
    Semicolon,     // ;

    Keyword(KeywordType),
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,

    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Error)]
pub enum LexerErrorType {
    #[error("unterminated string")]
    UnterminatedString,

    #[error("integer overflow")]
    IntegerOverflow,

    #[error("unknown character: {0:?}")]
    UnknownCharacter(char),
}

#[derive(Debug, Error)]
#[error("{error_type} at line {line}, column {column}")]
pub struct LexerError {
    pub error_type: LexerErrorType,

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

    fn next_char_if(&mut self, condition: impl FnOnce(char) -> bool) -> Option<char> {
        match self.peek_char() {
            Some(peek) if condition(peek) => self.next_char(),
            _ => None,
        }
    }

    fn next_char_if_eq(&mut self, expected: char) -> Option<char> {
        self.next_char_if(|peek| peek == expected)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.next_char();
        }
    }

    fn read_string_literal(&mut self, line: usize, column: usize) -> Result<TokenType, LexerError> {
        let mut string_literal = String::new();
        loop {
            match self.next_char() {
                Some('"') => break,
                Some(ch) => string_literal.push(ch),
                None => {
                    return Err(LexerError {
                        error_type: LexerErrorType::UnterminatedString,
                        line,
                        column,
                    });
                }
            }
        }

        Ok(TokenType::StringLiteral(string_literal))
    }

    fn read_integer_literal(
        &mut self,
        current_char: char,
        line: usize,
        column: usize,
    ) -> Result<TokenType, LexerError> {
        let mut integer_literal = String::from(current_char);
        while let Some(ch) = self.next_char_if(|ch| ch.is_ascii_digit()) {
            integer_literal.push(ch);
        }

        Ok(TokenType::IntegerLiteral(
            integer_literal.parse::<i64>().map_err(|_| LexerError {
                error_type: LexerErrorType::IntegerOverflow,
                line,
                column,
            })?,
        ))
    }

    fn read_identifier_or_keyword(&mut self, current_char: char) -> TokenType {
        let mut identifier = String::from(current_char);
        while let Some(ch) = self.next_char_if(|ch| (ch == '_') || ch.is_ascii_alphanumeric()) {
            identifier.push(ch);
        }

        let keyword_type = match identifier.as_str() {
            "true" => Some(KeywordType::True),
            "false" => Some(KeywordType::False),
            "fn" => Some(KeywordType::Fn),

            _ => None,
        };

        keyword_type.map_or_else(|| TokenType::Identifier(identifier), TokenType::Keyword)
    }

    fn read_token(&mut self) -> Result<Option<Token>, LexerError> {
        self.skip_whitespace();

        let Some(current_char) = self.next_char() else {
            return Ok(None);
        };

        let line = self.line;
        let column = self.column;

        let token_type = match current_char {
            '=' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::EqualEqual,
                None => TokenType::Equal,
            },

            '!' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::BangEqual,
                None => TokenType::Bang,
            },

            '+' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::PlusEqual,
                None => TokenType::Plus,
            },

            '-' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::MinusEqual,
                None => TokenType::Minus,
            },

            '/' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::SlashEqual,
                None => TokenType::Slash,
            },

            '*' => match self.next_char_if_eq('=') {
                Some(_) => TokenType::AsteriskEqual,
                None => TokenType::Asterisk,
            },

            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,

            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,

            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,

            ',' => TokenType::Comma,
            ':' => TokenType::Colon,
            ';' => TokenType::Semicolon,

            '"' => self.read_string_literal(line, column)?,

            current_char if current_char.is_ascii_digit() => {
                self.read_integer_literal(current_char, line, column)?
            }

            current_char if (current_char == '_') || current_char.is_ascii_alphabetic() => {
                self.read_identifier_or_keyword(current_char)
            }

            current_char => {
                return Err(LexerError {
                    error_type: LexerErrorType::UnknownCharacter(current_char),
                    line,
                    column,
                });
            }
        };

        Ok(Some(Token {
            token_type,
            line,
            column,
        }))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_token().transpose()
    }
}

#[cfg(test)]
macro_rules! test_lexer {
    ($name:ident, [$($char:literal => $expected:expr),+]) => {
        #[test]
        fn $name() -> Result<(), LexerError> {
            let source = concat!($($char, " "),+);
            let expected = [$($expected),+];

            let lexer = Lexer::new(source);
            let tokens = lexer.collect::<Result<Vec<_>, LexerError>>()?;

            assert_eq!(tokens.len(), expected.len());

            for (token, expected) in tokens.iter().zip(expected.iter()) {
                assert_eq!(token.token_type, *expected);
            }

            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    test_lexer!(operators, [
        "="  => TokenType::Equal,
        "==" => TokenType::EqualEqual,
        "!"  => TokenType::Bang,
        "!=" => TokenType::BangEqual,
        "+"  => TokenType::Plus,
        "+=" => TokenType::PlusEqual,
        "-"  => TokenType::Minus,
        "-=" => TokenType::MinusEqual,
        "/"  => TokenType::Slash,
        "/=" => TokenType::SlashEqual,
        "*"  => TokenType::Asterisk,
        "*=" => TokenType::AsteriskEqual
    ]);

    test_lexer!(delimiters, [
        "(" => TokenType::LeftParen,
        ")" => TokenType::RightParen,
        "[" => TokenType::LeftBracket,
        "]" => TokenType::RightBracket,
        "{" => TokenType::LeftBrace,
        "}" => TokenType::RightBrace
    ]);

    test_lexer!(keywords, [
        "true"  => TokenType::Keyword(KeywordType::True),
        "false" => TokenType::Keyword(KeywordType::False),
        "fn"    => TokenType::Keyword(KeywordType::Fn)
    ]);

    test_lexer!(identifiers, [
        "_test"     => TokenType::Identifier(String::from("_test")),
        "test"      => TokenType::Identifier(String::from("test")),
        "test_"     => TokenType::Identifier(String::from("test_")),
        "test_test" => TokenType::Identifier(String::from("test_test")),
        "test1"     => TokenType::Identifier(String::from("test1")),
        "test_2"    => TokenType::Identifier(String::from("test_2"))
    ]);

    test_lexer!(literals, [
        "1"                    => TokenType::IntegerLiteral(1),
        "1234"                 => TokenType::IntegerLiteral(1234),
        "\"test\""             => TokenType::StringLiteral(String::from("test")),
        "\"test with spaces\"" => TokenType::StringLiteral(String::from("test with spaces"))
    ]);
}
