use crate::lexer::TokenType;

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
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    BinaryOperation(BinaryOperation),
}

#[derive(Debug)]
pub struct BinaryOperation {
    pub operator: TokenType,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}
