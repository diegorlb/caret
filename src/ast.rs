use crate::lexer::TokenType;

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,

    Call,
    Access,
}

impl UnaryOperator {
    #[must_use]
    #[inline]
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

            Self::Call | Self::Access => (7, 8),
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

            TokenType::Dot => Self::Access,
            TokenType::LeftParen => Self::Call,

            _ => return Err(()),
        };

        Ok(operator)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Program(pub Vec<Statement>);

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    Return(Expression),
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableDeclaration {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    FunctionCall(FunctionCall),
    FieldAccess(FieldAccess),
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionCall {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldAccess {
    pub receiver: Box<Expression>,
    pub field: String,
}
