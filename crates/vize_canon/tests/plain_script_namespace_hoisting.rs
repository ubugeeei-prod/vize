//! A `namespace` in a plain (non-setup) `<script>` has to reach module scope.
//!
//! The plain-`<script>` body is moved inside `__setup()` so its diagnostics stay
//! anchored to user code, but TypeScript only accepts a namespace at the top
//! level of a module or namespace. Leaving one in the function body raised TS1235
//! and the export bridge never reached it, so a consumer also got TS2614 — both
//! false, because the authored `<script>` body *is* module scope. See #3383.

use std::fs;
use std::path::{Path, PathBuf};

use vize_canon::VirtualProject;

const MODULE_HEADER: &str = "// ========== Module Scope (imports) ==========\n";
const SETUP_HEADER: &str = "// ========== Setup Scope ==========";
const BRIDGE_HEADER: &str = "// Invoke setup to verify types\n";
const BRIDGE_END: &str = "export type Emits =";
/// Emitted by the generator ahead of every module statement in these fixtures,
/// because each one has an object-literal `export default`.
const DEFINE_COMPONENT_HELPER: &str =
    "declare const __vizeDefineComponent: typeof import('vue').defineComponent;\n";

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

/// The generated module for `vue_content`, as `(module scope, export bridge)`.
///
/// Both slices are delimited by generator section headers, so each assertion
/// below compares a *complete* region rather than searching for a fragment.
fn generated_sections(name: &str, vue_content: &str) -> (String, String) {
    let case_dir = unique_case_dir(name);
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("Namespaces.vue");
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();
    let content = project
        .find_by_original(&vue_path)
        .unwrap()
        .content
        .as_str()
        .to_string();
    let _ = fs::remove_dir_all(&case_dir);

    let module_start = content.find(MODULE_HEADER).expect("module scope header")
        + MODULE_HEADER.len()
        + DEFINE_COMPONENT_HELPER.len();
    let module_end = content.find(SETUP_HEADER).expect("setup scope header");
    let bridge_start = content.find(BRIDGE_HEADER).expect("bridge header") + BRIDGE_HEADER.len();
    let bridge_end = content.find(BRIDGE_END).expect("emits type");
    assert!(
        content[MODULE_HEADER.len()..].contains(DEFINE_COMPONENT_HELPER),
        "fixture should emit the defineComponent helper:\n{content}"
    );
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &content, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "virtual TS should parse without errors: {:?}",
        parsed.diagnostics
    );

    (
        content[module_start..module_end].to_string(),
        content[bridge_start..bridge_end].to_string(),
    )
}

#[test]
fn every_namespace_keyword_form_reaches_module_scope() {
    let (module, bridge) = generated_sections(
        "plain-script-namespace-forms",
        r#"<script lang="ts">
namespace Bare {
  export const a = 1;
}
export namespace Named {
  export type T = string;
  export const b = 2;
}
export namespace A.B.C {
  export const d = 4;
}
declare namespace Ambient {
  const e: number;
}
export default { name: "Namespaces" };
</script>
"#,
    );

    // Verbatim relocation: the `export` keyword travels with the declaration, so
    // an exported namespace stays exported and `Bare` stays module-local.
    assert_eq!(
        module,
        "namespace Bare {\n  export const a = 1;\n}\nexport namespace Named {\n  export type T = string;\n  export const b = 2;\n}\nexport namespace A.B.C {\n  export const d = 4;\n}\ndeclare namespace Ambient {\n  const e: number;\n}\n\n// ========== Exported Types ==========\nexport type Props = {};\n\n"
    );
    // No value crossed the setup boundary, so the bridge stays a bare call.
    assert_eq!(bridge, "__setup();\n\n");
}

#[test]
fn the_legacy_module_keyword_form_reaches_module_scope() {
    let (module, bridge) = generated_sections(
        "plain-script-namespace-legacy-module",
        r#"<script lang="ts">
export module Legacy {
  export const a = 1;
}
export default { name: "Namespaces" };
</script>
"#,
    );

    // `module` is relocated unchanged rather than rewritten to `namespace`: TS
    // reports TS1540 on the keyword, and that diagnostic is the authored code's,
    // not vize's to hide.
    assert_eq!(
        module,
        "export module Legacy {\n  export const a = 1;\n}\n\n// ========== Exported Types ==========\nexport type Props = {};\n\n"
    );
    assert_eq!(bridge, "__setup();\n\n");
}

