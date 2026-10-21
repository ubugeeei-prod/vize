import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { loadLinterManifest } from "../differential/linter-api.ts";
import {
  decodeNativeOutcome,
  expectedNativeReason,
  NATIVE_APIS,
  validateNativeContract,
} from "../differential/linter-native.ts";
import { ORIGINAL_REF_WIRES } from "../differential/linter-sfc-contract.ts";
import { sha256 } from "../differential/harness.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifest = path.join(root, "tests/_fixtures/differential/linter/manifest.json");
const loaded = loadLinterManifest(manifest, root);
// Independent authored whole vector; never derive its geometry from a native
// parser or expectedNativeReason and never grant execution credit here.
const positive =
  "FileIssues { issues: [FileIssue { unit: ScriptUnitId(0), span: Span { start: 64, end: 98 }, kind: UnsupportedSyntax }], interruptions: [] }";
function missing(rule: string) {
  return {
    api: "--native-current-api",
    entry: "sfc",
    kind: "UnprovidedRule",
    detail: `UnprovidedRule { rule: "${rule}" }`,
  };
}
const unsupportedTypeAware = {
  api: "--native-current-api",
  entry: "sfc",
  kind: "UnsupportedTypeAwareMode",
  detail: "UnsupportedTypeAwareMode",
};
function row(id: string) {
  return loaded.cases.find((fixture: any) => fixture.id === `linter/${id}`);
}
function packet(value: any) {
  return Buffer.from(`${JSON.stringify(value)}\n`);
}

void test("all current SFC wires have the complete bounded transition and old packs stay exact", () => {
  assert.equal(
    sha256(fs.readFileSync(manifest)),
    "986520db31a12e2eddb18fd6ad89e1e18f87701c4bcf97937a67db5cf1bde0a8",
  );
  assert.equal(
    sha256(
      fs.readFileSync(path.join(root, "tests/_fixtures/differential/linter/oracle-index.json")),
    ),
    "9a2dc43784e1546f490a1f6251f1263d064c8adeb6d727dca010d562b234d9b4",
  );
  const expected: [string, ReturnType<typeof missing> | null][] = [
    ["current-api/ref-string-untyped", null],
    ["current-api/ref-string-typed", null],
    ["current-api/patina-output-policy-9869-9874", missing("vue/require-scoped-style")],
    [
      "current-api/ref-call-control",
      { api: "--native-current-api", entry: "sfc", kind: "FileIssues", detail: positive },
    ],
    [
      "current-api/slot-compat-vue2",
      {
        api: "--native-current-api",
        entry: "sfc",
        kind: "UnsupportedVueVersion",
        detail: "UnsupportedVueVersion { requested: V2 }",
      },
    ],
    [
      "current-api/shorthand-compat-vue2",
      {
        api: "--native-current-api",
        entry: "sfc",
        kind: "UnsupportedVueVersion",
        detail: "UnsupportedVueVersion { requested: V2 }",
      },
    ],
    ["current-api/slot-compat-vue3", missing("vue/no-deprecated-slot-attribute")],
    ["current-api/shorthand-compat-vue3", missing("vue/prefer-props-shorthand")],
    ["current-api/next-tick-unspecified", missing("script/no-next-tick")],
    [
      "current-api/cli-preset-validation-9853",
      {
        api: "--native-current-api",
        entry: "sfc",
        kind: "UnsupportedVaporMode",
        detail: "UnsupportedVaporMode { requested: true }",
      },
    ],
    [
      "current-api/next-tick-disabled",
      {
        api: "--native-current-api",
        entry: "sfc",
        kind: "Descriptor",
        detail:
          "Descriptor { issues: [DescriptorIssue { code: UnsupportedAttribute, container_index: Some(0), span: Span { start: 14, end: 19 } }], errors: [] }",
      },
    ],
    ["current-api/static-class-utf8-fix-corrected", missing("vapor/prefer-static-class")],
    ["current-api/valid-v-else-autofix-preserves-directive", missing("vue/valid-v-else")],
    ["current-api/dynamic-slot-v-for-component-bodies", missing("vue/valid-v-for")],
    ["current-api/incomplete-type-project-10005", missing("vue/valid-v-for")],
    ["current-api/dotted-slot-name-9865", missing("vue/valid-v-slot")],
    ["current-api/multiline-v-for-source-9880", unsupportedTypeAware],
    ["current-api/type-aware-slot-outlet-9834", missing("vue/valid-v-for")],
    ["current-api/vue-path-alias-types-9864", unsupportedTypeAware],
    ["current-api/nuxt-builtins-9866", missing("vue/require-component-registration")],
    ["current-api/patina-rule-guards-9870-9871", missing("vue/attribute-order")],
    ["next-tick/valid-arrow-expression-return-sfc", missing("script/valid-next-tick")],
    [
      "next-tick/valid-parenthesized-arrow-expression-return-sfc",
      missing("script/valid-next-tick"),
    ],
    ["next-tick/invalid-arrow-block-bare-call-sfc", missing("script/valid-next-tick")],
    ["next-tick/invalid-arrow-expression-void-call-sfc", missing("script/valid-next-tick")],
    ["next-tick/nested-arrow-restores-statement-context-sfc", missing("script/valid-next-tick")],
  ];
  expected.sort((a, b) => a[0].localeCompare(b[0]));
  const actual = loaded.cases
    .filter(
      (fixture: any) =>
        fixture.argv[0] === "--current-api" && JSON.parse(fixture.input).entry === "sfc",
    )
    .map((fixture: any) => [fixture.id.slice(7), expectedNativeReason(fixture)])
    .sort((a: any, b: any) => a[0].localeCompare(b[0]));
  assert.deepEqual(actual, expected);
  for (const wire of ORIGINAL_REF_WIRES)
    assert.deepEqual(JSON.parse(row(`current-api/${wire.id}`).input), wire);
  assert.deepEqual(expectedNativeReason(row("static-class/utf8-crlf-sfc")), {
    api: "--native-static-class",
    entry: "sfc",
    kind: "EntryUnavailable",
    detail: "native original sfc entry is not provided",
  });
});

