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
    AndZero,
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

impl CompareOp {
    pub fn as_string(&self) -> &'static str {
        match self {
            CompareOp::Eq => "==",
            CompareOp::NotEq => "!=",
            CompareOp::Lt => "<",
            CompareOp::Gt => ">",
            CompareOp::LtEq => "<=",
            CompareOp::GtEq => ">=",
            CompareOp::And => "&",
            CompareOp::Or => "|",
            CompareOp::AndZero => "!&",
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

impl BitMathOp {
    pub fn as_string(&self) -> &'static str {
        match self {
            BitMathOp::Or => "|",
            BitMathOp::And => "&",
            BitMathOp::Xor => "^",
            BitMathOp::ShiftLeft => "<<",
            BitMathOp::ShiftRight => ">>",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ByteMathOp {
    Add,
    Subtract,
}

impl ByteMathOp {
    pub fn as_string(&self) -> &'static str {
        match self {
            ByteMathOp::Add => "+",
            ByteMathOp::Subtract => "-",
        }
    }
}
