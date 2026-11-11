//! Lossless surface diagnostics and their parser-compatible presentation.

use vize_l0::{CompactString, ErrorCode, Span, ToCompactString, appends};

/// A parser diagnostic retained by the lossless L1 surface.
///
/// Tokenizer diagnostics and the structural recoveries performed while the
/// event stream becomes a tree share this type. Consumers therefore do not
/// need to run the legacy parser just to recover the original diagnostic
/// message, severity or span.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceError {
    pub code: ErrorCode,
    /// Compatibility start coordinate retained for existing native callers.
    pub offset: u32,
    end: u32,
    pub(super) encounter: u32,
    detail: SurfaceErrorDetail,
}

#[derive(Debug, Clone, Copy)]
enum SurfaceErrorDetail {
    Default,
    DuplicateAttribute,
    MissingDirectiveName,
    AdoptionAgency,
    IgnoredRepeatedForm,
    FosterParenting,
    NestingTooDeep,
    /// A raw input-stream character. Parsing keeps the character and the tree.
    InputStream,
}

impl SurfaceError {
    pub(crate) const fn tokenizer(code: ErrorCode, offset: u32) -> Self {
        Self {
            code,
            offset,
            end: offset.saturating_add(1),
            encounter: offset,
            detail: SurfaceErrorDetail::Default,
        }
    }

    /// An input-stream character error. The character stays in the tree, so
    /// later rules still see the template.
    pub(crate) const fn input_stream(code: ErrorCode, offset: u32) -> Self {
        Self {
            code,
            offset,
            end: offset.saturating_add(1),
            encounter: offset,
            detail: SurfaceErrorDetail::InputStream,
        }
    }

    /// Move every coordinate into its containing source frame.
    #[must_use]
    pub const fn rebased(mut self, by: u32) -> Self {
        self.offset = self.offset.saturating_add(by);
        self.end = self.end.saturating_add(by);
        self.encounter = self.encounter.saturating_add(by);
        self
    }

    pub(crate) const fn at(code: ErrorCode, span: Span) -> Self {
        Self {
            code,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::Default,
        }
    }

    pub(crate) const fn at_encounter(code: ErrorCode, span: Span, encounter: u32) -> Self {
        Self {
            code,
            offset: span.start,
            end: span.end,
            encounter,
            detail: SurfaceErrorDetail::Default,
        }
    }

    pub(crate) const fn duplicate_attribute(span: Span) -> Self {
        Self {
            code: ErrorCode::DuplicateAttribute,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::DuplicateAttribute,
        }
    }

    pub(crate) const fn missing_directive_name(span: Span) -> Self {
        Self {
            code: ErrorCode::MissingDirectiveName,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::MissingDirectiveName,
        }
    }

    pub(crate) const fn adoption_agency(span: Span) -> Self {
        Self {
            code: ErrorCode::ExtendPoint,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::AdoptionAgency,
        }
    }

    pub(crate) const fn ignored_repeated_form(span: Span) -> Self {
        Self {
            code: ErrorCode::ExtendPoint,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::IgnoredRepeatedForm,
        }
    }

    pub(crate) const fn foster_parenting(span: Span) -> Self {
        Self {
            code: ErrorCode::ExtendPoint,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::FosterParenting,
        }
    }

    pub(crate) const fn nesting_too_deep(span: Span) -> Self {
        Self {
            code: ErrorCode::ExtendPoint,
            offset: span.start,
            end: span.end,
            encounter: span.start,
            detail: SurfaceErrorDetail::NestingTooDeep,
        }
    }

    /// The exact half-open byte span reported by the preserved parser surface.
    #[must_use]
    pub fn span(self, source_len: u32) -> Span {
        if source_len == 0 {
            return Span::new(0, 0);
        }
        let start = if self.offset < source_len {
            self.offset
        } else {
            source_len - 1
        };
        let end = if self.end > start {
            self.end
        } else {
            start.saturating_add(1)
        };
        Span::new(start, if end < source_len { end } else { source_len })
    }

