//! Compiler errors on the unified diagnostic channel (P4-14b).
//!
//! [`CompilerError::to_diagnostic`] is the one adapter from the template
//! compiler's error type onto `vize_davinci::Diagnostic`, so every renderer —
//! the rich terminal renderer first — reads compiler errors the way it reads
//! everything else: a byte span, a stage, a localized headline, and the
//! code's catalogued help as a part.
//!
//! Localization is by code. An error carrying its code's standard message
//! renders the catalogue's text for the requested locale; an error built with
//! a call-site-specific message (`CompilerError::with_message`) keeps that
//! message, which has no catalogue entry of its own yet. English is the
//! enum's own text either way, so no English output moves.

use super::{CompilerError, ErrorCode};
use vize_davinci::diagnostic::{Advisory, Diagnostic, DiagnosticPart, Exemption, PartKind, Stage};
use vize_s0::i18n::{Locale, translator};
use vize_s0::{CompactString, Span, cstr};

/// The exemption compiler errors report under until the compiler produces
/// witnesses (P4-6), counted in `docs/davinci/plan/witness-exemptions.tsv`.
pub static COMPILER_ERROR: Exemption = Exemption::new("vize_relief", "compiler-error");

impl ErrorCode {
    /// The stage whose checks raise this code.
    #[must_use]
    pub fn stage(self) -> Stage {
        match self {
            Self::ExtendPoint => Stage::Surface,
            _ if self.is_parse_error() => Stage::Surface,
            _ if self.is_transform_error() => Stage::Lowered,
            _ => Stage::Emit,
        }
    }

    /// The catalogued headline for this code in `locale`.
    #[must_use]
    pub fn localized_message(self, locale: Locale) -> CompactString {
        let key = cstr!("{}.message", self.code());
        translator().get(locale, &key).as_ref().into()
    }

    /// The catalogued remedy for this code in `locale`.
    #[must_use]
    pub fn localized_help(self, locale: Locale) -> CompactString {
        let key = cstr!("{}.help", self.code());
        translator().get(locale, &key).as_ref().into()
    }
}

impl CompilerError {
    /// This error's headline in `locale`: the catalogued text when it carries
    /// its code's standard message, its own message otherwise.
    #[must_use]
    pub fn localized_message(&self, locale: Locale) -> CompactString {
        if self.message == self.code.message() {
            self.code.localized_message(locale)
        } else {
            self.message.clone()
        }
    }

    /// This error on the unified channel, worded in `locale`: a warning when
    /// the compiler recovers from it, an error otherwise; the code's help as
    /// a part (except for `ExtendPoint`, whose meaning is its message). An
    /// error without a location is anchored at the start of the file.
    #[must_use]
    pub fn to_diagnostic(&self, locale: Locale) -> Diagnostic {
        let span = self.loc.as_ref().map_or(Span::new(0, 0), |loc| loc.span);
        let message = self.localized_message(locale);
        let stage = self.code.stage();
        let mut diagnostic = if self.is_recoverable() {
            Diagnostic::new(Advisory::Warning, stage, span, message)
        } else {
            Diagnostic::legacy_error(&COMPILER_ERROR, stage, span, message)
        };
        if self.code != ErrorCode::ExtendPoint {
            let help = self.code.localized_help(locale);
            diagnostic = diagnostic.with_part(DiagnosticPart::new(PartKind::Help, span, help));
        }
        diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::{COMPILER_ERROR, CompilerError, ErrorCode};
    use crate::SourceLocation;
    use vize_davinci::diagnostic::{Advisory, Diagnostic, DiagnosticPart, PartKind, Stage};
    use vize_s0::Span;
    use vize_s0::i18n::Locale;

    #[test]
    fn every_code_speaks_its_own_english_and_translates_in_ja_and_zh() {
        for code in ErrorCode::ALL {
            assert_eq!(code.localized_message(Locale::En).as_str(), code.message());
            let english_help = code.localized_help(Locale::En);
            for locale in [Locale::Ja, Locale::Zh] {
                assert_ne!(code.localized_message(locale).as_str(), code.message());
                assert_ne!(code.localized_help(locale), english_help);
            }
        }
    }

    #[test]
    fn a_standard_error_becomes_a_localized_diagnostic_with_help_and_exemption() {
        let loc = SourceLocation {
            span: Span::new(4, 9),
        };
        let error = CompilerError::new(ErrorCode::VIfNoExpression, Some(loc));
        let expected = Diagnostic::legacy_error(
            &COMPILER_ERROR,
            Stage::Lowered,
            Span::new(4, 9),
            "v-if/v-else-if に式がありません。",
        )
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            Span::new(4, 9),
            "`v-if=\"visible\"` のように条件式を指定してください",
        ));
        assert_eq!(error.to_diagnostic(Locale::Ja), expected);
    }

    #[test]
    fn a_recoverable_error_is_a_warning_and_a_custom_message_is_kept() {
        let error = CompilerError::with_message(
            ErrorCode::DuplicateAttribute,
            "Duplicate attribute `class`.",
            None,
        );
        let expected = Diagnostic::new(
            Advisory::Warning,
            Stage::Surface,
            Span::new(0, 0),
            "Duplicate attribute `class`.",
        )
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            Span::new(0, 0),
            "请删除其中一个，或将它们的值合并",
        ));
        assert_eq!(error.to_diagnostic(Locale::Zh), expected);
    }

    #[test]
    fn extension_points_carry_no_generic_help() {
        let error = CompilerError::with_message(ErrorCode::ExtendPoint, "Recovered.", None);
        let expected = Diagnostic::legacy_error(
            &COMPILER_ERROR,
            Stage::Surface,
            Span::new(0, 0),
            "Recovered.",
        );
        assert_eq!(error.to_diagnostic(Locale::En), expected);
    }
}
