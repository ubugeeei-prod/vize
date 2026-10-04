import assert from "node:assert/strict";

// Immutable whole original wires pin this pack's expectations, never product
// admission. They retain historical options alongside complete current oracles.
export const ORIGINAL_REF_WIRES = [
  {
    id: "ref-string-untyped",
    history: "cba6fd5bbb7519696059416fd77aa1ba3192cba3",
    source:
      '<script setup lang="ts">\nconst text = "ref(null)"\n</script>\n<template><div ref="text" /></template>\n',
    filename: "History.vue",
    entry: "sfc",
    rule: "script/prefer-use-template-ref",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "ref-string-typed",
    history: "cba6fd5bbb7519696059416fd77aa1ba3192cba3",
    source:
      '<script setup lang="ts">\nconst text = "ref<HTMLInputElement | null>(null)"\n</script>\n<template><div ref="text" /></template>\n',
    filename: "History.vue",
    entry: "sfc",
    rule: "script/prefer-use-template-ref",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "ref-call-control",
    history: "cba6fd5bbb7519696059416fd77aa1ba3192cba3",
    source:
      '<script setup lang="ts">\nimport { ref } from "vue"\nconst text = ref<HTMLInputElement | null>(null)\n</script>\n<template><div ref="text" /></template>\n',
    filename: "History.vue",
    entry: "sfc",
    rule: "script/prefer-use-template-ref",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
];

// This original explicitly disabled option leaves no active callback. The
// retained authored Vapor block still requires genuine Descriptor admission.
export const ORIGINAL_NEXT_TICK_DISABLED_WIRE = {
  id: "next-tick-disabled",
  history: "f9fa82f7867e3a9373a8d0ee30162c6947c7c113",
  source:
    '<script setup vapor>\nimport { nextTick } from "vue"\nawait nextTick()\n</script>\n<template><div /></template>\n',
  filename: "History.vue",
  entry: "sfc",
  rule: "script/no-next-tick",
  vue_version: null,
  vapor: false,
  diagnostics: 0,
  fixes: 0,
};

export const ORIGINAL_DISABLED_VAPOR_REFUSAL =
  "Descriptor { issues: [DescriptorIssue { code: UnsupportedAttribute, container_index: Some(0), span: Span { start: 14, end: 19 } }], errors: [] }";

export const ORIGINAL_REF_CALL_REFUSAL =
  "FileIssues { issues: [FileIssue { unit: ScriptUnitId(0), span: Span { start: 64, end: 98 }, kind: UnsupportedSyntax }], interruptions: [] }";

export function expectedSfcReason(input: any, api: string) {
  if (input.id === ORIGINAL_NEXT_TICK_DISABLED_WIRE.id) {
    assert.deepEqual(
      input,
      ORIGINAL_NEXT_TICK_DISABLED_WIRE,
      "complete disabled original SFC wire must stay exact",
    );
    return { api, entry: "sfc", kind: "Descriptor", detail: ORIGINAL_DISABLED_VAPOR_REFUSAL };
  }
  if (input.rule !== "script/prefer-use-template-ref")
    return {
      api,
      entry: "sfc",
      kind: "UnprovidedRule",
      detail: `UnprovidedRule { rule: "${input.rule}" }`,
    };
  const original = ORIGINAL_REF_WIRES.find((wire) => wire.id === input.id);
  assert(original, "only the three immutable original ref wires are registered");
  assert.deepEqual(input, original, "complete original SFC input/options must stay exact");
  return original.id === "ref-call-control"
    ? { api, entry: "sfc", kind: "FileIssues", detail: ORIGINAL_REF_CALL_REFUSAL }
    : null;
}
