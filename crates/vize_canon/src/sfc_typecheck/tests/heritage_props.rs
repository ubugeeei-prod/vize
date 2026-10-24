use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn imported_heritage_props_are_not_undefined_bindings() {
    // Regression: props inherited through an imported type
    // (`interface Props extends Pick<ImportedProps, ...>`) were flagged as
    // undefined references in template expressions by the editor while
    // `vize check` was clean — croquis alone cannot resolve imported types,
    // so the script compile context's resolved props must be merged in.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let project = std::env::temp_dir().join(format!(
        "vize-canon-heritage-props-{}-{nonce}",
        std::process::id()
    ));
    let src = project.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        src.join("types.ts"),
        "export interface RootProps { side?: 'left' | 'right'; resizable?: boolean }",
    )
    .unwrap();

    let component = src.join("Comp.vue");
    let source = r#"<script setup lang="ts">
import type { RootProps } from './types'

interface Props extends Pick<RootProps, 'side' | 'resizable'> {
  label?: string
}

const props = defineProps<Props>()
</script>

<template>
  <div v-if="side === 'left'">{{ label }} {{ resizable }}</div>
</template>
"#;

    let options = SfcTypeCheckOptions::new(component.to_string_lossy().as_ref());
    let result = type_check_sfc(source, &options);
    let undefined: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("undefined-binding"))
        .map(|d| d.message.clone())
        .collect();
    assert!(
        undefined.is_empty(),
        "inherited props flagged as undefined: {undefined:?}"
    );

    let _ = std::fs::remove_dir_all(project);
}
