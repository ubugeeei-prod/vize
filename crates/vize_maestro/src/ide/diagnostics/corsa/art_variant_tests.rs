//! Every authored `<variant>` of a `.art.vue` file must reach the checker in
//! its own typed document over the authored script context (#4015).

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::VirtualTsOptions;

use super::super::{DiagnosticService, VirtualTsResult};
use super::virtual_ts_art::art_variant_virtual_name;

const TWO_VARIANTS: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', 2)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 'two')" />
  </variant>
</art>
"#;

fn art_uri() -> Url {
    Url::parse("file:///project/src/Button.art.vue").expect("parse uri")
}

fn generate(content: &str) -> Vec<(usize, VirtualTsResult)> {
    DiagnosticService::generate_virtual_ts_for_art_with_dependencies(
        &art_uri(),
        content,
        &VirtualTsOptions::default(),
    )
    .expect("art virtual TypeScript")
    .variants
    .into_iter()
    .map(|variant| (variant.variant_index, variant.virtual_result))
    .collect()
}

/// Authored byte range of `needle`, which must occur exactly once.
fn unique_range(content: &str, needle: &str) -> std::ops::Range<usize> {
    let start = content.find(needle).expect("needle present");
    assert_eq!(
        content[start + 1..].find(needle),
        None,
        "needle {needle:?} must be unique",
    );
    start..start + needle.len()
}

fn mapped_source_ranges(result: &VirtualTsResult) -> Vec<std::ops::Range<usize>> {
    result
        .source_mappings
        .iter()
        .map(|mapping| mapping.src_range.clone())
        .collect()
}

/// Whether any generated mapping reaches into the authored range at all.
fn covers(result: &VirtualTsResult, range: &std::ops::Range<usize>) -> bool {
    mapped_source_ranges(result)
        .iter()
        .any(|mapped| mapped.start < range.end && range.start < mapped.end)
}

#[test]
fn every_art_variant_is_generated_into_its_own_typed_document() {
    let generated = generate(TWO_VARIANTS);

    assert_eq!(
        generated
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>(),
        vec![0, 1],
    );
}

#[test]
fn each_art_variant_document_maps_only_its_own_authored_expressions() {
    let generated = generate(TWO_VARIANTS);
    let primary = unique_range(TWO_VARIANTS, "format('primary', 2)");
    let secondary = unique_range(TWO_VARIANTS, "format('secondary', 'two')");

    assert!(covers(&generated[0].1, &primary));
    assert!(!covers(&generated[0].1, &secondary));
    assert!(covers(&generated[1].1, &secondary));
    assert!(!covers(&generated[1].1, &primary));
}

#[test]
fn every_art_variant_document_declares_the_authored_script_callable() {
    for (_, result) in generate(TWO_VARIANTS) {
        assert!(
            result
                .code
                .contains("function format(value: string, precision: number): string"),
            "variant document lost the authored callable:\n{}",
            result.code,
        );
    }
}

#[test]
fn art_variant_documents_take_distinct_identities_in_the_authored_directory() {
    let uri = art_uri();

    assert_eq!(
        art_variant_virtual_name(&uri, 0).as_str(),
        "/project/src/Button.art.vue.ts",
    );
    assert_eq!(
        art_variant_virtual_name(&uri, 1).as_str(),
        "/project/src/Button.art.vue.art_variant_1.ts",
    );
}

/// Two variants that reuse one alias name must not see each other's binding:
/// the isolated scopes are what make a `v-for` alias variant-local.
#[test]
fn same_name_bindings_do_not_leak_between_isolated_variants() {
    const SHADOWED: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Numbers" default>
    <Button v-for="entry in [1, 2]" :label="entry" />
  </variant>
  <variant name="Strings">
    <Button v-for="entry in ['a', 'b']" :label="entry" />
  </variant>
</art>
"#;

    let generated = generate(SHADOWED);
    let numbers = unique_range(SHADOWED, "entry in [1, 2]");
    let strings = unique_range(SHADOWED, "entry in ['a', 'b']");

    assert_eq!(generated.len(), 2);
    assert!(covers(&generated[0].1, &numbers));
    assert!(!covers(&generated[0].1, &strings));
    assert!(covers(&generated[1].1, &strings));
    assert!(!covers(&generated[1].1, &numbers));
}

