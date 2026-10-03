//! Exact/Error consumer of genuine direct-child marker counterexamples.
use super::{
    NO_TEXTAREA_MUSTACHE_RULE, NativeLintFinding,
    child_facts::{NativeChildFacts, NativeDirectInterpolations, TextareaMustacheDemand},
    header_facts::{NativeLintAttributes, NativeLintHeaders},
};
use vize_l0::{
    Span,
    diag::{Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage, verify::WitnessError},
    fact::{Demand, FactConsumer, FactError, ids},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTextareaLintError {
    Fact(FactError),
    Witness(WitnessError),
    IncompleteEvidence { child: u32 },
    IncompleteHeaderContext,
    InheritedLiteralContext { span: Span },
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

// Read the existing complete authentic header tables. This demand changes
// neither the header registry nor the supplied-child presence-only registry.
struct LiteralContextDemand;
impl FactConsumer for LiteralContextDemand {
    const NAME: &'static str = "native-textarea-literal-context";
    const DEMAND: Demand = Demand::NONE
        .with(ids::NATIVE_LINT_HEADERS)
        .with(ids::NATIVE_LINT_ATTRIBUTES);
}

impl NativeChildFacts<'_, '_, '_> {
    /// Emit the unchanged registered rule's Error after verifying the original
    /// exact header, full marker and derived counterexample causal chain.
    /// No expression parse, sibling walk or whole-body absence claim occurs.
    pub fn no_textarea_mustache(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeTextareaLintError> {
        self.check_literal_context()?;
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

    fn check_literal_context(&self) -> Result<(), NativeTextareaLintError> {
        let facts = self.header().facts::<LiteralContextDemand>();
        let headers = facts.get::<NativeLintHeaders>()?;
        let (_, header) = headers
            .iter()
            .next()
            .ok_or(NativeTextareaLintError::IncompleteHeaderContext)?;
        if header.tag() == "textarea"
            && header.header_is_literal()
            && !facts
                .get::<NativeLintAttributes>()?
                .iter()
                .any(|(_, attribute)| attribute.name() == "v-pre")
        {
            // The original registered L2 slot route can discard inherited pre
            // while selected L1 still preserves literal text. Neither absence
            // nor a fabricated marker can authenticate that facade meaning.
            return Err(NativeTextareaLintError::InheritedLiteralContext {
                span: header.opening(),
            });
        }
        Ok(())
    }
}
