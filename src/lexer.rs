use std::{iter::Peekable, str::Chars};

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
    SlashEqual,    // /*
    Asterisk,      // *
    AsteriskEqual, // *=
    LeftParen,     // (
    RightParen,    // )
    LeftBracket,   // [
    RightBracket,  // ]
    LeftBrace,     // {
    RightBrace,    // }

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

#[derive(Debug)]
pub enum LexerErrorType {
    UnterminatedString,
    IntegerOverflow,
    UnknownCharacter(char),
}

#[derive(Debug)]
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

    fn next_char_if_neq(&mut self, expected: char) -> Option<char> {
        self.next_char_if(|peek| peek != expected)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.next_char();
        }
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

            '"' => {
                let mut string_literal = String::new();
                while let Some(next_char) = self.next_char_if_neq('"') {
                    string_literal.push(next_char);
                }

                if self.next_char().is_none() {
                    return Err(LexerError {
                        error_type: LexerErrorType::UnterminatedString,
                        line,
                        column,
                    });
                }

                TokenType::StringLiteral(string_literal)
            }

            current_char if (current_char == '_') || current_char.is_ascii_alphabetic() => {
                let mut identifier = String::from(current_char);
                while let Some(next_char) = self.next_char_if(|next_char| {
                    (next_char == '_') || next_char.is_ascii_alphanumeric()
                }) {
                    identifier.push(next_char);
                }

                let keyword_type = match identifier.as_str() {
                    "true" => Some(KeywordType::True),
                    "false" => Some(KeywordType::False),
                    "fn" => Some(KeywordType::Fn),

                    _ => None,
                };

                keyword_type.map_or_else(|| TokenType::Identifier(identifier), TokenType::Keyword)
            }

            current_char if current_char.is_ascii_digit() => {
                let mut integer_literal = String::from(current_char);
                while let Some(next_char) =
                    self.next_char_if(|next_char| next_char.is_ascii_digit())
                {
                    integer_literal.push(next_char);
                }

                TokenType::IntegerLiteral(integer_literal.parse::<i64>().map_err(|_| {
                    LexerError {
                        error_type: LexerErrorType::IntegerOverflow,
                        line,
                        column,
                    }
                })?)
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
mod tests {
    use super::*;

    #[test]
    fn basic_operators() -> Result<(), LexerError> {
        let source = r"
            = ==
            ! !=
            + +=
            - -=
            / /=
            * *=
        ";

        let expected = [
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::Plus,
            TokenType::PlusEqual,
            TokenType::Minus,
            TokenType::MinusEqual,
            TokenType::Slash,
            TokenType::SlashEqual,
            TokenType::Asterisk,
            TokenType::AsteriskEqual,
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            let token = token?;
            assert_eq!(token.token_type, expected[i]);
        }

        Ok(())
    }

    #[test]
    fn basic_delimiters() -> Result<(), LexerError> {
        let source = r"()[]{}";

        let expected = [
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::LeftBrace,
            TokenType::RightBrace,
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            let token = token?;
            assert_eq!(token.token_type, expected[i]);
        }

        Ok(())
    }

    #[test]
    fn basic_keywords() -> Result<(), LexerError> {
        let source = r"
            true
            false
            fn
        ";

        let expected = [
            TokenType::Keyword(KeywordType::True),
            TokenType::Keyword(KeywordType::False),
            TokenType::Keyword(KeywordType::Fn),
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            let token = token?;
            assert_eq!(token.token_type, expected[i]);
        }

        Ok(())
    }

    #[test]
    fn basic_identifiers() -> Result<(), LexerError> {
        let source = r"
            _test
            test
            test_
            test_test
            test1
            test_2
        ";

        let expected = [
            TokenType::Identifier(String::from("_test")),
            TokenType::Identifier(String::from("test")),
            TokenType::Identifier(String::from("test_")),
            TokenType::Identifier(String::from("test_test")),
            TokenType::Identifier(String::from("test1")),
            TokenType::Identifier(String::from("test_2")),
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            let token = token?;
            assert_eq!(token.token_type, expected[i]);
        }

        Ok(())
    }

    #[test]
    fn basic_literals() -> Result<(), LexerError> {
        let source = r#"
            1
            1234
            "test"
            "test with spaces"
        "#;

        let expected = [
            TokenType::IntegerLiteral(1),
            TokenType::IntegerLiteral(1234),
            TokenType::StringLiteral(String::from("test")),
            TokenType::StringLiteral(String::from("test with spaces")),
        ];

        let lexer = Lexer::new(source);
        for (i, token) in lexer.enumerate() {
            let token = token?;
            assert_eq!(token.token_type, expected[i]);
        }

        Ok(())
    }
}
