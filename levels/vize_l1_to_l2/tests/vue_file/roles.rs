use super::support::{construct, script};
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l1_to_l2::vue_file::{VueFileProducer, VueScriptRole};
use vize_l2::file::Namespace;
use vize_l2::lang::js::ProgramInput;

#[test]
fn setup_only_native_template_resolves_bind_and_interpolation_at_actual_sites() {
    let arena = Allocator::default();
    let source = "é<script setup>const 日本語 = 1; let title = 2;</script><template><div :title=\"title\">{{日本語 + title}}</div></template>";
    let (syntax, block) =
        script(&arena, source, "const 日本語 = 1; let title = 2;", Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let native = construct(
        &arena,
        source,
        "<div :title=\"title\">{{日本語 + title}}</div>",
        &mut producer,
    )
    .unwrap();
    assert!(native.is_supported(), "{:?}", native.holes);
    let vue = producer.finish().unwrap();
    assert!(vue.ordinary().is_none());
    let receipt = vue.setup().unwrap();
    assert_eq!(receipt.span(), block.span());
    assert_eq!(native.embeds.len(), 2);
    for embed in &native.embeds {
        let resolution = vue.file().expression(embed.node.unwrap()).unwrap();
        assert_eq!(resolution.scope(), Some(receipt.scope()));
        let table = resolution.table().unwrap();
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert!(!table.occurrences().is_empty());
        for reference in table.occurrences() {
            let binding = resolution.binding(reference.binding).unwrap();
            assert_eq!(vue.exposure(binding).unwrap().role(), VueScriptRole::Setup);
            let span = table.expression().authored_span(reference.span).unwrap();
            assert_eq!(
                source.get(span.start as usize..span.end as usize),
                Some(reference.name)
            );
            assert!(span.start > receipt.span().end);
        }
    }
    assert!(core::ptr::eq(
        native.component.block().root_source(),
        source
    ));
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn combined_roles_use_actual_indices_and_parent_scope_even_when_authored_reverse() {
    let arena = Allocator::default();
    let source = "<script setup>const local = ordinary;</script><script>export const ordinary = 1;</script><template><p>{{ordinary}} {{local}}</p></template>";
    let (ordinary, ordinary_block) =
        script(&arena, source, "export const ordinary = 1;", Lang::Js).unwrap();
    let (setup, setup_block) = script(&arena, source, "const local = ordinary;", Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .ordinary(
            ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 1).unwrap(),
        )
        .unwrap();
    producer
        .setup(ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 0).unwrap())
        .unwrap();
    let native = construct(
        &arena,
        source,
        "<p>{{ordinary}} {{local}}</p>",
        &mut producer,
    )
    .unwrap();
    assert!(native.is_supported());
    let vue = producer.finish().unwrap();
    assert_eq!(vue.ordinary().unwrap().unit().index(), 1);
    assert_eq!(vue.setup().unwrap().unit().index(), 0);
    assert_eq!(
        vue.file()
            .scopes()
            .get(vue.setup().unwrap().scope().index() as usize)
            .unwrap()
            .parent,
        Some(vue.ordinary().unwrap().scope())
    );
    let normal = vue
        .file()
        .lookup(vue.setup().unwrap().scope(), "ordinary", Namespace::Value)
        .unwrap();
    assert_eq!(
        vue.exposure(normal).unwrap().role(),
        VueScriptRole::Ordinary
    );
    let local = vue
        .file()
        .lookup(vue.setup().unwrap().scope(), "local", Namespace::Value)
        .unwrap();
    assert_eq!(vue.exposure(local).unwrap().role(), VueScriptRole::Setup);
    assert_eq!(vue.declarations().len(), 2);
    assert!(
        vue.declarations()
            .iter()
            .all(|declaration| declaration.initializer_span.is_some())
    );
}

#[test]
fn script_only_and_setup_only_without_template_finish_without_missing_template() {
    let arena = Allocator::default();
    let source = "const local = 1;";
    let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
    for setup in [false, true] {
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        let input = ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap();
        if setup {
            producer.setup(input).unwrap();
        } else {
            producer.ordinary(input).unwrap();
        }
        let vue = producer.finish().unwrap();
        let binding = vue.file().bindings().next().unwrap();
        assert_eq!(vue.exposure(binding).is_some(), setup);
        assert_eq!(vue.file().artifact().node_count(), 0);
    }
}

#[test]
fn setup_type_import_identity_is_retained_without_runtime_exposure() {
    let arena = Allocator::default();
    let source = "import type {T} from 'types'; import {value} from 'dep'; const local = value;";
    let (syntax, block) = script(&arena, source, source, Lang::Ts).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let vue = producer.finish().unwrap();
    let scope = vue.setup().unwrap().scope();
    assert!(
        vue.exposure(vue.file().lookup(scope, "T", Namespace::Type).unwrap())
            .is_none()
    );
    assert!(
        vue.exposure(vue.file().lookup(scope, "value", Namespace::Value).unwrap())
            .is_some()
    );
    assert!(
        vue.exposure(vue.file().lookup(scope, "local", Namespace::Value).unwrap())
            .is_some()
    );
    assert_eq!(vue.declarations().len(), 3);
}
