pub mod cpu;
pub mod gte;

use std::marker::ConstParamTy;

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum ShiftType {
    Logical,
    Arithmetic,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum ShiftDirection {
    Left,
    Right,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum BranchType {
    Unconditional,
    Equal,
    NotEqual,
    LessEqualZero,
    LessThanZero,
    GreaterEqualZero,
    GreaterThanZero,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum BranchAddressing {
    AbsoluteImmediate,
    RelativeOffset,
    AbsoluteRegister,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum AluOperation {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Nor,
    Multiply,
    Divide,
    SetLessThan,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryAccessType {
    Load,
    Store,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryTransferSize {
    Byte,
    HalfWord,
    Word,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryAccessPortion {
    Full,
    Left,
    Right,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MultiplyMoveDirection {
    ToRegister,
    FromRegister,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MultiplyMoveRegister {
    Hi,
    Lo,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum CopOperation {
    MoveTo,
    MoveFrom,
    MoveControlTo,
    MoveControlFrom,
    ReturnFromException,
    LoadWordTo,
    StoreWordFrom,
}