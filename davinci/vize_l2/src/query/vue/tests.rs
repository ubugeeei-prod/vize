extern crate std;

use super::*;
use crate::lang::js::{L2IrProducer, ProgramInput, ProgramScope};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorOptions, L1Ir, ScriptRole, ScriptView},
    },
    embed::{
        EmbedSource,
        syntax::{EmbedSyntax, ProgramOptions, parse_program_once},
    },
    markup::SurfaceParseOptions,
};

mod corpus;
mod profile;

const ALL: VueDemand = VueDemand {
    setup: true,
    ordinary: Some(OrdinaryDemand::Both),
};

struct Parsed<'a> {
    l1ir: L1Ir<'a>,
    ordinary: Option<EmbedSyntax<'a>>,
    setup: Option<EmbedSyntax<'a>>,
}

impl<'a> Parsed<'a> {
    fn new(arena: &'a Allocator, source: &'a str) -> Option<Self> {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let view = descriptor.admitted().ok()?;
        let syntax = |script: ScriptView<'_, 'a>| {
            parse_program_once(
                arena,
                EmbedSource::authored(source, script.block().span()).unwrap(),
                ProgramOptions::module(script.lang()),
            )
        };
        let ordinary = view.ordinary().map(syntax);
        let setup = view.setup().map(syntax);
        Some(Self {
            l1ir: descriptor,
            ordinary,
            setup,
        })
    }

    fn produce(
        &self,
        arena: &'a Allocator,
        demand: VueDemand,
    ) -> Result<VueL2Ir<'a>, VueProgramError> {
        VueL2IrProducer::scripts(
            arena,
            self.l1ir.admitted().unwrap(),
            self.ordinary
                .as_ref()
                .and_then(EmbedSyntax::admitted_program),
            self.setup.as_ref().and_then(EmbedSyntax::admitted_program),
            demand,
        )
        .map(|producer| producer.finish().unwrap())
    }
}

#[test]
fn same_text_and_numeric_ids_cannot_attach_another_files_facts_or_binding() {
    let arena = Allocator::default();
    let parsed = Parsed::new(&arena, "<script setup>const x = 1</script>").unwrap();
    let a = parsed.produce(&arena, ALL).unwrap();
    let b = parsed.produce(&arena, ALL).unwrap();
    let foreign = b.l2ir().bindings().next().unwrap();
    let local = a.l2ir().bindings().next().unwrap();
    assert_eq!(local.id(), foreign.id());
    assert!(matches!(
        Model::new(a.l2ir()).with_vue(&b),
        Err(VueQueryError::ForeignFile)
    ));
    assert_eq!(
        a.model().binding_kind(foreign),
        Err(VueQueryError::ForeignFile)
    );
    let attached = Model::new(a.l2ir()).with_vue(&a).unwrap();
    assert_eq!(attached.binding_kind(local), a.model().binding_kind(local));
    assert!(core::ptr::eq(
        attached.contract().unwrap(),
        a.model().contract().unwrap()
    ));
    let raw = Model::new(a.l2ir()).vue();
    assert_eq!(
        raw.binding_kind(local),
        Err(VueQueryError::NotObserved(VueFamily::BindingKind))
    );
    assert_eq!(
        raw.macro_calls(),
        Err(VueQueryError::NotObserved(VueFamily::Setup))
    );
}

#[test]
fn macros_contract_options_and_shadowed_declarations_retain_their_authored_roles() {
    let arena = Allocator::default();
    let source = r#"<script>const value = 0; export default { props: ['title'], inject: ['service'] }</script>
<script setup>import { ref, defineProps } from 'vue';
const label = 1; const count = ref(0); const props = defineProps(['title']);
function nested() { const count = 2; return count }
</script>"#;
    let file = Parsed::new(&arena, source)
        .unwrap()
        .produce(&arena, ALL)
        .unwrap();
    assert!(file.l2ir().is_complete(), "{:?}", file.l2ir().issues());
    let model = file.model();
    let binding = |needle: &str| {
        model
            .binding_at_offset(source.find(needle).unwrap() as u32)
            .unwrap()
            .unwrap()
    };
    assert!(matches!(
        model.binding_kind(binding("value =")),
        Ok(Some(VueBinding::Ordinary(_)))
    ));
    assert!(matches!(
        model.binding_kind(binding("label =")),
        Ok(Some(VueBinding::Setup(SetupBinding {
            kind: SetupBindingKind::SetupConst,
            ..
        })))
    ));
    assert!(matches!(
        model.binding_kind(binding("count = ref")),
        Ok(Some(VueBinding::Setup(SetupBinding {
            kind: SetupBindingKind::SetupRef,
            ..
        })))
    ));
    assert_eq!(model.binding_kind(binding("count = 2")), Ok(None));
    assert_eq!(model.macro_calls().unwrap()[0].kind, MacroKind::DefineProps);
    assert!(model.contract().unwrap().props.is_some());
    let options = model.options().unwrap().unwrap();
    let members: alloc::vec::Vec<_> = options
        .members
        .iter()
        .map(|m| &source[m.name.start as usize..m.name.end as usize])
        .collect();
    assert_eq!(members, ["title", "service"]);
}

