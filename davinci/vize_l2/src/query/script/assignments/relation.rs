//! Assignability of primitive literal types to their respective broad types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Boolean,
    Number,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Assignable,
    Mismatch,
}

impl Primitive {
    pub const fn accepts(self, value: Self) -> Verdict {
        match (self, value) {
            (Self::Boolean, Self::Boolean)
            | (Self::Number, Self::Number)
            | (Self::String, Self::String) => Verdict::Assignable,
            _ => Verdict::Mismatch,
        }
    }

    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}
