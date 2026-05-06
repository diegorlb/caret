use crate::lexer::TokenType;

#[derive(Debug)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl UnaryOperator {
    #[must_use]
    pub const fn binding(&self) -> u8 {
        10
    }
}

impl BinaryOperator {
    #[must_use]
    pub const fn binding(&self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (1, 2),
            Self::Mul | Self::Div => (3, 4),
        }
    }
}

impl TryFrom<&TokenType> for UnaryOperator {
    type Error = ();

    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        let operator = match *value {
            TokenType::Minus => Self::Neg,
            TokenType::Bang => Self::Not,

            _ => return Err(()),
        };

        Ok(operator)
    }
}

impl TryFrom<&TokenType> for BinaryOperator {
    type Error = ();

    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        let operator = match *value {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Asterisk => Self::Mul,
            TokenType::Slash => Self::Div,

            _ => return Err(()),
        };

        Ok(operator)
    }
}

#[derive(Debug)]
pub struct Program(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    UnaryExpression(UnaryExpression),
    BinaryOperation(BinaryOperation),
}

#[derive(Debug)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

#[derive(Debug)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}
