use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

macro_rules! fixture {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../../tests/_fixtures/differential/typechecker/vapor-sfc-component-type/",
            $file
        ))
    };
}

/// vue-tsc reads a setup `vapor` attribute as a bare `generic`, so a Vapor
/// SFC exports a function that `createVaporApp` and `VaporComponent` accept,
/// with or without props and emits, and that a VDOM template still checks.
/// A required `defineSlots` slot and a VDOM SFC stay rejected, as vue-tsc
/// 3.3.14 rejects them.
#[test]
fn vapor_sfc_default_export_is_a_vapor_component() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "vapor-sfc-component-type",
        &[
            ("tsconfig.json", fixture!("tsconfig.json.txt")),
            ("src/Plain.vue", fixture!("Plain.vue.txt")),
            ("src/Typed.vue", fixture!("Typed.vue.txt")),
            ("src/Slotted.vue", fixture!("Slotted.vue.txt")),
            ("src/Vdom.vue", fixture!("Vdom.vue.txt")),
            ("src/Parent.vue", fixture!("Parent.vue.txt")),
            ("src/main.ts", fixture!("main.ts.txt")),
        ],
    );
    let diagnostics = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    let diagnostics = diagnostics.expect("type-check Vapor SFC project");
    // vue-tsc 3.3.14 reports these codes at these positions. The argument
    // types print differently, so the oracle is the position and the reason
    // TypeScript gives last, which matches vue-tsc's word for word.
    let diagnostics: Vec<_> = diagnostics
        .iter()
        .map(|(file, code, message)| {
            let (position, _) = message.split_once(":error ").unwrap_or_default();
            let reason = message.lines().last().unwrap_or_default();
            (file.as_str(), *code, position, reason)
        })
        .collect();
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/Parent.vue",
                Some(2322),
                "15:11",
                "15:11:error Type 'number' is not assignable to type 'string'.",
            ),
            (
                "src/main.ts",
                Some(2345),
                "11:16",
                "Property 'default' is missing in type 'StaticSlots' but required in type '$Slots'.",
            ),
            (
                "src/main.ts",
                Some(2345),
                "12:16",
                "Type '$ComponentInstance' is missing the following properties from type 'VaporComponentInstance<{} & ReservedProps & AllowedComponentProps & ComponentCustomProps, {}, StaticSlots, Record<string, any>, Block, {}>': vapor, uid, type, root, and 29 more.",
            ),
        ]
    );
}
