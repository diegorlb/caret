use std::iter::Peekable;

use thiserror::Error;

use crate::{
    ast::{
        BinaryOperation, BinaryOperator, Expression, FieldAccess, FunctionCall,
        FunctionDeclaration, Program, Statement, UnaryOperation, UnaryOperator,
        VariableDeclaration,
    },
    lexer::{KeywordType, Lexer, LexerError, TokenType},
};

type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    LexerError(#[from] LexerError),

    #[error("unexpected token: {token_type:?} at line {line}, column {column}")]
    UnexpectedToken {
        token_type: TokenType,
        line: usize,
        column: usize,
    },

    #[error("unexpected end of file at line {line}, column {column}")]
    UnexpectedEOF { line: usize, column: usize },
}

pub struct Parser<'c> {
    tokens: Peekable<Lexer<'c>>,

    last_line: usize,
    last_column: usize,
}

impl<'c> Parser<'c> {
    #[must_use]
    pub fn new(source: &'c str) -> Self {
        Self {
            tokens: Lexer::new(source).peekable(),
            last_line: 1,
            last_column: 0,
        }
    }

    fn peek_token(&mut self) -> ParserResult<Option<&TokenType>> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(Some(&token.token_type)),
            Some(Err(error)) => Err(ParserError::LexerError(error.clone())),
            None => Ok(None),
        }
    }

    fn next_token(&mut self) -> ParserResult<TokenType> {
        match self.tokens.next() {
            Some(Ok(token)) => {
                self.last_line = token.line;
                self.last_column = token.column;

                Ok(token.token_type)
            }

            Some(Err(err)) => Err(ParserError::LexerError(err)),

            None => Err(ParserError::UnexpectedEOF {
                line: self.last_line,
                column: self.last_column,
            }),
        }
    }

    fn next_token_if(
        &mut self,
        condition: impl FnOnce(&TokenType) -> bool,
    ) -> ParserResult<Option<TokenType>> {
        match self.peek_token()? {
            Some(token_type) if condition(token_type) => Ok(Some(self.next_token()?)),
            _ => Ok(None),
        }
    }

    fn expect_token(&mut self, expected: &TokenType) -> ParserResult<TokenType> {
        self.next_token().and_then(|token_type| {
            if token_type == *expected {
                Ok(token_type)
            } else {
                Err(ParserError::UnexpectedToken {
                    token_type,
                    line: self.last_line,
                    column: self.last_column,
                })
            }
        })
    }

    fn parse_comma_separated<T>(
        &mut self,
        mut map: impl FnMut(&mut Self) -> ParserResult<T>,
        end_token: &TokenType,
    ) -> ParserResult<Vec<T>> {
        let mut items = Vec::new();
        while self
            .peek_token()?
            .is_some_and(|token_type| token_type != end_token)
        {
            items.push(map(self)?);

            if self
                .next_token_if(|token_type| *token_type == TokenType::Comma)?
                .is_none()
            {
                break;
            }
        }

        Ok(items)
    }

    /// Returns the parse of this [`Parser`].
    ///
    /// # Errors
    /// This function will return an error if there were any errors whilst parsing tokens.
    pub fn parse(&mut self) -> ParserResult<Program> {
        let mut statements = Vec::new();
        while self.peek_token()?.is_some() {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }

        Ok(Program(statements))
    }

    fn parse_statement(&mut self) -> ParserResult<Statement> {
        let token_type = self
            .peek_token()?
            .cloned()
            .ok_or(ParserError::UnexpectedEOF {
                line: self.last_line,
                column: self.last_column,
            })?;

        match token_type {
            TokenType::Keyword(keyword) => match keyword {
                KeywordType::Let => self.parse_variable_declaration_statement(),
                KeywordType::Fn => self.parse_function_declaration_statement(),
                KeywordType::Return => self.parse_return_statement(),

                KeywordType::True | KeywordType::False => self.parse_expression_statement(),
            },

            TokenType::Identifier(_)
            | TokenType::IntegerLiteral(_)
            | TokenType::LeftParen
            | TokenType::Minus
            | TokenType::Bang => self.parse_expression_statement(),

            unknown_token => Err(ParserError::UnexpectedToken {
                token_type: unknown_token,
                line: self.last_line,
                column: self.last_column,
            }),
        }
    }

    fn parse_expression(&mut self, current_binding: u8) -> ParserResult<Expression> {
        let token_type = self.next_token()?;
        let lhs = match token_type {
            TokenType::Identifier(identifier) => Expression::Identifier(identifier),
            TokenType::StringLiteral(literal) => Expression::StringLiteral(literal),
            TokenType::IntegerLiteral(literal) => Expression::IntegerLiteral(literal),

            TokenType::Keyword(KeywordType::True) => Expression::BooleanLiteral(true),
            TokenType::Keyword(KeywordType::False) => Expression::BooleanLiteral(false),

            TokenType::LeftParen => {
                let expression = self.parse_expression(0)?;
                self.expect_token(&TokenType::RightParen)?;

                expression
            }

            TokenType::Minus | TokenType::Bang => self.parse_unary_expression(token_type)?,

            token_type => {
                return Err(ParserError::UnexpectedToken {
                    token_type,
                    line: self.last_line,
                    column: self.last_column,
                });
            }
        };

        self.parse_binary_expression(current_binding, lhs)
    }

    fn parse_unary_expression(&mut self, token_type: TokenType) -> ParserResult<Expression> {
        let operator =
            UnaryOperator::try_from(&token_type).map_err(|()| ParserError::UnexpectedToken {
                token_type,
                line: self.last_line,
                column: self.last_column,
            })?;

        let expression = self.parse_expression(operator.binding())?;

        Ok(Expression::UnaryOperation(UnaryOperation {
            operator,
            expression: Box::new(expression),
        }))
    }

    fn parse_binary_expression(
        &mut self,
        current_binding: u8,
        mut lhs: Expression,
    ) -> ParserResult<Expression> {
        while let Some(token_type) = self.peek_token()? {
            let Ok(operator) = BinaryOperator::try_from(token_type) else {
                return Ok(lhs);
            };

            let (left_binding, right_binding) = operator.binding();

            if left_binding < current_binding {
                return Ok(lhs);
            }

            self.next_token()?;

            lhs = match operator {
                BinaryOperator::Call => self.parse_call_expression(lhs)?,
                BinaryOperator::Access => self.parse_access_expression(lhs)?,

                _ => Expression::BinaryOperation(BinaryOperation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(self.parse_expression(right_binding)?),
                }),
            };
        }

        Ok(lhs)
    }

    fn parse_call_expression(&mut self, lhs: Expression) -> ParserResult<Expression> {
        let args = self
            .parse_comma_separated(|parser| parser.parse_expression(0), &TokenType::RightParen)?;

        self.expect_token(&TokenType::RightParen)?;

        Ok(Expression::FunctionCall(FunctionCall {
            callee: Box::new(lhs),
            args,
        }))
    }

    fn parse_access_expression(&mut self, lhs: Expression) -> ParserResult<Expression> {
        let field = match self.next_token()? {
            TokenType::Identifier(value) => value,

            token_type => {
                return Err(ParserError::UnexpectedToken {
                    token_type,
                    line: self.last_line,
                    column: self.last_column,
                });
            }
        };

        Ok(Expression::FieldAccess(FieldAccess {
            receiver: Box::new(lhs),
            field,
        }))
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.parse_expression(0)?;

        self.expect_token(&TokenType::Semicolon)?;

        Ok(Statement::Expression(expression))
    }

    fn parse_variable_declaration_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(&TokenType::Keyword(KeywordType::Let))?;

        let name = match self.next_token()? {
            TokenType::Identifier(value) => value,

            token_type => {
                return Err(ParserError::UnexpectedToken {
                    token_type,
                    line: self.last_line,
                    column: self.last_column,
                });
            }
        };

        self.expect_token(&TokenType::Equal)?;

        let value = self.parse_expression(0)?;

        self.expect_token(&TokenType::Semicolon)?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            name,
            value,
        }))
    }

    fn parse_function_declaration_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(&TokenType::Keyword(KeywordType::Fn))?;

        let name = match self.next_token()? {
            TokenType::Identifier(value) => value,

            token_type => {
                return Err(ParserError::UnexpectedToken {
                    token_type,
                    line: self.last_line,
                    column: self.last_column,
                });
            }
        };

        self.expect_token(&TokenType::LeftParen)?;
        let params = self.parse_comma_separated(
            |parser| match parser.next_token()? {
                TokenType::Identifier(name) => Ok(name),

                unexpected_token_type => Err(ParserError::UnexpectedToken {
                    token_type: unexpected_token_type,
                    line: parser.last_line,
                    column: parser.last_column,
                }),
            },
            &TokenType::RightParen,
        )?;
        self.expect_token(&TokenType::RightParen)?;

        let body = self.parse_block_statement()?;

        Ok(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            params,
            body,
        }))
    }

    fn parse_return_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(&TokenType::Keyword(KeywordType::Return))?;

        let expression = self.parse_expression(0)?;

        self.expect_token(&TokenType::Semicolon)?;

        Ok(Statement::Return(expression))
    }

    fn parse_block_statement(&mut self) -> ParserResult<Vec<Statement>> {
        let mut statements = Vec::new();

        self.expect_token(&TokenType::LeftBrace)?;
        while self
            .peek_token()?
            .is_some_and(|token_type| *token_type != TokenType::RightBrace)
        {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.expect_token(&TokenType::RightBrace)?;

        Ok(statements)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<(), ParserError> {
        let source = r"
            fn test(arg1, arg2) {
                let a = 2;
                -2 + 3 * (4 + a);
                true;
                !false;
                return a;
                test(a);
                a.b;
                a.b.c();
            }
        ";

        let mut parser = Parser::new(source);
        let program = parser.parse()?;

        println!("{program:#?}");
        assert_eq!(program.0.len(), 1);

        Ok(())
    }
}