/// `isolate="false"` shares one setup run across variants. The typed context is
/// the same either way, so every variant still sees the authored bindings.
#[test]
fn shared_setup_variants_keep_the_authored_bindings_in_every_document() {
    const SHARED: &str = r#"<script setup lang="ts" isolate="false">
defineArt("./Button.vue", { title: "Button" });

const sharedLabel: string = "shared";
</script>

<art>
  <variant name="Primary" default>
    <Button :label="sharedLabel" />
  </variant>
  <variant name="Secondary">
    <Button :label="sharedLabel.toUpperCase()" />
  </variant>
</art>
"#;

    let generated = generate(SHARED);

    assert_eq!(generated.len(), 2);
    for (_, result) in &generated {
        assert!(
            result
                .code
                .contains("const sharedLabel: string = \"shared\""),
            "shared setup binding missing:\n{}",
            result.code,
        );
    }
}

/// A classic `<script>` art file has no `<script setup>` decomposition, but its
/// declarations still have to reach every variant document.
#[test]
fn classic_script_art_files_type_every_variant() {
    const CLASSIC: &str = r#"<script lang="ts">
export function shout(value: string): string {
  return value.toUpperCase();
}
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="shout('primary')" />
  </variant>
  <variant name="Secondary">
    <Button :label="shout('secondary')" />
  </variant>
</art>
"#;

    let generated = generate(CLASSIC);

    assert_eq!(generated.len(), 2);
    for (_, result) in &generated {
        assert!(
            result
                .code
                .contains("function shout(value: string): string"),
            "classic script body missing:\n{}",
            result.code,
        );
    }
    assert!(covers(
        &generated[1].1,
        &unique_range(CLASSIC, "shout('secondary')"),
    ));
}

/// `defineArt(...)` is dropped from the typed script context. Concatenating the
/// surviving chunks would shorten the script, so everything declared after it
/// would map to an earlier authored line.
#[test]
fn dropped_setup_statements_keep_later_script_offsets_authored() {
    const SCRIPT: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

const broken: number = "not a number";
</script>

<art>
  <variant name="Primary" default>
    <Button :label="broken" />
  </variant>
</art>
"#;

    let generated = generate(SCRIPT);
    let declaration = unique_range(SCRIPT, "broken: number");

    assert_eq!(generated.len(), 1);
    assert!(covers(&generated[0].1, &declaration));
}

/// CRLF documents must keep byte-exact authored ranges: a `\r` dropped from the
/// offset arithmetic shifts every later variant mapping by one byte per line.
#[test]
fn crlf_art_files_map_later_variants_to_exact_authored_bytes() {
    let crlf = TWO_VARIANTS.replace('\n', "\r\n");
    let generated = generate(&crlf);
    let secondary = unique_range(&crlf, "format('secondary', 'two')");

    assert_eq!(generated.len(), 2);
    assert!(covers(&generated[1].1, &secondary));
    assert!(!covers(
        &generated[1].1,
        &unique_range(&crlf, "format('primary', 2)"),
    ));
}

/// A surrogate-pair literal before the second variant proves the mapping is
/// byte-based rather than counted in UTF-16 code units.
#[test]
fn astral_unicode_before_a_variant_keeps_exact_authored_ranges() {
    const ASTRAL: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('🎨 primary', 2)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 'two')" />
  </variant>
</art>
"#;

    let generated = generate(ASTRAL);
    let secondary = unique_range(ASTRAL, "format('secondary', 'two')");

    assert_eq!(generated.len(), 2);
    assert!(covers(&generated[1].1, &secondary));
}