#[test]
fn a_component_without_options_differs_from_an_unobserved_options_family() {
    let arena = Allocator::default();
    let parsed = Parsed::new(&arena, "<script>export const helper = 1</script>").unwrap();
    for demand in [
        None,
        Some(OrdinaryDemand::Options),
        Some(OrdinaryDemand::Bindings),
        Some(OrdinaryDemand::Both),
    ] {
        let owner = parsed
            .produce(
                &arena,
                VueDemand {
                    setup: false,
                    ordinary: demand,
                },
            )
            .unwrap();
        let binding = owner.l2ir().bindings().next().unwrap();
        let model = owner.model();
        if matches!(demand, Some(OrdinaryDemand::Options | OrdinaryDemand::Both)) {
            assert_eq!(model.options(), Ok(None));
        } else {
            assert_eq!(
                model.options(),
                Err(VueQueryError::NotObserved(VueFamily::Options))
            );
        }
        match demand {
            Some(OrdinaryDemand::Bindings | OrdinaryDemand::Both) => assert!(matches!(
                model.binding_kind(binding),
                Ok(Some(VueBinding::Ordinary(_)))
            )),
            Some(OrdinaryDemand::Options) => assert_eq!(
                model.binding_kind(binding),
                Err(VueQueryError::NotObserved(VueFamily::OrdinaryBindings))
            ),
            None => assert_eq!(
                model.binding_kind(binding),
                Err(VueQueryError::NotObserved(VueFamily::BindingKind))
            ),
        }
    }
}

#[test]
fn incomplete_files_refuse_even_locally_observed_declarations_and_contracts() {
    let arena = Allocator::default();
    for source in [
        "<script setup>const props = defineProps(['key'])</script>",
        "<script setup>const valid = 1; missing;</script>",
    ] {
        let file = Parsed::new(&arena, source)
            .unwrap()
            .produce(&arena, ALL)
            .unwrap();
        assert!(!file.l2ir().is_complete());
        let model = file.model();
        assert_eq!(model.contract(), Err(VueQueryError::IncompleteFile));
        assert_eq!(model.options(), Err(VueQueryError::IncompleteFile));
        assert_eq!(
            model.binding_kind(file.l2ir().bindings().next().unwrap()),
            Err(VueQueryError::IncompleteFile)
        );
    }
}

#[test]
fn scripts_require_the_exact_descriptor_role_original_source_and_language() {
    let arena = Allocator::default();
    let parsed = Parsed::new(&arena, "<script setup lang='ts'>const x = 1</script>").unwrap();
    let view = parsed.l1ir.admitted().unwrap();
    let setup = || parsed.setup.as_ref().unwrap().admitted_program().unwrap();
    assert!(matches!(
        VueL2IrProducer::scripts(&arena, view, None, None, ALL),
        Err(VueProgramError::Missing(ScriptRole::Setup))
    ));
    assert!(matches!(
        VueL2IrProducer::scripts(&arena, view, Some(setup()), Some(setup()), ALL),
        Err(VueProgramError::Unexpected(ScriptRole::Ordinary))
    ));
    let copy = vize_l0::String::from(view.source());
    let other = Parsed::new(&arena, &copy).unwrap();
    let foreign = other.setup.as_ref().unwrap().admitted_program().unwrap();
    assert!(
        matches!(VueL2IrProducer::scripts(&arena, view, None, Some(foreign), ALL), Err(VueProgramError::Program(e)) if e.kind == crate::ir::L2IrIssueKind::InvalidSource)
    );
    let wrong = oxc_parser::Parser::new(&arena, setup().source(), oxc_span::SourceType::mjs())
        .parse_observed();
    assert!(
        matches!(VueL2IrProducer::scripts(&arena, view, None, wrong.admitted(), ALL), Err(VueProgramError::Program(e)) if e.kind == crate::ir::L2IrIssueKind::InvalidProfile)
    );
}

#[test]
fn a_type_import_has_no_value_binding_kind() {
    let arena = Allocator::default();
    let parsed = Parsed::new(
        &arena,
        "<script setup lang=ts>import type { T } from 'types'; const x = 1;</script>",
    )
    .unwrap();
    let owner = parsed.produce(&arena, ALL).unwrap();
    assert!(owner.l2ir().is_complete(), "{:?}", owner.l2ir().issues());
    let binding = owner
        .l2ir()
        .bindings()
        .find(|b| b.declaration().is_some_and(|d| d.name == "T"))
        .unwrap();
    assert_eq!(owner.model().binding_kind(binding), Ok(None));
}

#[test]
fn a_refused_destructure_does_not_publish_partial_contracts() {
    let arena = Allocator::default();
    let parsed = Parsed::new(&arena, "<script setup>import { defineProps } from 'vue'; const { a } = defineProps(['a']); const { b } = defineProps(['b']);</script>").unwrap();
    let owner = parsed.produce(&arena, ALL).unwrap();
    assert!(
        owner
            .l2ir()
            .issues()
            .iter()
            .any(|issue| issue.kind == crate::ir::L2IrIssueKind::UnsupportedSyntax)
    );
    assert_eq!(owner.model().contract(), Err(VueQueryError::IncompleteFile));
}
