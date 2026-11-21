//! Read the checker's existing script diagnostic format from an original parse.
use super::{ScriptParseDiagnostic, diagnostic_span, script_source_type};
use oxc_span::{FileExtension, SourceType};
use vize_carton::ToCompactString;
use vize_l1::embed::authored_program::AuthoredProgram;

/// The checker grammar for one authored SFC script. For example, a plain JS
/// editor buffer and a JS block inspected by Canon can request different grammars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptDiagnosticProfile(SourceType);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptDiagnosticRefusal {
    DifferentGrammar,
    OffsetOverflow,
}

impl ScriptDiagnosticProfile {
    #[must_use]
    pub fn for_language(lang: Option<&str>) -> Self {
        Self(script_source_type(lang))
    }

    /// Select the existing standalone `.ts` parse for a plain TypeScript
    /// diagnostic reader. Both request the same unambiguous grammar. The
    /// parser distinguishes `.mts` and `.cts` at angle-bracket assertions and
    /// generic arrows, so those profiles and declaration files stay separate.
    #[must_use]
    pub fn sharing_plain_typescript(self, available: Option<SourceType>) -> Self {
        let with_filename = SourceType::from(FileExtension::Ts);
        if self.0 == SourceType::ts() && available == Some(with_filename) {
            Self(with_filename)
        } else {
            self
        }
    }

    #[must_use]
    pub const fn source_type(self) -> SourceType {
        self.0
    }

    pub(crate) fn read(
        self,
        syntax: &AuthoredProgram<'_>,
    ) -> Result<Vec<ScriptParseDiagnostic>, ScriptDiagnosticRefusal> {
        if self.0 != syntax.source_type() {
            return Err(ScriptDiagnosticRefusal::DifferentGrammar);
        }
        let block = syntax.block();
        let length = block.source().len();
        let absolute = |local| {
            block
                .start()
                .checked_add(local)
                .ok_or(ScriptDiagnosticRefusal::OffsetOverflow)
        };
        let parsed = syntax.parsed();
        let mut diagnostics = Vec::with_capacity(parsed.diagnostics.len());
        for error in &parsed.diagnostics {
            let (start, end) = diagnostic_span(error, length);
            if end < start {
                return Err(ScriptDiagnosticRefusal::OffsetOverflow);
            }
            diagnostics.push(ScriptParseDiagnostic {
                message: error.to_compact_string(),
                start: absolute(start)?,
                end: absolute(end)?,
            });
        }
        if parsed.panicked && diagnostics.is_empty() {
            diagnostics.push(ScriptParseDiagnostic {
                message: "Parser panicked while parsing script".into(),
                start: block.start(),
                end: absolute((length as u32).max(1))?,
            });
        }
        Ok(diagnostics)
    }
}

#[cfg(test)]
mod tests;