void test("original typed positive cannot become handled or alter any complete issue/interrupt vector", () => {
  const fixture = row("current-api/ref-call-control");
  const reason = {
    api: "--native-current-api",
    entry: "sfc",
    kind: "FileIssues",
    detail: positive,
  };
  assert.deepEqual(decodeNativeOutcome(packet({ state: "unsupported", reason }), fixture), {
    state: "unsupported",
    reason,
  });
  assert.throws(() =>
    decodeNativeOutcome(
      packet({ state: "handled", observation: fixture.expected.toString() }),
      fixture,
    ),
  );
  for (const detail of [
    positive.replace("ScriptUnitId(0)", "ScriptUnitId(1)"),
    positive.replace("start: 64", "start: 63"),
    positive.replace("end: 98", "end: 97"),
    positive.replace("UnsupportedSyntax", "UnresolvedName"),
    "FileIssues { issues: [], interruptions: [] }",
    positive.replace("interruptions: []", "interruptions: [InterruptedProgram]"),
    `Sfc(${positive})`,
  ])
    assert.throws(() =>
      decodeNativeOutcome(packet({ state: "unsupported", reason: { ...reason, detail } }), fixture),
    );
  for (const id of ["ref-string-untyped", "ref-string-typed"])
    assert.throws(() =>
      decodeNativeOutcome(
        packet({ state: "unsupported", reason: missing("script/prefer-use-template-ref") }),
        row(`current-api/${id}`),
      ),
    );
});

void test("handled string packets cannot rewrite any original source/options/history wire", () => {
  const fixture = row("current-api/ref-string-untyped");
  const value = JSON.parse(fixture.input);
  const handled = packet({ state: "handled", observation: fixture.expected.toString() });
  for (const change of [
    (v: any) => {
      v.source += "<!-- unprovided -->";
    },
    (v: any) => {
      v.filename = "Other.vue";
    },
    (v: any) => {
      v.history = "0".repeat(40);
    },
    (v: any) => {
      v.id = "invented-string";
    },
    (v: any) => {
      v.diagnostics = 1;
    },
    (v: any) => {
      v.fixes = 1;
    },
    (v: any) => {
      v.vue_version = "3";
    },
    (v: any) => {
      v.vapor = false;
    },
    (v: any) => {
      v.rule = "script/no-next-tick";
    },
    (v: any) => {
      v.entry = "script";
    },
  ]) {
    const changed = structuredClone(value);
    change(changed);
    assert.throws(() => decodeNativeOutcome(handled, { ...fixture, input: packet(changed) }));
  }
});

