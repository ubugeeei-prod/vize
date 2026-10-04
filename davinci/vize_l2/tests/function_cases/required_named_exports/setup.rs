use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::{
    file::{DeclarationKind, Namespace},
    lang::js::{
        FileProducer, NativeSetupIssueKind, NativeTemplateIssueKind, NativeTemplateOwner,
        ProgramInput, ProgramScope, SetupIssueKind, VueSetup,
    },
};

fn descriptor<'a>(arena: &'a Allocator, source: &'a str) -> DescriptorObservation<'a> {
    Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}

#[test]
fn a_complete_original_named_export_never_grants_setup_or_ordinary_eligibility() {
    let arena = Allocator::default();
    let source = "<template>{{ primitive }}</template><script setup lang=ts>/* original 🌸 */export function f(value:number):number{return value;}const primitive=1;</script>";
    let observation = descriptor(&arena, source);
    let admitted = observation.admitted().unwrap();
    let script = admitted.setup().unwrap();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    );
    let original = syntax.admitted_program().unwrap();
    let body = original.program().body.as_ptr();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(original, script.block(), script.container_index()).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.scopes().len(), 3);
    let parameter = file
        .lookup(file.scopes()[2].id, "value", Namespace::Value)
        .unwrap();
    assert_eq!(
        parameter.declaration().unwrap().kind,
        DeclarationKind::Parameter
    );
    assert_eq!(parameter.declaration().unwrap().span.slice(source), "value");
    assert!(
        matches!(VueSetup::checked(&file, script, syntax.admitted_program().unwrap()),
        Err(issue) if issue.kind == SetupIssueKind::UnsupportedSyntax)
    );
    assert!(file.ordinary_empty_script().is_none());
    let function = file
        .lookup(file.scopes()[1].id, "f", Namespace::Value)
        .unwrap();
    assert_eq!(file.exports().len(), 1);
    assert_eq!(file.exports()[0].local, Some(function.id()));
    assert_eq!(file.exports()[0].unit, file.units()[0].id);
    assert_eq!(syntax.program().unwrap().body.as_ptr(), body);
    assert_eq!(syntax.comments().count(), 1);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
}

#[test]
fn the_original_named_export_setup_cursor_refuses_before_consuming_any_template_child() {
    let arena = Allocator::default();
    let source = "<template>{{ primitive }}</template><script setup lang=ts>/* original */export function f(value:number):number{return value;}const primitive=1;</script>";
    let observation = descriptor(&arena, source);
    let selected = NativeTemplateComponent::parse_in(&arena, observation.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner =
        NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("original selected owner"));
    owner.parse_setup_program().unwrap();
    let syntax = owner.retained_setup().unwrap();
    let body = syntax.program().unwrap().body.as_ptr();
    let span = syntax.source().span();
    assert_eq!(syntax.comments().count(), 1);
    assert!(matches!(owner.begin_setup(),
        Err(issue) if issue.span == span && issue.kind == NativeTemplateIssueKind::SetupPolicy(NativeSetupIssueKind::UnsupportedSyntax)));
    let output = Box::new(owner.finish());
    assert!(output.view().is_err());
    let syntax = output.retained_setup().unwrap();
    assert_eq!(syntax.program().unwrap().body.as_ptr(), body);
    assert_eq!(syntax.source().span(), span);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    let file = output.file().unwrap();
    assert!(file.native_interpolations().is_empty());
    assert!(
        file.bindings()
            .any(|binding| binding.declaration().is_some_and(|declaration| {
                declaration.kind == DeclarationKind::Parameter
                    && declaration.span.slice(source) == "value"
            }))
    );
}
