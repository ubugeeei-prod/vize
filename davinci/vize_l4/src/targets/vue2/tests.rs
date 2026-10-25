use super::{Vue2Access, Vue2AccessError, Vue2ErrorKind, emit};
use crate::{
    expr::{
        ResolvedExpressions,
        vue::{Access, Binding},
    },
    write::{NoLinks, Recorded},
};
use alloc::vec::Vec;
use vize_l0::{Allocator, String};
use vize_l1::dialect::vue2::surface::parse_component;
use vize_l1_to_l2::native_vue2::lower_vue2_component;
use vize_l2::{
    artifact::Artifact,
    expr::ExprRef,
    resolution::{BindingId, BindingLookup, ResolutionTable, resolve_expression},
};
use vize_l3::decision::{
    build_decisions, build_dom_decisions, dom::ContextOnly, policy::TargetPolicy,
};

const CONTEXT: &[&str] = &[
    "count",
    "label",
    "Math",
    "_vm",
    "value",
    "render",
    "staticRenderFns",
    "_vm1",
    "render1",
    "staticRenderFns1",
];
struct Lookup;
impl BindingLookup for Lookup {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        CONTEXT
            .iter()
            .position(|known| *known == name)
            .map(|index| BindingId::new(index as u32))
    }
}
fn tables<'a>(artifact: &Artifact<'a>) -> Vec<ResolutionTable<'a>> {
    let mut tables = Vec::new();
    artifact
        .visit_nodes(&mut |_, node| {
            node.for_each_expression(&mut |expression| {
                let ExprRef::Js(expression) = expression else {
                    panic!("original JS")
                };
                tables.push(resolve_expression(expression, &Lookup).unwrap());
            });
        })
        .unwrap();
    tables.sort_by_key(|row| (row.expression().span.start, row.expression().span.end));
    tables
}
fn bindings() -> Vec<Binding<'static>> {
    CONTEXT
        .iter()
        .enumerate()
        .map(|(index, name)| Binding {
            id: BindingId::new(index as u32),
            access: if *name == "Math" {
                Access::Global
            } else {
                Access::Context
            },
        })
        .collect()
}
fn module(source: &str) -> String {
    module_with_bindings(source, &bindings())
}

