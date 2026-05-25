/// Conditionals for comparisons.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

impl CompareOp {
    pub fn from_value(value: usize) -> CompareOp {
        match value {
            0 => CompareOp::Eq,
            1 => CompareOp::NotEq,
            2 => CompareOp::Gt,
            3 => CompareOp::Lt,
            4 => CompareOp::GtEq,
            5 => CompareOp::LtEq,
            6 => CompareOp::And,
            7 => CompareOp::Or,
            other => {
                panic!("Unknown conditional op {:?}", other);
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BitMathOp {
    Or,
    And,
    Xor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ByteMathOp {
    Add,
    Subtract,
}