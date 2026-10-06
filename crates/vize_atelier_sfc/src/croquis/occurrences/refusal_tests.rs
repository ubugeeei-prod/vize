//! Adversarial completeness and source authority controls; no new diagnostics.
#![expect(
    clippy::disallowed_macros,
    reason = "authored complete source controls use std format"
)]

use super::super::analyze_sfc_descriptor_with_context;
use super::analyze_sfc_descriptor_with_occurrences;
use crate::{SfcParseOptions, croquis::SfcCroquisOptions, parse_sfc};
use vize_l0::Allocator;

fn refused(script: &str, expression: &str) {
    let source =
        format!("<script setup>{script}</script><template>{{{{ {expression} }}}}</template>");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
    let allocator = Allocator::default();
    let (root, _) =
        vize_armature::parse(&allocator, &descriptor.template.as_ref().unwrap().content);
    let options = SfcCroquisOptions::lint_demand();
    let ordinary = analyze_sfc_descriptor_with_context(&descriptor, Some(&root), options);
    let (captured, packet) =
        analyze_sfc_descriptor_with_occurrences(&descriptor, Some(&root), options);
    assert!(
        packet.is_none(),
        "incomplete facts were admitted for {source}"
    );
    assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
    assert_eq!(
        serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
        serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap()
    );
}

#[test]
fn unvisited_top_level_runtime_forms_refuse_instead_of_claiming_no_use() {
    for script in [
        "const id=1; if(true){consume(id)}",
        "const id=1; for(let i=0;i<1;i++){consume(id)}",
        "const id=1; try{consume(id)}catch{}",
        "const id=1; class C { m(){return id} }",
        "const id=1; const {[id]: named} = object",
        "const id=1; const f = (value = id) => value",
        "const id=1; eval('id')",
        "const id=1; enum E { X=id }",
        "const id=1; function f(){ throw id }",
        "const id=1; onMounted(function id(){ return id })",
    ] {
        refused(script, "id");
    }
}

#[test]
fn unsupported_hoisted_var_and_named_function_owners_never_become_outer_reads() {
    for script in [
        "const id=1; function f(){ if(true){var id=2} return id }",
        "const id=1; const f = function id(){ return id }",
        "const id=1; function f(){ for(var id of [1]){consume(id)} return id }",
    ] {
        refused(script, "id");
    }
}

#[test]
fn unmodeled_value_type_queries_refuse_without_changing_ordinary_analysis() {
    for script in [
        "const id=1; type Value=typeof id",
        "const id=1; interface Value {value:typeof id}",
        "const id=1; type Value<T extends typeof id = typeof id> = T",
        "const id=1; const value:typeof id=id",
        "const id=1; function f(value:typeof id){return value}",
        "const id=1; function f(this:typeof id){return id}",
        "const id=1; function f():typeof id{return id}",
        "const id=1; const f=(value:typeof id):typeof id=>value",
        "const id=1; function f<T extends typeof id>(){return id}",
        "const id=1; const value=id as typeof id",
        "const id=1; const value=id satisfies typeof id",
        "const id=1; const value=ref(1) as typeof id",
        "const id=1; consume<typeof id>(id)",
        "const id=1; onMounted(():typeof id=>id)",
        "const id=1; function f(){const value:typeof id=id;return value}",
    ] {
        refused(script, "id");
    }
}

#[test]
fn unmodeled_template_value_type_queries_refuse_without_losing_ordinary_names() {
    for expression in [
        "id as typeof id",
        "id satisfies typeof id",
        "(():typeof id=>id)()",
        "((value:typeof id)=>value)(id)",
        "consume<typeof id>(id)",
    ] {
        refused("const id=1", expression);
    }
}

#[test]
fn setup_generic_metadata_refuses_without_an_original_script_identifier_ast() {
    let source = "<script setup lang=\"ts\" generic=\"T extends typeof id\">const id=1</script><template>{{ id }}</template>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let allocator = Allocator::default();
    let (root, errors) =
        vize_armature::parse(&allocator, &descriptor.template.as_ref().unwrap().content);
    assert!(errors.is_empty());
    let options = SfcCroquisOptions::lint_demand();
    let ordinary = analyze_sfc_descriptor_with_context(&descriptor, Some(&root), options);
    let (captured, packet) =
        analyze_sfc_descriptor_with_occurrences(&descriptor, Some(&root), options);
    assert!(packet.is_none());
    assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
    assert_eq!(
        serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
        serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap(),
    );
}

#[test]
fn invalid_or_unclosed_template_expression_relations_refuse_as_a_whole() {
    for expression in [
        "id +",
        "id.",
        "f(...[id])",
        "new f(...[id])",
        "eval('id')",
        "(() => { for(const value of [id]){ consume(value) } })()",
    ] {
        refused("const id=1", expression);
    }
}

#[test]
fn template_capture_requires_the_actual_source_and_binding_read_demand() {
    let source = "<script setup>const id=1</script><template>{{ id }}</template>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::parse(&allocator, "{{ other }}");
    assert!(errors.is_empty());
    let (_, packet) = analyze_sfc_descriptor_with_occurrences(
        &descriptor,
        Some(&root),
        SfcCroquisOptions::lint_demand(),
    );
    assert!(packet.is_none());
    let (original, errors) =
        vize_armature::parse(&allocator, &descriptor.template.as_ref().unwrap().content);
    assert!(errors.is_empty());
    for options in [
        SfcCroquisOptions::for_compile(),
        SfcCroquisOptions::lint_demand().with_derived_template(),
    ] {
        let ordinary = analyze_sfc_descriptor_with_context(&descriptor, Some(&original), options);
        let (captured, packet) =
            analyze_sfc_descriptor_with_occurrences(&descriptor, Some(&original), options);
        assert!(packet.is_none());
        assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
    }
}
