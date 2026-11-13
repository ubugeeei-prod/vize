//! #12943: unbound-method distinguishes lexical arrows from function expressions.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const COMPOSABLES: &str = r#"export function useArrow() {
  const copy = (): void => {};
  return { copy };
}

export function useAsyncArrow() {
  const save = async (): Promise<void> => {
    await Promise.resolve();
  };
  return { save };
}

export function useArrowProperty() {
  return { open: (): void => {} };
}

export function useFunctionDeclaration() {
  function load(): void {}
  return { load };
}

export function useFunctionExpression() {
  return { close: function (): void {} };
}

export function useMethodShorthand() {
  return {
    reset(): void {},
  };
}
"#;

const COMPONENT: &str = r#"<script setup lang="ts">
import {
  useArrow,
  useArrowProperty,
  useAsyncArrow,
  useFunctionDeclaration,
  useFunctionExpression,
  useMethodShorthand,
} from "./composables";

const { copy } = useArrow();
const { save } = useAsyncArrow();
const { open } = useArrowProperty();
const { load } = useFunctionDeclaration();
const { close } = useFunctionExpression();
const { reset } = useMethodShorthand();
</script>

<template>
  <button type="button" @click="copy">copy</button>
  <button type="button" @click="save">save</button>
  <button type="button" @click="open">open</button>
  <button type="button" @click="load">load</button>
  <button type="button" @click="close">close</button>
  <button type="button" @click="reset">reset</button>
</template>"#;

#[test]
fn composable_arrows_are_safe_but_function_expression_and_method_are_not() {
    if !corsa_available() {
        return;
    }
    let project = tempfile::tempdir().expect("project");
    let source_dir = project.path().join("src");
    std::fs::create_dir_all(&source_dir).expect("src");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022"
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#,
    )
    .expect("tsconfig");
    std::fs::write(source_dir.join("composables.ts"), COMPOSABLES).expect("composables");
    let component = source_dir.join("Destructure.vue");
    std::fs::write(&component, COMPONENT).expect("component");

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(
        &linter,
        COMPONENT,
        component.to_str().expect("UTF-8 component path"),
    );
    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "type/unbound-method")
        .map(|diagnostic| {
            COMPONENT
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    assert_eq!(diagnostics, ["close", "reset"], "{:#?}", result.diagnostics);
}
