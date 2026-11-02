//! Every test uses the real original script parser, including mutation and Program admission.
use crate::drawer::{Drawer, DrawerOptions};
use crate::facts::components::homogeneous_index_value;

fn selected(script: &str, generic: Option<&str>) -> Option<vize_carton::String> {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup_with_generic(script, generic);
    let summary = drawer.finish();
    homogeneous_index_value(&summary, "map", summary.scopes.current_id())
        .map(vize_carton::String::from)
}

#[test]
fn same_actual_import_in_every_unannotated_const_entry_keeps_its_exact_type() {
    let script = "import Child from './Child.vue'; const map = { a: Child, b: Child };";
    assert_eq!(
        selected(script, Some("K extends 'a' | 'b'")).as_deref(),
        Some("Child")
    );
    assert_eq!(
        selected(script, None),
        None,
        "ordinary setup keeps the existing demand gate"
    );
}

#[test]
fn unsupported_or_changed_entry_identity_is_not_normalized() {
    for script in [
        "import Child from './Child.vue'; import Other from './Other.vue'; const map = { a: Child, b: Other };",
        "import Child from './Child.vue'; const map: Record<string, unknown> = { a: Child, b: Child };",
        "import Child from './Child.vue'; const map = ({ a: Child, b: Child } as const);",
        "import Child from './Child.vue'; let map = { a: Child, b: Child };",
        "import Child from './Child.vue'; const map = { ...{a: Child}, b: Child };",
        "import Child from './Child.vue'; const map = { __proto__: Child, b: Child };",
        "import Child from './Child.vue'; const key = 'a'; const map = { [key]: Child, b: Child };",
        "import Child from './Child.vue'; const map = { get a() { return Child }, b: Child };",
        "import type Child from './Child.vue'; const map = { a: Child, b: Child };",
        "import * as Child from './Child.vue'; const map = { a: Child, b: Child };",
        "import Child from './Child.vue'; const Alias = Child; const map = { a: Alias, b: Alias };",
        "import Child from './Child.vue'; const map = { a: Child, b: Child }; map.a = Child;",
        "import Child from './Child.vue'; const map = { a: Child, b: Child }; delete map.a;",
        "import Child from './Child.vue'; const map = { a: Child, b: Child }; Child.member = 1;",
        "import Child from './Child.vue'; const map = { a: Child, b: Child }; eval('map');",
    ] {
        assert_eq!(
            selected(script, Some("K extends 'a' | 'b'")),
            None,
            "{script}"
        );
    }
}

#[test]
fn local_shadow_and_multiple_programs_cannot_inherit_an_import_map_proof() {
    use crate::{ScopeBinding, ScopeKind};
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup_with_generic(
        "import Child from './Child.vue'; const map = { a: Child, b: Child };",
        Some("K extends 'a' | 'b'"),
    );
    let mut summary = drawer.finish();
    let local = summary.scopes.enter_scope(ScopeKind::VSlot);
    summary.scopes.add_binding(
        "map".into(),
        ScopeBinding::new(crate::BindingType::SetupConst, 900),
    );
    assert_eq!(homogeneous_index_value(&summary, "map", local), None);
    summary.source_program_count = 2;
    assert_eq!(
        homogeneous_index_value(&summary, "map", summary.scopes.current_id()),
        None
    );
}

#[test]
fn caller_owned_ast_does_not_acquire_original_parser_admission() {
    let script = "import Child from './Child.vue'; const map = { a: Child, b: Child };";
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, script, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup_program(&parsed.program, script, Some("K extends 'a' | 'b'"));
    let summary = drawer.finish();
    assert_eq!(
        homogeneous_index_value(&summary, "map", summary.scopes.current_id()),
        None
    );
}
