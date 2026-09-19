use super::emit_lib_reference_directives;
use vize_carton::String;

#[test]
fn virtual_ts_lib_references_are_pluggable() {
    let mut output = String::default();
    emit_lib_reference_directives(&mut output, &["es2021", "webworker", "bad\" />"]);

    assert_eq!(
        output.as_str(),
        "/// <reference lib=\"es2021\" />\n/// <reference lib=\"webworker\" />\n"
    );
}