#[test]
fn same_named_namespace_blocks_are_relocated_separately() {
    let (module, bridge) = generated_sections(
        "plain-script-namespace-merged-blocks",
        r#"<script lang="ts">
export namespace Twice {
  export const one = 1;
}
export namespace Twice {
  export type Two = string;
}
export default { name: "Namespaces" };
</script>
"#,
    );

    // Two blocks merge in TypeScript, so both have to arrive at module scope;
    // collapsing them to one would drop half of the namespace.
    assert_eq!(
        module,
        "export namespace Twice {\n  export const one = 1;\n}\nexport namespace Twice {\n  export type Two = string;\n}\n\n// ========== Exported Types ==========\nexport type Props = {};\n\n"
    );
    assert_eq!(bridge, "__setup();\n\n");
}

#[test]
fn a_merge_partner_declaration_travels_with_the_namespace() {
    let (module, bridge) = generated_sections(
        "plain-script-namespace-merge-partners",
        r#"<script lang="ts">
export class C {
  x = 1;
}
export namespace C {
  export type T = number;
}
export function f() {
  return 1;
}
export namespace f {
  export const v = 1;
}
export enum E {
  A = "a",
}
export namespace E {
  export const label = "e";
}
export const untouched = 3;
export default { name: "Namespaces" };
</script>
"#,
    );

    // A namespace merges with a same-named class/function/enum. Bridging the
    // partner as `export const C = …` would make the module-scope namespace
    // collide with a block-scoped variable (TS2451/TS2300), so the partner is
    // relocated too and the merge is reproduced as authored.
    assert_eq!(
        module,
        "export class C {\n  x = 1;\n}\nexport namespace C {\n  export type T = number;\n}\nexport function f() {\n  return 1;\n}\nexport namespace f {\n  export const v = 1;\n}\nexport enum E {\n  A = \"a\",\n}\nexport namespace E {\n  export const label = \"e\";\n}\n\n// ========== Exported Types ==========\nexport type Props = {};\n\n"
    );
    // Only the declaration with no namespace to merge into still needs the
    // bridge, and it keeps the value-only shape #3382 established.
    assert_eq!(
        bridge,
        "const __vize_plain_script_exports = __setup();\nexport const untouched = __vize_plain_script_exports.untouched;\n\n"
    );
}

#[test]
fn captured_setup_bindings_are_re_declared_before_the_namespace() {
    let (module, bridge) = generated_sections(
        "plain-script-namespace-captures",
        r#"<script lang="ts">
const localBase = 10;
class Local {
  id = 1;
}
export const shared = 2;
export namespace Uses {
  export const derived = localBase + shared;
  export const made: Local = new Local();
}
export default { name: "Namespaces" };
</script>
"#,
    );

    // The namespace body reads bindings that stay inside `__setup()`. They are
    // re-declared at module scope as ambient aliases of the setup return, which
    // keeps the authored type without duplicating the initializer. `shared` is
    // also a plain-script export, and the bridge's `export const shared = …`
    // lands *after* the namespace (TS2448), so the alias carries the `export`.
    assert_eq!(
        module,
        "// Setup-scope bindings a hoisted namespace body reads\ndeclare const localBase: ReturnType<typeof __setup>[\"localBase\"];\ndeclare const Local: ReturnType<typeof __setup>[\"Local\"];\ntype Local = InstanceType<typeof Local>;\nexport declare const shared: ReturnType<typeof __setup>[\"shared\"];\n\nexport namespace Uses {\n  export const derived = localBase + shared;\n  export const made: Local = new Local();\n}\n\n// ========== Exported Types ==========\nexport type Props = {};\n\n"
    );
    // `shared` keeps only its type-space obligations here; its value side is the
    // ambient alias above, so the bridge must not declare the name twice.
    assert_eq!(bridge, "__setup();\n\n");
}
