use crate::types::TypeInfo;
use crate::value::Value;
use crate::ffi::ExternalFunction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Intrinsic { Sqrt, Abs, Min, Max, Floor, Ceil, Round, Sin, Cos, Tan, Exp, Log, Pow }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expr {
    Argument(usize), Capture(usize), Constant(Value),
    Cast { expr: Box<Expr>, source_type: TypeInfo, target_type: TypeInfo }, Index { sequence: Box<Expr>, index: Box<Expr> }, Len { sequence: Box<Expr> }, Field { object: Box<Expr>, name: String }, Struct { type_info: TypeInfo, fields: Vec<(String, Expr)> }, Tuple { elements: Vec<Expr> }, Intrinsic { intrinsic: Intrinsic, arguments: Vec<Expr> },
    ExternalCall { function: String, arguments: Vec<Expr>, return_type: TypeInfo },
    Add { lhs: Box<Expr>, rhs: Box<Expr> }, Sub { lhs: Box<Expr>, rhs: Box<Expr> }, Mul { lhs: Box<Expr>, rhs: Box<Expr> }, Div { lhs: Box<Expr>, rhs: Box<Expr> }, Rem { lhs: Box<Expr>, rhs: Box<Expr> },
    Eq { lhs: Box<Expr>, rhs: Box<Expr> }, Ne { lhs: Box<Expr>, rhs: Box<Expr> }, Lt { lhs: Box<Expr>, rhs: Box<Expr> }, Le { lhs: Box<Expr>, rhs: Box<Expr> }, Gt { lhs: Box<Expr>, rhs: Box<Expr> }, Ge { lhs: Box<Expr>, rhs: Box<Expr> },
    And { lhs: Box<Expr>, rhs: Box<Expr> }, Or { lhs: Box<Expr>, rhs: Box<Expr> }, BitAnd { lhs: Box<Expr>, rhs: Box<Expr> }, BitOr { lhs: Box<Expr>, rhs: Box<Expr> }, BitXor { lhs: Box<Expr>, rhs: Box<Expr> }, Shl { lhs: Box<Expr>, rhs: Box<Expr> }, Shr { lhs: Box<Expr>, rhs: Box<Expr> }, Not { operand: Box<Expr> }, Neg { operand: Box<Expr> }, IfElse { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Statement { Let { local: usize, type_info: TypeInfo, value: Expr, mutable: bool }, Assign { local: usize, value: Expr }, AssignIndex { sequence: Expr, index: Expr, value: Expr }, While { condition: Expr, body: Block }, For { local: usize, type_info: TypeInfo, start: Expr, end: Expr, inclusive: bool, body: Block } }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block { pub statements: Vec<Statement>, pub result: Option<Expr> }
impl Block { pub fn expression(result: Expr) -> Self { Self { statements: Vec::new(), result: Some(result) } } pub fn statements(statements: Vec<Statement>) -> Self { Self { statements, result: None } } }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Closure { pub captures: Vec<TypeInfo>, pub arguments: Vec<TypeInfo>, pub return_type: TypeInfo, pub body: Block, pub external_functions: Vec<ExternalFunction> }
