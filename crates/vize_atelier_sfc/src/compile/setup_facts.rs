//! Native `<script setup>` analysis for the compile route.
//!
//! The prepared setup content is parsed once by L1 with the block's `lang`,
//! and the L2 Program walk runs the Vue setup observer over that Program. A
//! setup that does not parse is a compile error carrying the parse
//! diagnostics, as in Vue. The binding metadata and the script tables the
//! template lanes read come from the observed facts.

use oxc_ast::ast::Program;
use vize_atelier_core::lane::ScriptTables;
use vize_l0::{Allocator, SourceRoot, Span, String, ToCompactString};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{NativeSyntax, ProgramOptions, parse_program_once},
};
use vize_l2::framework::vue::{SetupBindingKind, SetupFacts, SetupObserver};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

use crate::types::{BindingMetadata, BindingType, SfcDescriptor, SfcError};

/// The setup content's Program and the facts observed over it.
pub(crate) struct SetupAnalysis<'a> {
    content: &'a str,
    syntax: NativeSyntax<'a>,
    facts: SetupFacts,
}

impl<'a> SetupAnalysis<'a> {
    /// Parse and observe `content`, the prepared text of the descriptor's
    /// `<script setup>`.
    pub(crate) fn new(
        arena: &'a Allocator,
        content: &'a str,
        descriptor: &SfcDescriptor<'_>,
    ) -> Result<Self, SfcError> {
        let lang = descriptor
            .script_setup
            .as_ref()
            .and_then(|setup| setup.lang.as_deref());
        let span = u32::try_from(content.len())
            .map(|end| Span::new(0, end))
            .map_err(|_| refused("the setup content exceeds 4 GiB"))?;
        let source = EmbedSource::authored(content, span).map_err(refused)?;
        let syntax = parse_program_once(arena, source, program_options(lang));
        let Some(admitted) = syntax.admitted_program() else {
            return Err(syntax_error(&syntax, content, descriptor));
        };
        let block = SourceRoot::new(content).map_err(refused)?.whole_block();
        let input = ProgramInput::checked(admitted, block, 0).map_err(refused)?;
        let mut producer = FileProducer::new(arena, content).map_err(refused)?;
        let mut observer = SetupObserver::new();
        producer
            .program_observed(input, ProgramScope::Nested, &mut observer)
            .map_err(refused)?;
        let facts = observer.finish().map_err(refused)?;
        Ok(Self {
            content,
            syntax,
            facts,
        })
    }

    /// The setup Program the legacy compile stages still walk.
    pub(super) fn program(&self) -> Option<&Program<'a>> {
        self.syntax.program()
    }

    /// The binding metadata of the setup's runtime-declared props and
    /// top-level declarations, and the reactive sources and component
    /// registrations the template lanes read.
    ///
    /// A `defineProps` destructure the facts leave unclassified is refused;
    /// the compile context reports the patterns Vue rejects before this.
    pub(crate) fn tables(&self) -> Result<(BindingMetadata, ScriptTables), SfcError> {
        if let Some(pattern) = self.facts.unclassified().first() {
            return Err(refused(format_args!(
                "unsupported defineProps destructure `{}`",
                self.text(*pattern)?
            )));
        }
        let mut metadata = BindingMetadata {
            is_script_setup: true,
            ..Default::default()
        };
        // Declared props first: a setup declaration of the same name owns
        // the binding, as in Vue.
        for key in self.facts.runtime_prop_keys() {
            metadata
                .bindings
                .insert(self.text(*key)?.to_compact_string(), BindingType::Props);
        }
        let mut reactive = Vec::new();
        let mut components = Vec::new();
        for binding in self.facts.bindings() {
            let name = self.text(binding.name)?;
            metadata
                .bindings
                .insert(name.to_compact_string(), binding_type(binding.kind));
            if let Some(source) = binding.source {
                reactive.push((name, source));
            }
            if binding.kind == SetupBindingKind::SetupConst {
                components.push(name);
            }
        }
        if let Some(props) = self.facts.props_destructure() {
            for field in &props.fields {
                let (local, key) = (self.text(field.local)?, self.text(field.key)?);
                if local != key {
                    metadata
                        .props_aliases
                        .insert(local.to_compact_string(), key.to_compact_string());
                }
            }
        }
        Ok((metadata, ScriptTables::new(reactive, components)))
    }

    fn text(&self, span: Span) -> Result<&'a str, SfcError> {
        self.content
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| refused(format_args!("a fact span {span:?} outside the content")))
    }
}

/// The parser profile for `<script setup lang>`: JavaScript for no `lang`,
/// `js` and `jsx`, as in Vue; TypeScript for every other `lang`, which is
/// also how the compiler reads a language Vue leaves uncompiled (`uts`).
/// `jsx` and `tsx` enable JSX.
fn program_options(lang: Option<&str>) -> ProgramOptions {
    let javascript = matches!(lang, None | Some("js" | "jsx"));
    let mut options = ProgramOptions::module(if javascript { Lang::Js } else { Lang::Ts });
    options.jsx = matches!(lang, Some("jsx" | "tsx"));
    options
}

const fn binding_type(kind: SetupBindingKind) -> BindingType {
    match kind {
        SetupBindingKind::SetupConst => BindingType::SetupConst,
        SetupBindingKind::SetupLet => BindingType::SetupLet,
        SetupBindingKind::SetupRef => BindingType::SetupRef,
        SetupBindingKind::SetupMaybeRef => BindingType::SetupMaybeRef,
        SetupBindingKind::SetupReactiveConst => BindingType::SetupReactiveConst,
        SetupBindingKind::LiteralConst => BindingType::LiteralConst,
        SetupBindingKind::Props => BindingType::Props,
        SetupBindingKind::PropsAliased => BindingType::PropsAliased,
    }
}

/// The parse diagnostics of a setup that does not parse, located in the
/// `.vue` file when `content` is the authored setup text.
fn syntax_error(
    syntax: &NativeSyntax<'_>,
    content: &str,
    descriptor: &SfcDescriptor<'_>,
) -> SfcError {
    let mut message = String::from("<script setup> does not parse");
    let mut first = None;
    for diagnostic in syntax.diagnostics() {
        message.push_str("\n  ");
        message.push_str(diagnostic.message());
        let span = diagnostic
            .labels()
            .find(|label| label.primary())
            .or_else(|| diagnostic.labels().next())
            .and_then(|label| label.authored_span().ok());
        first = first.or(span);
    }
    if let Some(hole) = syntax.hole()
        && first.is_none()
    {
        message.push_str(&vize_l0::cstr!(": {hole:?}"));
    }
    let setup = descriptor.script_setup.as_ref();
    let loc = setup.map(|setup| match first {
        Some(span) if content == setup.content.as_ref() => super::setup_exports::location(
            &descriptor.source,
            setup.loc.start + span.start as usize,
            setup.loc.start + span.end as usize,
        ),
        _ => setup.loc.clone(),
    });
    SfcError {
        message: message.as_str().into(),
        code: Some("SCRIPT_SETUP_SYNTAX".into()),
        loc,
    }
}

/// A native analysis refusal of a setup that parses.
fn refused(reason: impl core::fmt::Debug) -> SfcError {
    SfcError {
        message: vize_l0::cstr!("<script setup> analysis refused: {reason:?}")
            .as_str()
            .into(),
        code: Some("SCRIPT_SETUP_ANALYSIS".into()),
        loc: None,
    }
}
