/// Value axis of the S3 reactivity lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ReactivityClass {
    /// The binding is independent of runtime inputs.
    Static = 0,
    /// The binding depends only on stable component props.
    PropsStable = 1,
    /// The binding reads local reactive state, capture, provide, or inject.
    Reactive = 2,
    /// The binding is effectful or escapes beyond the tracked component.
    Unstable = 3,
}

impl ReactivityClass {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::PropsStable => "props-stable",
            Self::Reactive => "reactive",
            Self::Unstable => "unstable",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"static" => Some(Self::Static),
            b"props-stable" => Some(Self::PropsStable),
            b"reactive" => Some(Self::Reactive),
            b"unstable" => Some(Self::Unstable),
            _ => None,
        }
    }

    /// Least upper bound in the stable-to-unstable order.
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    const fn rank(self) -> u8 {
        self as u8
    }
}
