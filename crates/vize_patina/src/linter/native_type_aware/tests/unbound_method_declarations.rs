//! #12975: unbound-method follows the property's declaration form.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use std::path::Path;

const COMPOSABLES: &str = r#"export function useShorthand() {
  const close = function (): void {};
  return { close };
}

export function useTyped(): { load: () => void; save(): void } {
  return { load: () => {}, save() {} };
}

export class Panel {
  toggle = function (): void {};
}

export function usePanel(): Panel {
  return new Panel();
}
"#;

const COMPONENT: &str = r#"<script setup lang="ts">
import { usePanel, useShorthand, useTyped } from "./composables";

interface Router {
  push(to: string): Promise<void>;
  replace: (to: string) => void;
}
declare function useRouter(): Router;
declare const controller: AbortController;
declare const when: Date;

const seen = new Set<string>();
const kept = ["a"].filter(seen.has);
const { close } = useShorthand();
const { load } = useTyped();
const { save } = useTyped();
const { toggle } = usePanel();
const { push, replace } = useRouter();
const stamp = when.getTime;
setTimeout(controller.abort, 10);
</script>

<template>
  <p>{{ kept.length }} {{ close }} {{ load }} {{ save }} {{ toggle }} {{ push }} {{ replace }} {{ stamp }}</p>
</template>"#;

#[test]
fn declaration_forms_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("unbound-method-declarations-12975-")
        .tempdir_in(workspace)
        .expect("project");
    let source_dir = project.path().join("src");
    std::fs::create_dir_all(&source_dir).expect("src");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "lib": ["ES2022", "DOM"],
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
    let component = source_dir.join("Unbound.vue");
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
    assert_eq!(
        diagnostics,
        [
            "seen.has",
            "save",
            "toggle",
            "push",
            "when.getTime",
            "controller.abort"
        ],
        "{:#?}",
        result.diagnostics
    );
}
