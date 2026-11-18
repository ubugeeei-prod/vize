use crate::support::fix_history_runner::check_pack;

#[test]
fn hoisted_plain_script_functions_keep_types_and_directives() {
    check_pack(
        "plain-script-function-export-identity-6279",
        "fix_history_cases::plain_script_function_export_identity_6279::hoisted_plain_script_functions_keep_types_and_directives",
        "e112d3a7fd0c8990869ec56c1b07d124522f5465",
        Some(6279),
        &[
            "function-exports-lf-consumer",
            "function-exports-crlf-consumer",
            "function-exports-invalid-generic-and-overload-calls",
            "hoisted-function-without-directive-reports-return",
        ],
    );
}
