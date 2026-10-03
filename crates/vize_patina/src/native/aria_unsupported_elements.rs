//! Exact/Error consumer of genuine original-header SDK counterexamples.

use vize_l0::{
    SmallVec, String,
    diag::{Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage, verify::WitnessError},
    fact::FactError,
};

use super::{
    ARIA_UNSUPPORTED_ELEMENTS_RULE, NativeLintFinding,
    header_facts::{
        NativeHeaderFacts, NativeLintAttributes, NativeLintHeaders, UnsupportedAriaDemand,
    },
};

/// SDK failures remain typed; no failure can become an unwitnessed Error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeAriaLintError {
    Fact(FactError),
    Witness(WitnessError),
    IncompleteEvidence { attribute: u32 },
}
impl From<FactError> for NativeAriaLintError {
    fn from(error: FactError) -> Self {
        Self::Fact(error)
    }
}
impl From<WitnessError> for NativeAriaLintError {
    fn from(error: WitnessError) -> Self {
        Self::Witness(error)
    }
}

impl NativeHeaderFacts<'_, '_> {
    /// Emit the unchanged rule's Error only after verifying every causal SDK
    /// chain against this authentic header's exact declared fact view.
    /// Complete original attribute ranges and owned full Help are retained.
    pub fn aria_unsupported_elements(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeAriaLintError> {
        let facts = self.facts::<UnsupportedAriaDemand>();
        let headers = facts.get::<NativeLintHeaders>()?;
        let attributes = facts.get::<NativeLintAttributes>()?;
        let mut proven: SmallVec<[_; 4]> = SmallVec::new();
        for (key, counterexample) in self.unsupported_aria()?.iter() {
            let missing = || NativeAriaLintError::IncompleteEvidence { attribute: *key };
            let header = headers
                .get(&counterexample.header_key())
                .ok_or_else(missing)?;
            let attribute = attributes.get(key).ok_or_else(missing)?;
            let chain = self.unsupported_aria_chain(*key)?.ok_or_else(missing)?;
            self.verify(&chain)?;
            proven.push((header.tag(), attribute.name(), counterexample.span(), chain));
        }
        // An empty or exempt header cannot touch a catalog. Every chain is
        // verified before any localized result, so failures are whole-result.
        Ok(proven
            .into_iter()
            .map(|(tag, attribute, range, chain)| {
                let template = messages.lookup("a11y/aria-unsupported-elements.message");
                let message = substitute(
                    &substitute(template.as_ref(), "{tag}", tag),
                    "{attr}",
                    attribute,
                );
                NativeLintFinding {
                    rule_name: ARIA_UNSUPPORTED_ELEMENTS_RULE,
                    diagnostic: Diagnostic::proven(Stage::Surface, range, message.as_str(), chain)
                        .with_part(DiagnosticPart::new(
                            PartKind::Help,
                            range,
                            messages
                                .lookup("a11y/aria-unsupported-elements.help")
                                .as_ref(),
                        )),
                }
            })
            .collect())
    }
}

// Preserve the actual catalog's ordered tag-then-attribute substitutions,
// including repeated placeholders, while keeping the host lookup portable.
fn substitute(template: &str, placeholder: &str, value: &str) -> String {
    let mut result = String::new("");
    for (index, piece) in template.split(placeholder).enumerate() {
        if index != 0 {
            result.push_str(value);
        }
        result.push_str(piece);
    }
    result
}