fn module_with_bindings(source: &str, bindings: &[Binding<'_>]) -> String {
    let allocator = Allocator::default();
    let original = parse_component(&allocator, source)
        .unwrap()
        .into_expression_pool();
    let observation = lower_vue2_component(&allocator, original);
    let artifact = observation.admitted().unwrap();
    let tables = tables(artifact);
    let ids = bindings
        .iter()
        .map(|binding| binding.id)
        .collect::<Vec<_>>();
    let policy = ContextOnly::new(&tables, &ids);
    let analysis = build_dom_decisions(artifact, &policy).unwrap();
    let access = Vue2Access::checked(bindings).unwrap();
    emit::<NoLinks>(
        &analysis,
        ResolvedExpressions::checked(&tables).unwrap(),
        &access,
    )
    .unwrap()
    .finish()
    .text
}

#[test]
fn actual_retained_vue2_source_uses_instance_helpers_and_complete_vue2_exports() {
    assert_eq!(
        module("<div id='a'>Hello {{ count }}</div>"),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", {attrs: {\"id\": \"a\"}}, [_vm._v(\"Hello \" + _vm._s(_vm.count))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
}

#[test]
fn empty_authored_class_keeps_data_without_a_static_class_field() {
    assert_eq!(
        module("<div class=''>{{ count }}</div>"),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", {}, [_vm._v(_vm._s(_vm.count))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
}

#[test]
fn explicit_context_global_and_shorthand_access_preserve_authored_meaning() {
    assert_eq!(
        module("<div>{{ Math.round(count) }}</div>"),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(Math.round(_vm.count)))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
    assert_eq!(
        module("<div>{{ ({count}) }}</div>"),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(({count: _vm.count})))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
    assert_eq!(
        module("<div>{{ _vm }}</div>"),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(_vm._vm))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
}

#[test]
fn missing_real_resolution_or_runtime_binding_is_a_typed_refusal() {
    let arena = Allocator::default();
    let original = parse_component(&arena, "<div>{{ count }}</div>")
        .unwrap()
        .into_expression_pool();
    let observation = lower_vue2_component(&arena, original);
    let artifact = observation.admitted().unwrap();
    let tables = tables(artifact);
    let ids = [BindingId::new(0)];
    let analysis = build_dom_decisions(artifact, &ContextOnly::new(&tables, &ids)).unwrap();
    let access = Vue2Access::checked(&bindings()).unwrap();
    let error = emit::<NoLinks>(
        &analysis,
        ResolvedExpressions::checked(&[]).unwrap(),
        &access,
    )
    .unwrap_err();
    assert_eq!(
        error.kind,
        Vue2ErrorKind::Expression(crate::expr::EmitError {
            span: tables.first().unwrap().expression().span,
            kind: crate::expr::EmitErrorKind::MissingResolution
        })
    );
    let missing = Vue2Access::checked(&[]).unwrap();
    let error = emit::<NoLinks>(
        &analysis,
        ResolvedExpressions::checked(&tables).unwrap(),
        &missing,
    )
    .unwrap_err();
    assert_eq!(
        error.kind,
        Vue2ErrorKind::Expression(crate::expr::EmitError {
            span: tables.first().unwrap().expression().span,
            kind: crate::expr::EmitErrorKind::Access(crate::expr::AccessError::MissingBinding)
        })
    );
}

#[test]
fn target_refuses_fragment_roots_and_unrelated_target_decisions() {
    let arena = Allocator::default();
    let original = parse_component(&arena, "<div>a</div><p>b</p>")
        .unwrap()
        .into_expression_pool();
    let observation = lower_vue2_component(&arena, original);
    let artifact = observation.admitted().unwrap();
    let access = Vue2Access::checked(&[]).unwrap();
    let analysis = build_decisions(artifact, TargetPolicy::Dom).unwrap();
    assert_eq!(
        emit::<NoLinks>(
            &analysis,
            ResolvedExpressions::checked(&[]).unwrap(),
            &access
        )
        .unwrap_err()
        .kind,
        Vue2ErrorKind::Root
    );
    let analysis = build_decisions(artifact, TargetPolicy::Ssr).unwrap();
    assert_eq!(
        emit::<NoLinks>(
            &analysis,
            ResolvedExpressions::checked(&[]).unwrap(),
            &access
        )
        .unwrap_err()
        .kind,
        Vue2ErrorKind::WrongPolicy
    );
}

#[test]
fn unsupported_setup_identities_never_receive_an_instance_fallback() {
    assert_eq!(
        Vue2Access::checked(&[Binding {
            id: BindingId::new(0),
            access: Access::SetupRef
        }])
        .err(),
        Some(Vue2AccessError::UnsupportedBinding)
    );
    assert_eq!(
        Vue2Access::checked(&[
            Binding {
                id: BindingId::new(0),
                access: Access::Context
            },
            Binding {
                id: BindingId::new(0),
                access: Access::Context
            }
        ])
        .err(),
        Some(Vue2AccessError::UnorderedOrDuplicateIdentity)
    );
}

#[test]
fn named_expression_links_still_point_to_the_original_utf8_source() {
    let arena = Allocator::default();
    let source = "<div>雪😀 {{ count }}</div>";
    let original = parse_component(&arena, source)
        .unwrap()
        .into_expression_pool();
    let observation = lower_vue2_component(&arena, original);
    let artifact = observation.admitted().unwrap();
    let tables = tables(artifact);
    let bindings = bindings();
    let ids = bindings
        .iter()
        .map(|binding| binding.id)
        .collect::<Vec<_>>();
    let analysis = build_dom_decisions(artifact, &ContextOnly::new(&tables, &ids)).unwrap();
    let access = Vue2Access::checked(&bindings).unwrap();
    let emitted = emit::<Recorded>(
        &analysis,
        ResolvedExpressions::checked(&tables).unwrap(),
        &access,
    )
    .unwrap()
    .finish();
    assert_eq!(emitted.text, module(source));
    let link = emitted
        .links
        .links()
        .iter()
        .find(|link| link.name.as_deref() == Some("count"))
        .unwrap();
    assert_eq!(
        source.get(link.authored.start as usize..link.authored.end as usize),
        Some("count")
    );
    assert_eq!(
        emitted
            .text
            .get(link.generated.start as usize..link.generated.end as usize),
        Some("_vm.count")
    );
}

fn raw_bindings(role: Access<'static>) -> Vec<Binding<'static>> {
    bindings()
        .into_iter()
        .map(|mut binding| {
            if matches!(binding.id.index(), 3 | 5..=9) {
                binding.access = role;
            }
            binding
        })
        .collect()
}

#[test]
fn verbatim_receiver_names_include_semantic_escaped_identifiers() {
    for role in [Access::Global, Access::Local] {
        for (expression, emitted) in [("_vm", "_vm"), (r"_\u0076m", r"_\u0076m")] {
            let source = vize_l0::cstr!("<div>{{{{ {expression} }}}}</div>");
            assert_eq!(
                module_with_bindings(&source, &raw_bindings(role)),
                vize_l0::cstr!(
                    "export function render() {{\n  const _vm1 = this;\n  return _vm1._c(\"div\", void 0, [_vm1._v(_vm1._s({emitted}))]);\n}}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
                )
            );
        }
    }
}

#[test]
fn verbatim_module_names_use_private_declarations_with_stable_public_exports() {
    for role in [Access::Global, Access::Local] {
        assert_eq!(
            module_with_bindings("<div>{{ render }}</div>", &raw_bindings(role)),
            "function render1() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(render))]);\n}\nrender1._withStripped = true;\nexport { render1 as render };\nexport const staticRenderFns = [];\n"
        );
        assert_eq!(
            module_with_bindings("<div>{{ staticRenderFns }}</div>", &raw_bindings(role)),
            "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(staticRenderFns))]);\n}\nrender._withStripped = true;\nconst staticRenderFns1 = [];\nexport { staticRenderFns1 as staticRenderFns };\n"
        );
    }
}

#[test]
fn fresh_names_reserve_only_used_verbatim_names_and_choose_smallest_free_suffix() {
    let source = "<div>{{ [_vm,_vm1,render,render1,staticRenderFns,staticRenderFns1] }}</div>";
    for role in [Access::Global, Access::Local] {
        assert_eq!(
            module_with_bindings(source, &raw_bindings(role)),
            "function render2() {\n  const _vm2 = this;\n  return _vm2._c(\"div\", void 0, [_vm2._v(_vm2._s([_vm,_vm1,render,render1,staticRenderFns,staticRenderFns1]))]);\n}\nrender2._withStripped = true;\nexport { render2 as render };\nconst staticRenderFns2 = [];\nexport { staticRenderFns2 as staticRenderFns };\n"
        );
    }
    let mut mixed = bindings();
    for binding in &mut mixed {
        if matches!(binding.id.index(), 3 | 5 | 6) {
            binding.access = Access::Global;
        }
    }
    assert_eq!(
        module_with_bindings(source, &mixed),
        "function render1() {\n  const _vm1 = this;\n  return _vm1._c(\"div\", void 0, [_vm1._v(_vm1._s([_vm,_vm1._vm1,render,_vm1.render1,staticRenderFns,_vm1.staticRenderFns1]))]);\n}\nrender1._withStripped = true;\nexport { render1 as render };\nconst staticRenderFns1 = [];\nexport { staticRenderFns1 as staticRenderFns };\n"
    );
    assert_eq!(
        module_with_bindings(
            "<div>{{ ({_vm:count,render:label,staticRenderFns:value}) }}</div>",
            &raw_bindings(Access::Global)
        ),
        "export function render() {\n  const _vm = this;\n  return _vm._c(\"div\", void 0, [_vm._v(_vm._s(({_vm:_vm.count,render:_vm.label,staticRenderFns:_vm.value})))]);\n}\nrender._withStripped = true;\nexport const staticRenderFns = [];\n"
    );
}
