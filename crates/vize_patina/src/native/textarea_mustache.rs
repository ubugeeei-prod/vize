//! Exact/Error consumer of genuine direct-child marker counterexamples.
use super::{
    NO_TEXTAREA_MUSTACHE_RULE, NativeLintFinding,
    child_facts::{NativeChildFacts, NativeDirectInterpolations, TextareaMustacheDemand},
    header_facts::NativeLintHeaders,
};
use vize_l0::{
    diag::{Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage, verify::WitnessError},
    fact::FactError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTextareaLintError {
    Fact(FactError),
    Witness(WitnessError),
    IncompleteEvidence { child: u32 },
}
impl From<FactError> for NativeTextareaLintError {
    fn from(error: FactError) -> Self {
        Self::Fact(error)
    }
}
impl From<WitnessError> for NativeTextareaLintError {
    fn from(error: WitnessError) -> Self {
        Self::Witness(error)
    }
}

impl NativeChildFacts<'_, '_, '_> {
    /// Emit the unchanged registered rule's Error after verifying the original
    /// exact header, full marker and derived counterexample causal chain.
    /// No expression parse, sibling walk or whole-body absence claim occurs.
    pub fn no_textarea_mustache(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeTextareaLintError> {
        let facts = self.facts::<TextareaMustacheDemand>();
        let headers = facts.get::<NativeLintHeaders>()?;
        let markers = facts.get::<NativeDirectInterpolations>()?;
        let mut proven = Vec::new();
        for (key, counterexample) in self.textarea_mustache()?.iter() {
            let missing = || NativeTextareaLintError::IncompleteEvidence { child: *key };
            headers
                .get(&counterexample.header_key())
                .ok_or_else(missing)?;
            markers.get(key).ok_or_else(missing)?;
            let chain = self.textarea_mustache_chain(*key)?.ok_or_else(missing)?;
            self.verify(&chain)?;
            proven.push((counterexample.span(), chain));
        }
        if proven.is_empty() {
            return Ok(Vec::new());
        }
        let message = messages.lookup("vue/no-textarea-mustache.message");
        let help = messages.lookup("vue/no-textarea-mustache.help");
        Ok(proven
            .into_iter()
            .map(|(range, chain)| NativeLintFinding {
                rule_name: NO_TEXTAREA_MUSTACHE_RULE,
                diagnostic: Diagnostic::proven(Stage::Surface, range, message.as_ref(), chain)
                    .with_part(DiagnosticPart::new(PartKind::Help, range, help.as_ref())),
            })
            .collect())
    }
}
