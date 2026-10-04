use crate::decision::StaticLevel;

#[derive(Clone, Copy)]
pub(super) struct Levels {
    pub(super) neutral: StaticLevel,
    pub(super) output: StaticLevel,
}

impl Levels {
    pub(super) const STATIC: Self = Self {
        neutral: StaticLevel::Static,
        output: StaticLevel::Static,
    };
    pub(super) const DYNAMIC: Self = Self {
        neutral: StaticLevel::Dynamic,
        output: StaticLevel::Dynamic,
    };
    pub(super) const DYNAMIC_TEXT: Self = Self {
        neutral: StaticLevel::DynamicText,
        output: StaticLevel::DynamicText,
    };

    pub(super) fn join(self, other: Self) -> Self {
        Self {
            neutral: join(self.neutral, other.neutral),
            output: join(self.output, other.output),
        }
    }

    pub(super) fn nested(self) -> Self {
        Self {
            neutral: nested(self.neutral),
            output: nested(self.output),
        }
    }
}

fn join(left: StaticLevel, right: StaticLevel) -> StaticLevel {
    match (left, right) {
        (StaticLevel::Dynamic, _) | (_, StaticLevel::Dynamic) => StaticLevel::Dynamic,
        (StaticLevel::DynamicText, _) | (_, StaticLevel::DynamicText) => StaticLevel::DynamicText,
        (StaticLevel::Static, StaticLevel::Static) => StaticLevel::Static,
    }
}

fn nested(level: StaticLevel) -> StaticLevel {
    match level {
        StaticLevel::DynamicText => StaticLevel::Dynamic,
        StaticLevel::Static | StaticLevel::Dynamic => level,
    }
}
