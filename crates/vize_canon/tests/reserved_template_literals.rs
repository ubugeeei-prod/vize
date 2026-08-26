use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::{String, cstr};

const EXTENDS_SOURCE: &str = r#"<script lang="ts">
import Base from './Base'

export default {
  extends: Base,
}
</script>

<template>
  <button
    :disabled="false"
    :aria-hidden="true"
    :data-empty="null"
    :data-state="falsey ? trueValue : nullish"
    @click="inheritedHandler(false)"
  />
</template>
"#;

const SETUP_SPREAD_SOURCE: &str = r#"<script lang="ts">
export default {
  setup() {
    const known = {}
    return { ...known }
  },
}
</script>

<template>
  <button
    :disabled="false"
    :aria-hidden="true"
    :data-empty="null"
    :data-state="falsey ? trueValue : nullish"
  />
</template>
"#;

#[test]
fn unresolved_options_extends_never_declares_template_literals() {
    let virtual_ts = generate_virtual_ts(EXTENDS_SOURCE);

    for reserved in ["false", "true", "null"] {
        assert!(
            !virtual_ts.contains(cstr!("const {reserved}: any").as_str()),
            "a template literal must not become an inherited binding `{reserved}`:\n{virtual_ts}"
        );
        assert!(
            !virtual_ts.contains(cstr!("void {reserved};").as_str()),
            "a template literal must not become a generated identifier `{reserved}`:\n{virtual_ts}"
        );
    }
}

#[test]
fn unresolved_options_extends_keeps_legal_inherited_names() {
    let virtual_ts = generate_virtual_ts(EXTENDS_SOURCE);

    for name in ["falsey", "trueValue", "nullish", "inheritedHandler"] {
        assert!(
            virtual_ts.contains(cstr!("const {name}: any = undefined as any;").as_str()),
            "legal inherited template binding `{name}` must remain available:\n{virtual_ts}"
        );
    }
}

#[test]
fn options_setup_spread_never_captures_template_literals() {
    let virtual_ts = generate_virtual_ts(SETUP_SPREAD_SOURCE);

    for literal in ["false", "true", "null"] {
        assert!(
            !virtual_ts.contains(cstr!("type __R_{literal} =").as_str())
                && !virtual_ts.contains(cstr!("var {literal}:").as_str()),
            "a template literal must not become a setup-return binding `{literal}`:\n{virtual_ts}"
        );
    }
    for name in ["falsey", "trueValue", "nullish"] {
        assert!(
            virtual_ts.contains(
                cstr!("type __R_{name} = __VizeOptionsSetupBinding<\"{name}\">;").as_str()
            ),
            "legal spread-backed template binding `{name}` must remain available:\n{virtual_ts}"
        );
    }
}

#[test]
fn generated_typescript_with_template_literals_is_parseable() {
    for source in [EXTENDS_SOURCE, SETUP_SPREAD_SOURCE] {
        let virtual_ts = generate_virtual_ts(source);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
                .parse();

        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "generated TypeScript must parse without literal declarations: {:#?}\n{virtual_ts}",
            parsed.diagnostics
        );
    }
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
