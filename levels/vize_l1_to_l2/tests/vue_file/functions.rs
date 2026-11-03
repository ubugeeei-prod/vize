use super::support::{construct, script};
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l1_to_l2::vue_file::{VueFileIssueKind, VueFileProducer, VueScriptRole};
use vize_l2::file::{DeclarationKind, Namespace};
use vize_l2::lang::js::ProgramInput;

#[test]
fn setup_function_exposure_uses_only_actual_unit_scope_declarations() {
    let arena = Allocator::default();
    let content = "const title = 1; function show(title) { const local = title; return local; }";
    let source = "é<script setup>const title = 1; function show(title) { const local = title; return local; }</script><template><p>{{show(title)}}</p></template>";
    let (syntax, block) = script(&arena, source, content, Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let native = construct(&arena, source, "<p>{{show(title)}}</p>", &mut producer).unwrap();
    assert!(native.is_supported());
    let vue = producer.finish().unwrap();
    let scope = vue.setup().unwrap().scope();
    assert_eq!(vue.declarations().len(), 2);
    assert_eq!(vue.file().bindings().count(), 4);
    for binding in vue.file().bindings() {
        let declaration = binding.declaration().unwrap();
        if declaration.scope == scope {
            assert_eq!(vue.exposure(binding).unwrap().role(), VueScriptRole::Setup);
        } else {
            assert!(vue.exposure(binding).is_none());
            assert!(matches!(
                declaration.kind,
                DeclarationKind::Parameter | DeclarationKind::Const
            ));
        }
    }
    assert!(
        vue.file()
            .lookup(scope, "local", Namespace::Value)
            .is_none()
    );
    let resolution = vue
        .file()
        .expression(native.embeds[0].node.unwrap())
        .unwrap();
    assert_eq!(resolution.scope(), Some(scope));
    let table = resolution.table().unwrap();
    assert!(core::ptr::eq(
        table.expression().ast,
        native.embeds[0].syntax.expression().unwrap()
    ));
    assert_eq!(table.occurrences().len(), 2);
    for occurrence in table.occurrences() {
        let binding = resolution.binding(occurrence.binding).unwrap();
        assert_eq!(binding.declaration().unwrap().scope, scope);
        assert!(vue.exposure(binding).is_some());
    }
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(syntax.program().unwrap().body.len(), 2);
}

#[test]
fn ordinary_only_functions_have_no_template_exposure_and_combined_roles_expose_top_level_only() {
    let arena = Allocator::default();
    let ordinary = "export function outer(value) { const local = value; return local; }";
    let setup = "function show(value) { return value + 1; }";
    let source = "<script>export function outer(value) { const local = value; return local; }</script><script setup>function show(value) { return value + 1; }</script><template>{{outer(show)}}</template>";
    let (ordinary_syntax, ordinary_block) = script(&arena, source, ordinary, Lang::Js).unwrap();
    let (setup_syntax, setup_block) = script(&arena, source, setup, Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .ordinary(
            ProgramInput::checked(
                ordinary_syntax.admitted_program().unwrap(),
                ordinary_block,
                0,
            )
            .unwrap(),
        )
        .unwrap();
    let ordinary_file = producer.finish().unwrap();
    assert!(
        ordinary_file
            .file()
            .bindings()
            .all(|binding| ordinary_file.exposure(binding).is_none())
    );
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .ordinary(
            ProgramInput::checked(
                ordinary_syntax.admitted_program().unwrap(),
                ordinary_block,
                0,
            )
            .unwrap(),
        )
        .unwrap();
    producer
        .setup(
            ProgramInput::checked(setup_syntax.admitted_program().unwrap(), setup_block, 1)
                .unwrap(),
        )
        .unwrap();
    let native = construct(&arena, source, "{{outer(show)}}", &mut producer).unwrap();
    assert!(native.is_supported());
    let vue = producer.finish().unwrap();
    assert_eq!(vue.declarations().len(), 2);
    assert_eq!(vue.file().bindings().count(), 5);
    let ordinary_receipt = vue.ordinary().unwrap();
    let setup_receipt = vue.setup().unwrap();
    assert_eq!(
        vue.file().scopes()[setup_receipt.scope().index() as usize].parent,
        Some(ordinary_receipt.scope())
    );
    for binding in vue.file().bindings() {
        let declaration = binding.declaration().unwrap();
        if declaration.scope == ordinary_receipt.scope() {
            assert_eq!(
                vue.exposure(binding).unwrap().role(),
                VueScriptRole::Ordinary
            );
        } else if declaration.scope == setup_receipt.scope() {
            assert_eq!(vue.exposure(binding).unwrap().role(), VueScriptRole::Setup);
        } else {
            assert!(vue.exposure(binding).is_none());
        }
    }
    assert_eq!(
        vue.file().exports()[0].local,
        vue.file()
            .lookup(ordinary_receipt.scope(), "outer", Namespace::Value)
            .map(|binding| binding.id())
    );
}

#[test]
fn child_locals_cannot_resolve_a_template_use_and_original_observations_survive() {
    let arena = Allocator::default();
    let content = "/* original */ function show(value) { const local = value; return local; }";
    let source = "<script setup>/* original */ function show(value) { const local = value; return local; }</script><template>{{local}}</template>";
    let (syntax, block) = script(&arena, source, content, Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let native = construct(&arena, source, "{{local}}", &mut producer).unwrap();
    assert!(!native.is_supported());
    let rejected = producer.finish().unwrap_err();
    let file = rejected.file().unwrap();
    assert_eq!(file.bindings().count(), 3);
    assert_eq!(file.scopes().len(), 3);
    assert!(!file.is_complete());
    assert!(!file.template_issues().is_empty());
    assert_eq!(syntax.program().unwrap().comments.len(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(native.component.block().source(), "{{local}}");
}

#[test]
fn same_name_define_props_and_recursive_script_calls_remain_typed_unfinished() {
    let arena = Allocator::default();
    for source in [
        "function defineProps(value) { return value; } const props = defineProps(['name']);",
        "function f(value) { return f(value); }",
    ] {
        let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer
            .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
            .unwrap();
        let rejected = producer.finish().unwrap_err();
        assert!(rejected.issues().iter().any(|issue| matches!(
            issue.kind,
            VueFileIssueKind::UnsupportedCall { optional: false }
        )));
        assert!(!rejected.file().unwrap().is_complete());
        assert_eq!(syntax.diagnostics().count(), 0);
        assert!(!syntax.program().unwrap().body.is_empty());
    }
}