function probes() {
  const values = [
    {
      schema: "vize.linter-history-observer",
      version: 3,
      apis: ["--current-api", "--report", "--static-class"],
      preset: "Incremental",
      locale: "En",
      help: "Full",
      native: "configured-template-and-sfc",
      nativeApis: NATIVE_APIS,
    },
    {
      schema: "vize.linter-native-observer",
      version: 2,
      apis: NATIVE_APIS,
      owners: { template: "NativeLintComponent", sfc: "NativeSfcLintOwner" },
      entries: ["template", "sfc"],
      wholeOutput: "Case+Observation",
      fallback: false,
    },
  ];
  return values.map((value, i) => {
    const bytes = packet(value);
    return {
      argv: [i ? "--native-contract" : "--contract"],
      exitStatus: 0,
      stdoutBase64: bytes.toString("base64"),
      sha256: sha256(bytes),
    };
  });
}
void test("recomputed probe hashes cannot hide old versions, wrong original owner maps or entry routing", () => {
  validateNativeContract({ probes: probes() });
  for (const [index, change] of [
    [
      0,
      (v: any) => {
        v.version = 2;
      },
    ],
    [
      0,
      (v: any) => {
        v.native = "configured-bare-template";
      },
    ],
    [
      1,
      (v: any) => {
        v.version = 1;
      },
    ],
    [
      1,
      (v: any) => {
        delete v.owners.sfc;
      },
    ],
    [
      1,
      (v: any) => {
        v.owners.sfc = "NativeTemplateFile";
      },
    ],
    [
      1,
      (v: any) => {
        v.entries.reverse();
      },
    ],
    [
      1,
      (v: any) => {
        v.entries.push("script");
      },
    ],
    [
      1,
      (v: any) => {
        v.fallback = true;
      },
    ],
  ] as const) {
    const receipt = { probes: probes() };
    const probe = receipt.probes[index];
    const value = JSON.parse(Buffer.from(probe.stdoutBase64, "base64").toString());
    change(value);
    const bytes = packet(value);
    probe.stdoutBase64 = bytes.toString("base64");
    probe.sha256 = sha256(bytes);
    assert.throws(() => validateNativeContract(receipt));
  }
});

void test("disabled original Vapor wire retains its complete Descriptor refusal before any clean credit", () => {
  const fixture = row("current-api/next-tick-disabled");
  assert.deepEqual(JSON.parse(fixture.input), {
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
  });
  const reason = {
    api: "--native-current-api",
    entry: "sfc",
    kind: "Descriptor",
    detail:
      "Descriptor { issues: [DescriptorIssue { code: UnsupportedAttribute, container_index: Some(0), span: Span { start: 14, end: 19 } }], errors: [] }",
  };
  assert.deepEqual(decodeNativeOutcome(packet({ state: "unsupported", reason }), fixture), {
    state: "unsupported",
    reason,
  });
  assert.throws(() =>
    decodeNativeOutcome(
      packet({ state: "handled", observation: fixture.expected.toString() }),
      fixture,
    ),
  );
  for (const changed of [
    missing("script/no-next-tick"),
    { ...reason, detail: reason.detail.replace("Some(0)", "Some(1)") },
    { ...reason, detail: reason.detail.replace("start: 14", "start: 13") },
    { ...reason, detail: reason.detail.replace("end: 19", "end: 20") },
    { ...reason, detail: reason.detail.replace("UnsupportedAttribute", "UnsupportedRole") },
    { ...reason, detail: "Descriptor { issues: [], errors: [] }" },
  ])
    assert.throws(() =>
      decodeNativeOutcome(packet({ state: "unsupported", reason: changed }), fixture),
    );
  for (const vapor of [null, true])
    assert.throws(() =>
      decodeNativeOutcome(packet({ state: "unsupported", reason }), {
        ...fixture,
        input: packet({ ...JSON.parse(fixture.input), vapor }),
      }),
    );
});
