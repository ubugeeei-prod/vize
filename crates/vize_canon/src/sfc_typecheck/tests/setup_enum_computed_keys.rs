use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn computed_keys_stay_with_setup_enum() {
    let source = r#"<script setup lang="ts">
enum Keys {
  A = "A",
  B = "B",
}
type Foo = {
  [Keys.A]: number | null;
  [Keys.B]: number;
};
declare const foo: Foo;
const value = foo[Keys.A] ?? 0;
</script>

<template>
  <p>{{ value }}</p>
</template>"#;
    let options = SfcTypeCheckOptions::new("Keyed.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);

    assert!(
        result.diagnostics.is_empty(),
        "setup enum and its dependent type alias must resolve together: {:#?}",
        result.diagnostics
    );

    let virtual_ts = result.virtual_ts.expect("virtual ts produced");
    let setup_start = virtual_ts
        .find("function __setup()")
        .expect("__setup function emitted");
    let enum_pos = virtual_ts.find("enum Keys").expect("enum emitted");
    let type_pos = virtual_ts.find("type Foo").expect("type alias emitted");
    assert!(
        enum_pos > setup_start && type_pos > setup_start,
        "the enum and dependent alias must remain in setup scope:\n{virtual_ts}"
    );
}
