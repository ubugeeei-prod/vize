import assert from "node:assert/strict";
import { test } from "node:test";

import {
  composedCopies,
  composedSource,
  generateComposedCopies,
  isComposedCopy,
} from "../../tools/commands/generate/composed-copies.ts";

// The copies are generated from these sources, so a change to any of them
// runs this test.
const plainSources = [
  "crates/vize_atelier_core/src",
  "crates/vize_atelier_dom/src",
  "crates/vize_atelier_ssr/src",
];

test("every composed copy matches the source it is generated from", () => {
  assert.deepEqual(
    composedCopies.map((spec) => `crates/${spec.crate}/src`),
    plainSources,
  );
  assert.equal(generateComposedCopies(["--check"]), 0);
});

test("a copy names itself and the copies of its dependencies", () => {
  const dom = composedCopies.find((spec) => spec.crate === "vize_atelier_dom");
  assert(dom);
  const source = [
    "use crate::options::DomCompilerOptions;",
    "use vize_atelier_core::codegen::generate;",
    "pub(crate) fn f() { $crate::g!() }",
    "dom_map_copy! { plain { a() } composed { b() } }",
  ].join("\n");
  const copy = composedSource(dom, "compile/inner.rs", source);
  assert.match(copy, /^\/\/ DO NOT EDIT\. Generated from crates\/vize_atelier_dom\/src\/compile\/inner\.rs/u);
  assert.match(copy, /use crate::composed::options::DomCompilerOptions;/u);
  assert.match(copy, /use vize_atelier_core::composed::codegen::generate;/u);
  assert.match(copy, /pub\(crate\) fn f\(\) \{ \$crate::composed::g!\(\) \}/u);
  // Only `lib.rs` defines the macro; its uses stay as they are.
  assert.match(copy, /dom_map_copy! \{ plain \{ a\(\) \} composed \{ b\(\) \} \}/u);
});

test("a shared module is named from the plain source", () => {
  const core = composedCopies.find((spec) => spec.crate === "vize_atelier_core");
  assert(core);
  const copy = composedSource(core, "codegen/context.rs", "mod result;\nuse crate::codegen::Foo;\n");
  assert.match(copy, /^use crate::codegen as result;$/mu);
  assert.match(copy, /^use crate::composed::codegen::Foo;$/mu);
});

test("only the copy modules count as copies", () => {
  assert(isComposedCopy("crates/vize_atelier_core/src/composed.rs"));
  assert(isComposedCopy("crates/vize_atelier_ssr/src/composed/codegen.rs"));
  assert(!isComposedCopy("crates/vize_atelier_dom/src/compile/inner/composed.rs"));
  assert(!isComposedCopy("crates/vize_atelier_sfc/src/composed.rs"));
});