    /// User-facing text matching the parser/template diagnostic contract.
    #[must_use]
    pub fn message(self, source: &str) -> CompactString {
        if self.code == ErrorCode::MissingAttributeValue {
            let name = missing_attribute_name(source, self.offset).unwrap_or("attribute");
            let mut message = vize_l0::String::with_capacity(name.len() + 70);
            appends!(
                message,
                "Attribute `",
                name,
                "` is missing a value after `=`; continuing without the value."
            );
            return message.to_compact_string();
        }
        match self.detail {
            SurfaceErrorDetail::DuplicateAttribute => {
                let span = self.span(source.len() as u32);
                let name = source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or("attribute");
                let mut message = vize_l0::String::with_capacity(name.len() + 79);
                appends!(
                    message,
                    "Duplicate attribute `",
                    name,
                    "`. Keeping the repeated attribute so parsing can continue."
                );
                message.to_compact_string()
            }
            SurfaceErrorDetail::MissingDirectiveName => {
                let span = self.span(source.len() as u32);
                let authored = source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or("directive");
                let name = authored
                    .strip_prefix("v-")
                    .map(|tail| {
                        let end = tail
                            .find(['.', ':'])
                            .map_or(authored.len(), |index| index + 2);
                        authored.get(..end).unwrap_or(authored)
                    })
                    .unwrap_or(authored);
                let mut message = vize_l0::String::with_capacity(name.len() + 80);
                appends!(
                    message,
                    "Directive `",
                    name,
                    "` is missing a name. Ignoring it so the rest of the tag can be parsed."
                );
                message.to_compact_string()
            }
            SurfaceErrorDetail::AdoptionAgency =>
                "Misnested formatting end tag was repaired using the adoption agency recovery path."
                    .to_compact_string(),
            SurfaceErrorDetail::IgnoredRepeatedForm =>
                "HTML tree construction ignored this start tag because an equivalent element is already open."
                    .to_compact_string(),
            SurfaceErrorDetail::FosterParenting =>
                "Foster parenting moved this element before the nearest open table."
                    .to_compact_string(),
            SurfaceErrorDetail::NestingTooDeep => {
                "Element nesting is too deep.".to_compact_string()
            }
            SurfaceErrorDetail::InputStream | SurfaceErrorDetail::Default => {
                recovery_message(self.code).unwrap_or_else(|| self.code.message().to_compact_string())
            }
        }
    }

    /// A recoverable parser/template diagnostic is a warning.
    #[must_use]
    pub const fn is_recoverable(self) -> bool {
        matches!(self.code, ErrorCode::DuplicateAttribute)
    }

    /// Whether semantic analysis can safely continue over the recovered tree.
    #[must_use]
    pub fn has_documented_recovery(self) -> bool {
        self.code.has_documented_parse_recovery()
            || matches!(
                self.detail,
                SurfaceErrorDetail::DuplicateAttribute
                    | SurfaceErrorDetail::MissingDirectiveName
                    | SurfaceErrorDetail::AdoptionAgency
                    | SurfaceErrorDetail::IgnoredRepeatedForm
                    | SurfaceErrorDetail::FosterParenting
                    | SurfaceErrorDetail::InputStream
            )
    }
}

fn missing_attribute_name(source: &str, offset: u32) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut end = (offset as usize).min(bytes.len());
    while end > 0 && bytes.get(end - 1).is_some_and(u8::is_ascii_whitespace) {
        end -= 1;
    }
    if end == 0 || bytes.get(end - 1) != Some(&b'=') {
        return None;
    }
    end -= 1;
    let mut start = end;
    while start > 0
        && bytes
            .get(start - 1)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'<' | b'>' | b'/'))
    {
        start -= 1;
    }
    source.get(start..end).filter(|name| !name.is_empty())
}

fn recovery_message(code: ErrorCode) -> Option<CompactString> {
    Some(match code {
        ErrorCode::EofBeforeTagName =>
            "Unexpected end of input after `<`; treating it as text so parsing can continue.",
        ErrorCode::EofInTag =>
            "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue.",
        ErrorCode::EofInComment =>
            "Comment is missing its closing `-->`; preserving the unfinished comment so parsing can finish.",
        ErrorCode::InvalidFirstCharacterOfTagName =>
            "Tag name starts with an invalid character; treating the malformed tag as text.",
        ErrorCode::MissingDynamicDirectiveArgumentEnd =>
            "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary.",
        ErrorCode::MissingInterpolationEnd =>
            "Interpolation is missing its closing delimiter `}}`; treating the unfinished interpolation as text.",
        ErrorCode::UnexpectedCharacterInAttributeName =>
            "Attribute name contains an invalid character; inferred the nearest attribute boundary and continued.",
        ErrorCode::UnexpectedCharacterInUnquotedAttributeValue =>
            "Unquoted attribute value contains a character that should be quoted; keeping it in the value and continuing.",
        ErrorCode::UnexpectedEqualsSignBeforeAttributeName =>
            "Unexpected `=` before an attribute name; skipping it and continuing with the next attribute.",
        ErrorCode::MissingWhitespaceBetweenAttributes =>
            "Missing whitespace between attributes; inferred a new attribute boundary.",
        ErrorCode::IncorrectlyClosedComment =>
            "Comment was closed as `--!>`; treating it as `-->` so parsing can continue.",
        ErrorCode::IncorrectlyOpenedComment =>
            "Declaration or comment syntax is malformed; skipping it until the next `>`.",
        _ => return None,
    }
    .to_compact_string())
}

#[cfg(test)]
mod tests {
    use super::{Span, SurfaceError};

    #[test]
    fn structural_recoveries_keep_semantic_analysis_available() {
        let span = Span::new(4, 8);
        let recoveries = [
            SurfaceError::duplicate_attribute(span),
            SurfaceError::missing_directive_name(span),
            SurfaceError::adoption_agency(span),
            SurfaceError::ignored_repeated_form(span),
            SurfaceError::foster_parenting(span),
        ];

        assert!(
            recoveries
                .into_iter()
                .all(SurfaceError::has_documented_recovery)
        );
    }

    #[test]
    fn exhausted_nesting_remains_fatal_for_semantic_analysis() {
        assert!(!SurfaceError::nesting_too_deep(Span::new(4, 8)).has_documented_recovery());
    }
}
