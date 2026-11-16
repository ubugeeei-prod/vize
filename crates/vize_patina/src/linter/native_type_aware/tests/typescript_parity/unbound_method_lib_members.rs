//! #13081, #13082, #13083: unbound-method decides from where the receiver and
//! the member are declared, not from a name table or from the whole symbol.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde_json::json;
use std::path::Path;
use vize_l0::config::TypeAwareRuleOptions;

const RULE: &str = "type/unbound-method";

/// Global augmentations of the built-ins the rule exempts by type (#13082).
const AUGMENT: &str = r#"declare global {
  interface Math {}
  interface Atomics {}
  interface JSON {
    reviveAll(text: string): unknown;
  }
  interface ObjectConstructor {
    readonly marker: string;
  }
  interface Array<T> {
    last(): T | undefined;
  }
  interface Window {
    track(event: string): void;
  }
}

export const augmented = true;
"#;

const COMPONENT: &str = r#"<script setup lang="ts">
import { augmented } from "./augment";

declare const math: Math;
declare const json: JSON;
declare const numbers: NumberConstructor;
declare const ids: string[];
declare const view: Window & typeof globalThis;

const viaType = math.max;
const viaWindow = window.Math.max;
const keys = window.Object.keys;
const byName = Math.max;
const parse = numbers.parseFloat;
const now = window.Date.now;
const push = ids.push;
const kept = ids.filter(ids.includes);
const parseJson = json.parse;
const last = ids.last;
const revive = json.reviveAll;
const half = math.f16round;
const pause = Atomics.pause;
const escape = RegExp.escape;
const pattern = ids.map(RegExp.escape).join("|");
const wrapped = ids.map((term) => RegExp.escape(term)).join("|");
const nested = window.RegExp.escape;
const timer = window.setTimeout;
const { matchMedia } = window;
const frame = self.requestAnimationFrame;
const viaView = view.setTimeout;
const viaDocument = document.defaultView?.setTimeout;
const storage = window.localStorage.getItem;
const parentTimer = parent.setTimeout;
const computed = window["setTimeout"];
const track = window.track;
</script>

<template>
  <p>
    {{ augmented }} {{ viaType }} {{ viaWindow }} {{ keys }} {{ byName }} {{ parse }} {{ now }}
    {{ push }} {{ kept }} {{ parseJson }} {{ last }} {{ revive }} {{ half }} {{ pause }}
    {{ escape }} {{ pattern }} {{ wrapped }} {{ nested }} {{ timer }} {{ matchMedia }}
    {{ frame }} {{ viaView }} {{ viaDocument }} {{ storage }} {{ parentTimer }}
    {{ computed }} {{ track }}
  </p>
</template>"#;

#[test]
fn library_members_follow_their_declarations() {
    if !corsa_available() {
        return;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("unbound-method-lib-members-13081-")
        .tempdir_in(workspace)
        .expect("project");
    let source_dir = project.path().join("src");
    std::fs::create_dir_all(&source_dir).expect("src");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "lib": ["ESNext", "DOM"],
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
    std::fs::write(source_dir.join("augment.ts"), AUGMENT).expect("augmentation");
    let component = source_dir.join("Members.vue");
    std::fs::write(&component, COMPONENT).expect("component");

    let path = component.to_str().expect("UTF-8 component path");
    let reported = |check_global_object_methods: bool| {
        let options: TypeAwareRuleOptions = serde_json::from_value(json!({
            (RULE): { "checkGlobalObjectMethods": check_global_object_methods }
        }))
        .expect("type-aware options");
        let linter = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_type_aware_rule_options(options);
        let result = lint_sfc_with_corsa(&linter, COMPONENT, path);
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| {
                COMPONENT
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
    };
    // Members the augmentation adds are not the library's (#13082), and a
    // `Storage` or another frame's `Window` is not the global object.
    let always = [
        "ids.last",
        "json.reviveAll",
        "window.localStorage.getItem",
        "parent.setTimeout",
        "window.track",
    ];
    assert_eq!(reported(false), always);
    // typescript-eslint also reports the global object's platform methods,
    // which `Window` declares as methods (#13083).
    assert_eq!(
        reported(true),
        [
            "ids.last",
            "json.reviveAll",
            "window.setTimeout",
            "matchMedia",
            "self.requestAnimationFrame",
            "view.setTimeout",
            "document.defaultView?.setTimeout",
            "window.localStorage.getItem",
            "parent.setTimeout",
            "window[\"setTimeout\"]",
            "window.track",
        ]
    );
}
