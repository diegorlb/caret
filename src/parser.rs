use std::iter::Peekable;

use thiserror::Error;

use crate::{
    ast::{
        Expression, FieldAccess, FunctionCall, FunctionDeclaration, InfixOperation, InfixOperator,
        PrefixOperation, PrefixOperator, Program, Statement, VariableDeclaration,
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

    const fn unexpected_token_error(&self, token_type: TokenType) -> ParserError {
        ParserError::UnexpectedToken {
            token_type,
            line: self.last_line,
            column: self.last_column,
        }
    }

    fn peek_token(&mut self) -> ParserResult<Option<&TokenType>> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(Some(&token.token_type)),
            Some(Err(error)) => Err(ParserError::LexerError(*error)),
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

    #[allow(clippy::needless_pass_by_value)]
    fn expect_token(&mut self, expected: TokenType) -> ParserResult<()> {
        self.next_token().and_then(|token_type| {
            if token_type == expected {
                Ok(())
            } else {
                Err(self.unexpected_token_error(token_type))
            }
        })
    }

    fn expect_identifier(&mut self) -> ParserResult<String> {
        match self.next_token()? {
            TokenType::Identifier(identifier) => Ok(identifier),
            token_type => Err(self.unexpected_token_error(token_type)),
        }
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

                _ => self.parse_expression_statement(),
            },

            TokenType::Identifier(_)
            | TokenType::IntegerLiteral(_)
            | TokenType::StringLiteral(_)
            | TokenType::LeftParen
            | TokenType::Minus
            | TokenType::Bang => self.parse_expression_statement(),

            token_type => Err(self.unexpected_token_error(token_type)),
        }
    }

    fn parse_expression(&mut self, current_binding: u8) -> ParserResult<Expression> {
        let token_type = self.next_token()?;

        let mut lhs = self.parse_primary_expression(token_type)?;
        lhs = self.parse_postfix_expression(lhs)?;
        lhs = self.parse_infix_expression(current_binding, lhs)?;

        Ok(lhs)
    }

    fn parse_primary_expression(&mut self, token_type: TokenType) -> ParserResult<Expression> {
        let expression = match token_type {
            TokenType::Identifier(identifier) => Expression::Identifier(identifier),
            TokenType::StringLiteral(literal) => Expression::StringLiteral(literal),
            TokenType::IntegerLiteral(literal) => Expression::IntegerLiteral(literal),

            TokenType::Keyword(KeywordType::True) => Expression::BooleanLiteral(true),
            TokenType::Keyword(KeywordType::False) => Expression::BooleanLiteral(false),

            TokenType::LeftParen => {
                let expression = self.parse_expression(0)?;
                self.expect_token(TokenType::RightParen)?;

                expression
            }

            TokenType::Minus | TokenType::Bang => self.parse_prefix_expression(token_type)?,

            token_type => return Err(self.unexpected_token_error(token_type)),
        };

        Ok(expression)
    }

    fn parse_prefix_expression(&mut self, token_type: TokenType) -> ParserResult<Expression> {
        let operator = PrefixOperator::try_from(&token_type)
            .map_err(|()| self.unexpected_token_error(token_type))?;

        let expression = self.parse_expression(operator.binding())?;

        Ok(Expression::PrefixOperation(PrefixOperation {
            operator,
            expression: Box::new(expression),
        }))
    }

    fn parse_infix_expression(
        &mut self,
        current_binding: u8,
        mut lhs: Expression,
    ) -> ParserResult<Expression> {
        while let Some(token_type) = self.peek_token()? {
            let Ok(operator) = InfixOperator::try_from(token_type) else {
                return Ok(lhs);
            };

            let (left_binding, right_binding) = operator.binding();

            if left_binding < current_binding {
                return Ok(lhs);
            }

            self.next_token()?;

            let rhs = self.parse_expression(right_binding)?;
            lhs = Expression::InfixOperation(InfixOperation {
                operator,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            });
        }

        Ok(lhs)
    }

    fn parse_postfix_expression(&mut self, mut lhs: Expression) -> ParserResult<Expression> {
        while let Some(token_type) = self.peek_token()? {
            lhs = match token_type {
                TokenType::LeftParen => {
                    self.next_token()?;

                    let args = self.parse_comma_separated(
                        |parser| parser.parse_expression(0),
                        &TokenType::RightParen,
                    )?;

                    self.expect_token(TokenType::RightParen)?;

                    Expression::FunctionCall(FunctionCall {
                        callee: Box::new(lhs),
                        args,
                    })
                }

                TokenType::Dot => {
                    self.next_token()?;
                    let field = self.expect_identifier()?;

                    Expression::FieldAccess(FieldAccess {
                        receiver: Box::new(lhs),
                        field,
                    })
                }

                _ => break,
            }
        }

        Ok(lhs)
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.parse_expression(0)?;

        self.expect_token(TokenType::Semicolon)?;

        Ok(Statement::Expression(expression))
    }

    fn parse_variable_declaration_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(TokenType::Keyword(KeywordType::Let))?;

        let name = self.expect_identifier()?;

        let value = match self.next_token_if(|token_type| *token_type == TokenType::Equal)? {
            Some(_) => Some(self.parse_expression(0)?),
            None => None,
        };

        self.expect_token(TokenType::Semicolon)?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            name,
            value,
        }))
    }

    fn parse_function_declaration_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(TokenType::Keyword(KeywordType::Fn))?;

        let name = self.expect_identifier()?;

        self.expect_token(TokenType::LeftParen)?;
        let params =
            self.parse_comma_separated(Parser::expect_identifier, &TokenType::RightParen)?;
        self.expect_token(TokenType::RightParen)?;

        let body = self.parse_block_statement()?;

        Ok(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            params,
            body,
        }))
    }

    fn parse_return_statement(&mut self) -> ParserResult<Statement> {
        self.expect_token(TokenType::Keyword(KeywordType::Return))?;

        let expression = match self.peek_token()? {
            Some(TokenType::Semicolon) => None,
            _ => Some(self.parse_expression(0)?),
        };

        self.expect_token(TokenType::Semicolon)?;

        Ok(Statement::Return(expression))
    }

    fn parse_block_statement(&mut self) -> ParserResult<Vec<Statement>> {
        let mut statements = Vec::new();

        self.expect_token(TokenType::LeftBrace)?;
        while self
            .peek_token()?
            .is_some_and(|token_type| *token_type != TokenType::RightBrace)
        {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.expect_token(TokenType::RightBrace)?;

        Ok(statements)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_parser {
        ($name:ident, [$($source:literal => $expected:expr),+]) => {
            #[test]
            fn $name() -> ParserResult<()> {
                $({
                    let mut parser = Parser::new($source);
                    let program = parser.parse()?;

                    assert_eq!(program, $expected);
                })+

                Ok(())
            }
        };
    }

    test_parser!(variable_declaration_statement, [
        "let test;" => Program(vec![
            Statement::VariableDeclaration(VariableDeclaration {
                name: String::from("test"),
                value: None,
            })
        ]),

        "let test = 2;" => Program(vec![
            Statement::VariableDeclaration(VariableDeclaration {
                name: String::from("test"),
                value: Some(Expression::IntegerLiteral(2)),
            })
        ])
    ]);

    test_parser!(function_declaration_statement, [
        "fn test() {}" => Program(vec![
            Statement::FunctionDeclaration(FunctionDeclaration {
                name: String::from("test"),
                params: vec![],
                body: vec![],
            })
        ]),


        "fn test(arg1, arg2, arg3) {}" => Program(vec![
            Statement::FunctionDeclaration(FunctionDeclaration {
                name: String::from("test"),
                params: vec![String::from("arg1"), String::from("arg2"), String::from("arg3")],
                body: vec![],
            })
        ]),

        "fn test() { let other; }" => Program(vec![
            Statement::FunctionDeclaration(FunctionDeclaration {
                name: String::from("test"),
                params: vec![],
                body: vec![
                    Statement::VariableDeclaration(VariableDeclaration {
                        name: String::from("other"),
                        value: None,
                    }),
                ],
            })
        ])
    ]);

    test_parser!(return_statement, [
        "return;" => Program(vec![ Statement::Return(None) ]),
        "return true;" => Program(vec![
            Statement::Return(Some(Expression::BooleanLiteral(true)))
        ])
    ]);

    test_parser!(expressions, [
        r#"
            test;
            123;
            "test";
            true;
            false;
        "# => Program(vec![
            Statement::Expression(Expression::Identifier(String::from("test"))),
            Statement::Expression(Expression::IntegerLiteral(123)),
            Statement::Expression(Expression::StringLiteral(String::from("test"))),
            Statement::Expression(Expression::BooleanLiteral(true)),
            Statement::Expression(Expression::BooleanLiteral(false)),
        ]),


        r"
            (2);
            2 + 3;
            2 - 3;
            2 / 3;
            2 + 3 * 4;
            !true;
            -2;
            test();
            test(1, true);
            object.field;
        " => Program(vec![
            Statement::Expression(Expression::IntegerLiteral(2)),
            Statement::Expression(Expression::InfixOperation(InfixOperation {
                operator: InfixOperator::Add,
                lhs: Box::new(Expression::IntegerLiteral(2)),
                rhs: Box::new(Expression::IntegerLiteral(3)),
            })),
            Statement::Expression(Expression::InfixOperation(InfixOperation {
                operator: InfixOperator::Sub,
                lhs: Box::new(Expression::IntegerLiteral(2)),
                rhs: Box::new(Expression::IntegerLiteral(3)),
            })),
            Statement::Expression(Expression::InfixOperation(InfixOperation {
                operator: InfixOperator::Div,
                lhs: Box::new(Expression::IntegerLiteral(2)),
                rhs: Box::new(Expression::IntegerLiteral(3)),
            })),
            Statement::Expression(Expression::InfixOperation(InfixOperation {
                operator: InfixOperator::Add,
                lhs: Box::new(Expression::IntegerLiteral(2)),
                rhs: Box::new(Expression::InfixOperation(InfixOperation {
                    operator: InfixOperator::Mul,
                    lhs: Box::new(Expression::IntegerLiteral(3)),
                    rhs: Box::new(Expression::IntegerLiteral(4)),
                }))
            })),
            Statement::Expression(Expression::PrefixOperation(PrefixOperation {
                operator: PrefixOperator::Not,
                expression: Box::new(Expression::BooleanLiteral(true)),
            })),
            Statement::Expression(Expression::PrefixOperation(PrefixOperation {
                operator: PrefixOperator::Neg,
                expression: Box::new(Expression::IntegerLiteral(2)),
            })),
            Statement::Expression(Expression::FunctionCall(FunctionCall {
                callee: Box::new(Expression::Identifier(String::from("test"))),
                args: vec![],
            })),
            Statement::Expression(Expression::FunctionCall(FunctionCall {
                callee: Box::new(Expression::Identifier(String::from("test"))),
                args: vec![
                    Expression::IntegerLiteral(1),
                    Expression::BooleanLiteral(true)
                ],
            })),
            Statement::Expression(Expression::FieldAccess(FieldAccess {
                receiver: Box::new(Expression::Identifier(String::from("object"))),
                field: String::from("field")
            }))
        ])
    ]);
}
