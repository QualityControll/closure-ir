use crate::types::TypeInfo;
use crate::value::Value;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expr {
    Argument(usize),
    Constant(Value),
    Field { object: Box<Expr>, name: String },
    Tuple { elements: Vec<Expr> },
    Add { lhs: Box<Expr>, rhs: Box<Expr> },
    Sub { lhs: Box<Expr>, rhs: Box<Expr> },
    Mul { lhs: Box<Expr>, rhs: Box<Expr> },
    Div { lhs: Box<Expr>, rhs: Box<Expr> },
    Rem { lhs: Box<Expr>, rhs: Box<Expr> },
    Eq { lhs: Box<Expr>, rhs: Box<Expr> },
    Ne { lhs: Box<Expr>, rhs: Box<Expr> },
    Lt { lhs: Box<Expr>, rhs: Box<Expr> },
    Le { lhs: Box<Expr>, rhs: Box<Expr> },
    Gt { lhs: Box<Expr>, rhs: Box<Expr> },
    Ge { lhs: Box<Expr>, rhs: Box<Expr> },
    And { lhs: Box<Expr>, rhs: Box<Expr> },
    Or { lhs: Box<Expr>, rhs: Box<Expr> },
    BitAnd { lhs: Box<Expr>, rhs: Box<Expr> },
    BitOr { lhs: Box<Expr>, rhs: Box<Expr> },
    BitXor { lhs: Box<Expr>, rhs: Box<Expr> },
    Shl { lhs: Box<Expr>, rhs: Box<Expr> },
    Shr { lhs: Box<Expr>, rhs: Box<Expr> },
    Not { operand: Box<Expr> },
    Neg { operand: Box<Expr> },
    SqrtF32 { operand: Box<Expr> },
    SqrtF64 { operand: Box<Expr> },
    PowF32 { lhs: Box<Expr>, rhs: Box<Expr> },
    PowF64 { lhs: Box<Expr>, rhs: Box<Expr> },
    IfElse { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Statement {
    Let { local: usize, type_info: TypeInfo, value: Expr, mutable: bool },
    Assign { local: usize, value: Expr },
    While { condition: Expr, body: Block },
    For { local: usize, type_info: TypeInfo, start: Expr, end: Expr, inclusive: bool, body: Block },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub result: Option<Expr>,
}

impl Block {
    pub fn expression(result: Expr) -> Self {
        Self { statements: Vec::new(), result: Some(result) }
    }

    pub fn statements(statements: Vec<Statement>) -> Self {
        Self { statements, result: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Closure {
    pub arguments: Vec<TypeInfo>,
    pub return_type: TypeInfo,
    pub body: Block,
}
