//! Qualified imported macro references must fail rather than lose props.

use super::{compile_source, project};

#[test]
fn unresolved_namespace_import_in_a_macro_is_not_silently_erased() {
    let project = project();
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type * as Types from '@missing/Props';defineProps<Types.Foo>();</script>").expect_err("unresolved qualified type");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert_eq!(
        error.message,
        "Cannot resolve imported macro type `Types` from `@missing/Props`."
    );
}
