//! How versions must move for a classified change.
//!
//! The package version is semver. Before 1.0 the minor number is the
//! breaking axis (`0.1.x` → `0.2.0`) and any increase covers an additive
//! change; from 1.0 on, breaking needs a new major and additive a new minor.
//! A breaking change also raises the handshake protocol version by exactly
//! one, so a host refuses an incompatible guest before the first call.

use core::fmt;

use super::ContractSurface;
use super::compare::ContractSurfaceReport;
use crate::CompatibilityChangeKind;

/// A `MAJOR.MINOR.PATCH` version (no pre-release or build metadata).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContractVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl ContractVersion {
    /// Parse `MAJOR.MINOR.PATCH`; `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.');
        let mut next = || -> Option<u64> {
            let part = parts.next()?;
            let canonical = part == "0" || !part.starts_with('0');
            (canonical && part.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| part.parse().ok())
                .flatten()
        };
        let version = Self {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        parts.next().is_none().then_some(version)
    }

    /// The smallest version a change of `kind` may move to from `self`.
    #[must_use]
    pub fn least_after(self, kind: CompatibilityChangeKind) -> Self {
        match (kind, self.major) {
            (CompatibilityChangeKind::Breaking, 0) | (CompatibilityChangeKind::Additive, 1..) => {
                Self {
                    major: self.major,
                    minor: self.minor + 1,
                    patch: 0,
                }
            }
            (CompatibilityChangeKind::Breaking, _) => Self {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
            (CompatibilityChangeKind::Additive, 0) => Self {
                patch: self.patch + 1,
                ..self
            },
        }
    }
}

impl fmt::Display for ContractVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A way the versions fail to follow the classified change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionPolicyViolation {
    /// A version is not `MAJOR.MINOR.PATCH`.
    Unparsable {
        which: &'static str,
        version: vize_s0::String,
    },
    /// The surface changed but the version did not.
    Unchanged { version: ContractVersion },
    /// The version went backwards.
    Decreased {
        previous: ContractVersion,
        next: ContractVersion,
    },
    /// The version moved, but not far enough for the change.
    TooSmall {
        kind: CompatibilityChangeKind,
        required: ContractVersion,
        next: ContractVersion,
    },
    /// A breaking change did not raise the protocol version by exactly one.
    Protocol { previous: u32, next: u32 },
}

impl fmt::Display for VersionPolicyViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unparsable { which, version } => {
                write!(
                    f,
                    "the {which} version {version:?} is not MAJOR.MINOR.PATCH"
                )
            }
            Self::Unchanged { version } => {
                write!(f, "the surface changed but the version stayed {version}")
            }
            Self::Decreased { previous, next } => {
                write!(f, "the version must not decrease: {previous} -> {next}")
            }
            Self::TooSmall {
                kind,
                required,
                next,
            } => {
                let kind = match kind {
                    CompatibilityChangeKind::Additive => "an additive",
                    CompatibilityChangeKind::Breaking => "a breaking",
                };
                write!(
                    f,
                    "{kind} change needs version {required} or later, found {next}"
                )
            }
            Self::Protocol { previous, next } => write!(
                f,
                "a breaking change must raise the protocol version by exactly one: {previous} -> {next}"
            ),
        }
    }
}

/// Check that `next`'s versions follow `report`, the classified change
/// from `previous`. Empty when the policy holds.
#[must_use]
pub fn check_version_policy(
    previous: &ContractSurface,
    next: &ContractSurface,
    report: &ContractSurfaceReport,
) -> Vec<VersionPolicyViolation> {
    let parse = |which, version: &vize_s0::String| {
        ContractVersion::parse(version).ok_or_else(|| VersionPolicyViolation::Unparsable {
            which,
            version: version.clone(),
        })
    };
    let (old, new) = match (
        parse("previous", &previous.version),
        parse("next", &next.version),
    ) {
        (Ok(old), Ok(new)) => (old, new),
        (old, new) => return old.err().into_iter().chain(new.err()).collect(),
    };
    if new < old {
        return vec![VersionPolicyViolation::Decreased {
            previous: old,
            next: new,
        }];
    }
    let Some(kind) = report.changes.iter().map(|change| change.kind).max() else {
        return Vec::new();
    };
    if new == old {
        return vec![VersionPolicyViolation::Unchanged { version: old }];
    }
    let mut violations = Vec::new();
    let required = old.least_after(kind);
    if new < required {
        violations.push(VersionPolicyViolation::TooSmall {
            kind,
            required,
            next: new,
        });
    }
    let protocol_bumped = next.protocol_version == previous.protocol_version.saturating_add(1);
    if kind == CompatibilityChangeKind::Breaking && !protocol_bumped {
        violations.push(VersionPolicyViolation::Protocol {
            previous: previous.protocol_version,
            next: next.protocol_version,
        });
    }
    violations
}
