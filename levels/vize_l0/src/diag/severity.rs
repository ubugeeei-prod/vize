//! [`Severity`] — how much a diagnostic claims — and [`Advisory`], the subset
//! a diagnostic may claim without a proof.

/// How much a diagnostic claims.
///
/// The distinction is load-bearing rather than cosmetic: `assurance.md`'s
/// no-error-on-maybe rule is that an unproven finding "never produces an
/// error; it produces silence, or an explicitly-labeled hint/suggestion
/// severity that _says_ it is not a proof". [`Severity::Error`] is therefore
/// reachable only through the witness-taking constructors
/// ([`Diagnostic::proven`](super::Diagnostic::proven),
/// [`Diagnostic::legacy_error`](super::Diagnostic::legacy_error)); every
/// other constructor takes an [`Advisory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A proven violation, carrying its witness (or a counted exemption).
    Error,
    /// A probable problem, stated as such.
    Warning,
    /// Information that is not a problem.
    Info,
    /// A suggestion the author may ignore.
    Hint,
}

impl Severity {
    /// Every severity, from most to least claimed.
    pub const ALL: [Severity; 4] = [
        Severity::Error,
        Severity::Warning,
        Severity::Info,
        Severity::Hint,
    ];

    /// The stable lowercase spelling (`error`, `warning`, `info`, `hint`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }

    /// Parse the stable spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"error" => Some(Severity::Error),
            b"warning" => Some(Severity::Warning),
            b"info" => Some(Severity::Info),
            b"hint" => Some(Severity::Hint),
            _ => None,
        }
    }
}

/// A severity that claims no proof: everything but [`Severity::Error`].
///
/// [`Diagnostic::new`](super::Diagnostic::new) takes this type, not
/// [`Severity`], which is what makes an unwitnessed error a type error rather
/// than a review comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Advisory {
    /// A probable problem, stated as such.
    Warning,
    /// Information that is not a problem.
    Info,
    /// A suggestion the author may ignore.
    Hint,
}

impl Advisory {
    /// Every advisory severity, from most to least claimed.
    pub const ALL: [Advisory; 3] = [Advisory::Warning, Advisory::Info, Advisory::Hint];

    /// The [`Severity`] this advisory reports as.
    #[must_use]
    pub const fn severity(self) -> Severity {
        match self {
            Advisory::Warning => Severity::Warning,
            Advisory::Info => Severity::Info,
            Advisory::Hint => Severity::Hint,
        }
    }

    /// The advisory reporting as `severity`, or `None` for
    /// [`Severity::Error`] — which no advisory can claim.
    #[must_use]
    pub const fn from_severity(severity: Severity) -> Option<Self> {
        match severity {
            Severity::Error => None,
            Severity::Warning => Some(Advisory::Warning),
            Severity::Info => Some(Advisory::Info),
            Severity::Hint => Some(Advisory::Hint),
        }
    }
}

impl From<Advisory> for Severity {
    fn from(advisory: Advisory) -> Self {
        advisory.severity()
    }
}
