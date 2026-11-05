#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

const COMPONENTS: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/undeclared-listener-11913/comps.ts.txt"
);

#[test]
fn passing_a_prop_does_not_turn_an_undeclared_listener_into_a_zero_arity_listener() {
    let source = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/undeclared-listener-11913/valid.vue.txt"
    );
    assert_eq!(
        project::check(&[("src/comps.ts", COMPONENTS), ("src/use.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn declared_payloads_and_declared_zero_arity_listeners_stay_checked() {
    let source = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/undeclared-listener-11913/invalid.vue.txt"
    );
    let rows = project::check(&[("src/comps.ts", COMPONENTS), ("src/use.vue", source)]);
    assert_eq!(rows.len(), 2, "{rows:#?}");
    assert!(
        rows[0].starts_with("src/use.vue(7,28): error TS2322:"),
        "{rows:#?}"
    );
    assert!(
        rows[1].starts_with("src/use.vue(8,30): error TS2322:"),
        "{rows:#?}"
    );
    assert!(rows[0].contains("MouseEvent"), "{rows:#?}");
    assert!(rows[1].contains("() => any"), "{rows:#?}");
}
