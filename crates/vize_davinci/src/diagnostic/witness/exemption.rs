//! [`Exemption`] — one legacy producer's error kind, exempt from the witness
//! law by inventory.
//!
//! The inventory is `davinci-road/plan/witness-exemptions.tsv` (`producer`,
//! `code`, `exempt`), and `tests/tooling/davinci-witness-exemptions.test.ts`
//! derives it mechanically from the source: every `Exemption::new` is a
//! declaration, and `exempt` counts the construction sites that report under
//! it. The checker fails when the committed file drifts from the source, and
//! when any count rises against the base revision — so the inventory can only
//! shrink, and the phase-4 exit gate is the moment it reaches zero rows.
//!
//! Declare an exemption as a named `static` in the producer's crate, so the
//! checker can count its uses:
//!
//! ```
//! use vize_davinci::diagnostic::{Advisory, Diagnostic, Exemption, Severity, Stage};
//! use vize_s0::Span;
//!
//! static UNTERMINATED: Exemption = Exemption::new("doc_producer", "unterminated");
//!
//! let error = Diagnostic::legacy_error(&UNTERMINATED, Stage::Surface, Span::new(0, 1), "boom");
//! assert_eq!(error.severity(), Severity::Error);
//! assert_eq!(error.exemption(), Some(&UNTERMINATED));
//!
//! let hint = Diagnostic::new(Advisory::Hint, Stage::Surface, Span::new(0, 1), "tip");
//! assert_eq!(hint.exemption(), None);
//! ```

/// A declared exemption from the witness law: the producer (a crate name)
/// and the code of the error kind it reports without a witness.
///
/// Deliberately neither `Clone` nor `Copy`: an exemption is used by
/// `&'static` reference to its declaration, never duplicated.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Exemption {
    producer: &'static str,
    code: &'static str,
}

impl Exemption {
    /// The exemption for `producer`'s error kind `code`.
    ///
    /// # Panics
    ///
    /// Panics unless both names are non-empty and spelled in
    /// `[a-z0-9_./-]` — the characters an inventory row (tab-separated) and
    /// the checker's scan can carry verbatim. Exemptions are declared as
    /// `static` items, where that panic is a compile error.
    #[must_use]
    pub const fn new(producer: &'static str, code: &'static str) -> Self {
        assert!(
            is_inventory_name(producer),
            "an exemption's producer must be a non-empty [a-z0-9_./-] name"
        );
        assert!(
            is_inventory_name(code),
            "an exemption's code must be a non-empty [a-z0-9_./-] name"
        );
        Self { producer, code }
    }

    /// The producing crate.
    #[must_use]
    pub const fn producer(&self) -> &'static str {
        self.producer
    }

    /// The exempt error kind.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
}

const fn is_inventory_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let allowed = byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'_' | b'.' | b'/' | b'-');
        if !allowed {
            return false;
        }
        index += 1;
    }
    true
}
