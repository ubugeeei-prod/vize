use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_script_namespaces_are_module_scope_declarations() {
    check_pack(
        "plain-script-namespace-hoisting-3437",
        "fix_history_cases::plain_script_namespace_hoisting_3437::plain_script_namespaces_are_module_scope_declarations",
        "5435790bbb1f11b9d2c97e8e186000bcd67c58ea",
        Some(3437),
        &[
            "namespaces-used-as-values-and-types",
            "namespace-type-member-mismatch",
            "unexported-namespace-import",
            "legacy-module-keyword",
        ],
    );
}
