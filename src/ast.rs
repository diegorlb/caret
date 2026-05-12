use crate::lexer::TokenType;

#[derive(Debug, PartialEq, Eq)]
pub enum PrefixOperator {
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InfixOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl PrefixOperator {
    #[must_use]
    #[inline]
    pub const fn binding(&self) -> u8 {
        10
    }
}

impl InfixOperator {
    #[must_use]
    pub const fn binding(&self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (1, 2),
            Self::Mul | Self::Div => (3, 4),
        }
    }
}

impl TryFrom<&TokenType> for PrefixOperator {
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

impl TryFrom<&TokenType> for InfixOperator {
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

#[derive(Debug, PartialEq, Eq)]
pub struct Program(pub Vec<Statement>);

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Expression(Expression),
    StructDeclaration(StructDeclaration),
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    Return(Option<Expression>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructDeclaration {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableDeclaration {
    pub name: String,
    pub value: Option<Expression>,
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
    StructLiteral(StructLiteral),
    PrefixOperation(PrefixOperation),
    InfixOperation(InfixOperation),
    FunctionCall(FunctionCall),
    FieldAccess(FieldAccess),
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructLiteral {
    pub name: Option<String>,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrefixOperation {
    pub operator: PrefixOperator,
    pub expression: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfixOperation {
    pub operator: InfixOperator,
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
