import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const DEFINITION_PREFIX = "ide::definition::service::private_setup_tests::";
const PREFIX = DEFINITION_PREFIX;
export const DEFINITION_TESTS = ["private_template_definition_requires_exact_declaration_and_half_open_query", "private_template_definition_refuses_stale_foreign_and_provider_uncertain_owners", "private_template_definition_matches_the_complete_frozen_native_dispatcher"].map(name => PREFIX + name).sort();
const TESTS = DEFINITION_TESTS;
export const DEFINITION_FIXTURE_SHA256 = "1c384d581eeb402c4a0d25e9e93c76d02c37d038387cb8b163ab0dbbbdc15476";
export const DEFINITION_BOUND_PATHS = [
  "crates/vize_maestro/src/ide/definition/service.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests/original.rs",
  "crates/vize_maestro/src/ide/references.rs",
  "crates/vize_maestro/src/ide/references/canonical.rs",
  "crates/vize_maestro/src/ide/references/canonical/local_binding.rs",
  "crates/vize_maestro/src/ide/references/structural.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot.rs",
  "crates/vize_maestro/src/ide/references/structural/guard.rs",
  "crates/vize_maestro/src/ide/references/structural/template.rs",
  "crates/vize_maestro/src/ide/references/structural/edits.rs",
  "crates/vize_maestro/src/ide/references/structural/style.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot/registry.rs",
  "crates/vize_maestro/src/document/store/structural_navigation.rs",
  "crates/vize_maestro/src/ide/context.rs",
  "crates/vize_maestro/src/ide/definition/service/locations.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target/alias.rs",
  "crates/vize_maestro/src/ide/definition/service/template_props.rs",
  "crates/vize_maestro/src/ide/corsa_support/rename_scope.rs",
  "tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt",
  "npm/cli/content-mapper-helpers.d.cts",
  "npm/cli/content-mapper-helpers.d.mts",
  "npm/cli/content-mapper-helpers.d.ts",
  "npm/cli/generated/content-mapper-types.d.ts",
  "npm/cli/virtual-ts-globals.d.ts",
  "npm/cli/virtual-ts-patterns.d.ts",
  "npm/cli/virtual-ts-slots.d.ts",
  "npm/cli/virtual-ts.d.cts",
  "npm/cli/virtual-ts.d.mts",
  "npm/cli/virtual-ts.d.ts",
  "crates/vize_maestro/Cargo.toml",
  "crates/vize_canon/Cargo.toml",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
  "npm/cli/package.json",
  "npm/cli/scripts/generate-content-mapper-types.ts",
  "crates/vize_maestro/src/test_support.rs",
  "tools/support/compat/github/full-rust-runtime.ts"
] as const;
function object(value: unknown): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function text(value: unknown): string { assert(typeof value === "string"); return value; }
const CASES: readonly { id: string; source: string; offset: number; private: boolean }[] = [
  {"id": "local-first", "source": "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\n\nconst message = ref(\"hello\");\nconst broken: string = 42;\nvoid broken;\n</script>\n\n<template>\n  <main>\n    <p>🚀 {{ message }}</p>\n    <p>{{ message }}</p>\n  </main>\n</template>\n", "offset": 172, "private": true},
  {"id": "local-last", "source": "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\n\nconst message = ref(\"hello\");\nconst broken: string = 42;\nvoid broken;\n</script>\n\n<template>\n  <main>\n    <p>🚀 {{ message }}</p>\n    <p>{{ message }}</p>\n  </main>\n</template>\n", "offset": 197, "private": true},
  {"id": "crlf-first", "source": "<script setup lang=\"ts\">\r\nimport { ref } from \"vue\";\r\n\r\n\r\nconst message = ref(\"hello\");\r\nconst broken: string = 42;\r\nvoid broken;\r\n</script>\r\n\r\n<template>\r\n  <main>\r\n    <p>🚀 {{ message }}</p>\r\n    <p>{{ message }}</p>\r\n  </main>\r\n</template>\r\n", "offset": 183, "private": true},
  {"id": "crlf-last", "source": "<script setup lang=\"ts\">\r\nimport { ref } from \"vue\";\r\n\r\n\r\nconst message = ref(\"hello\");\r\nconst broken: string = 42;\r\nvoid broken;\r\n</script>\r\n\r\n<template>\r\n  <main>\r\n    <p>🚀 {{ message }}</p>\r\n    <p>{{ message }}</p>\r\n  </main>\r\n</template>\r\n", "offset": 209, "private": true},
  {"id": "same-public-prop-local-value", "source": "<script setup lang=\"ts\">import Child from './Child.vue'; const message = 'local';</script><template><Child :message=\"message\" /></template>", "offset": 119, "private": true},
  {"id": "unicode", "source": "<script setup lang=\"ts\">const before = '雪🌸'; const message = '日本語';</script><template>🚀 {{ message }}</template>", "offset": 107, "private": true},
  {"id": "import", "source": "<script setup lang=\"ts\">import { message } from './shared';</script><template>{{ message }}</template>", "offset": 83, "private": false},
  {"id": "props", "source": "<script setup lang=\"ts\">const { message } = defineProps<{ message: string }>();</script><template>{{ message }}</template>", "offset": 103, "private": false},
  {"id": "model", "source": "<script setup lang=\"ts\">const message = defineModel<string>();</script><template>{{ message }}</template>", "offset": 86, "private": false},
  {"id": "expose", "source": "<script setup lang=\"ts\">const message = 1; defineExpose({ message });</script><template>{{ message }}</template>", "offset": 93, "private": false},
  {"id": "emits", "source": "<script setup lang=\"ts\">const message = 1; defineEmits<{ save: [] }>();</script><template>{{ message }}</template>", "offset": 95, "private": false},
  {"id": "slots", "source": "<script setup lang=\"ts\">const message = 1; defineSlots<{ default(): unknown }>();</script><template>{{ message }}</template>", "offset": 105, "private": false},
  {"id": "options", "source": "<script setup lang=\"ts\">const message = 1; defineOptions({ name: 'Public' });</script><template>{{ message }}</template>", "offset": 101, "private": false},
  {"id": "defaults", "source": "<script setup lang=\"ts\">const message = 1; withDefaults(defineProps<{ x?: number }>(), { x: 1 });</script><template>{{ message }}</template>", "offset": 121, "private": false},
  {"id": "export", "source": "<script setup lang=\"ts\">export const message = 1;</script><template>{{ message }}</template>", "offset": 73, "private": false},
  {"id": "export-list", "source": "<script setup lang=\"ts\">const message = 1; export { message };</script><template>{{ message }}</template>", "offset": 86, "private": false},
  {"id": "pattern", "source": "<script setup lang=\"ts\">const { message } = store;</script><template>{{ message }}</template>", "offset": 74, "private": false},
  {"id": "object-shorthand", "source": "<script setup lang=\"ts\">const message = 1; const value = { message };</script><template>{{ message }}</template>", "offset": 93, "private": false},
  {"id": "computed", "source": "<script setup lang=\"ts\">const message = 1; const value = { ['x']: message };</script><template>{{ message }}</template>", "offset": 100, "private": false},
  {"id": "member", "source": "<script setup lang=\"ts\">const message = 1; const value = store.message;</script><template>{{ message }}</template>", "offset": 95, "private": false},
  {"id": "capture", "source": "<script setup lang=\"ts\">const message = 1; const f = () => message;</script><template>{{ message }}</template>", "offset": 91, "private": false},
  {"id": "shadow", "source": "<script setup lang=\"ts\">const message = 1; const f = (message: number) => message;</script><template>{{ message }}</template>", "offset": 106, "private": false},
  {"id": "let", "source": "<script setup lang=\"ts\">let message = 1;</script><template>{{ message }}</template>", "offset": 64, "private": false},
  {"id": "assignment", "source": "<script setup lang=\"ts\">const message = 1; message = 2;</script><template>{{ message }}</template>", "offset": 79, "private": false},
  {"id": "update", "source": "<script setup lang=\"ts\">const message = 1; message++;</script><template>{{ message }}</template>", "offset": 77, "private": false},
  {"id": "attribute-shorthand", "source": "<script setup lang=\"ts\">const message = 1;</script><template><Child :message /></template>", "offset": 71, "private": false},
  {"id": "public-property-key", "source": "<script setup lang=\"ts\">const message = 1;</script><template><Child :message=\"message\" /></template>", "offset": 71, "private": false},
  {"id": "template-shadow", "source": "<script setup lang=\"ts\">const message = 1;</script><template><i v-for=\"message in [1]\">{{ message }}</i></template>", "offset": 92, "private": false},
  {"id": "invalid", "source": "<script setup lang=\"ts\">const message = ;</script><template>{{ message }}</template>", "offset": 65, "private": false},
  {"id": "dual-script", "source": "<script setup lang=\"ts\">const message = 1;</script><template>{{ message }}</template><script>export default {}</script>", "offset": 66, "private": false},
  {"id": "style", "source": "<script setup lang=\"ts\">const message = 1;</script><template>{{ message }}</template><style>.x{color:v-bind(message)}</style>", "offset": 66, "private": false},
  {"id": "art", "source": "<script setup lang=\"ts\">const message = 1;</script><template>{{ message }}</template><art title=\"A\" component=\"./Child.vue\"><variant name=\"V\">{{ message }}</variant></art>", "offset": 66, "private": false},
  {"id": "token-end", "source": "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\n\nconst message = ref(\"hello\");\nconst broken: string = 42;\nvoid broken;\n</script>\n\n<template>\n  <main>\n    <p>🚀 {{ message }}</p>\n    <p>{{ message }}</p>\n  </main>\n</template>\n", "offset": 202, "private": false},
  {"id": "non-token", "source": "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\n\nconst message = ref(\"hello\");\nconst broken: string = 42;\nvoid broken;\n</script>\n\n<template>\n  <main>\n    <p>🚀 {{ message }}</p>\n    <p>{{ message }}</p>\n  </main>\n</template>\n", "offset": 162, "private": false},
];
function execution(stdout: string): void {
  assert.deepEqual([...stdout.matchAll(/^test (\S+) \.\.\. ok$/gm)].map(match => match[1]).sort(), TESTS);
  const summaries = [...stdout.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1); assert.deepEqual(summaries[0]?.slice(1), ["3", "0", "0", "0"]);
}
function expectedRange(source: string): Record<string, unknown> {
  const bytes = Buffer.from(source), start = bytes.indexOf("const message") + Buffer.byteLength("const ");
  assert(start >= Buffer.byteLength("const "));
  const lines = bytes.subarray(0, start).toString("utf8").split("\n");
  const position = { line: lines.length - 1, character: lines.at(-1)?.length ?? 0 };
  return { start: position, end: { line: position.line, character: position.character + 7 } };
}
function definitionPairs(stderr: string): Record<string, unknown>[] {
  assert.ok(!stderr.includes("VIZE_PRIVATE_DEFINITION_NATIVE_DISABLED"));
  const rows = stderr.split("\n").filter(line => line.startsWith("VIZE_PRIVATE_DEFINITION_NATIVE "))
    .map(line => object(JSON.parse(line.slice("VIZE_PRIVATE_DEFINITION_NATIVE ".length))));
  assert.equal(rows.length, 34); let positive = 0;
  for (const [index, row] of rows.entries()) {
    assert.deepEqual(Object.keys(row).sort(), ["actual", "admitted", "case", "id", "offset", "old", "private", "source"]);
    const expected = CASES[index]; assert(expected !== undefined);
    assert.deepEqual({ id: row["id"], source: row["source"], offset: row["offset"], private: row["private"] }, expected);
    assert.equal(row["case"], index); assert.equal(row["admitted"], expected.private);
    for (const key of ["old", "actual"]) assert.ok(row[key] === null || typeof row[key] === "object", "complete Option enum/object required");
    assert.deepEqual(row["actual"], row["old"], "preserve complete arrays, enum shapes and every nested/unknown field");
    if (expected.private) {
      positive++; const packet = object(row["actual"]), uri = text(packet["uri"]), url = new URL(uri);
      assert.equal(url.protocol, "file:"); assert.equal(url.host, ""); assert.equal(url.search, ""); assert.equal(url.hash, "");
      const filename = fileURLToPath(url); assert.equal(path.basename(filename), "App.vue"); assert.equal(pathToFileURL(filename).href, uri);
      assert.deepEqual(packet, { uri, range: expectedRange(expected.source) });
    }
  }
  assert.equal(positive, 6); return rows;
}
const SDK_CRATES = ["corsa", "corsa_client", "corsa_core", "corsa_jsonrpc", "corsa_lsp", "corsa_orchestrator", "corsa_runtime"];
function libraryArtifact(values: Record<string, unknown>[], target: string, repo: string): Record<string, unknown> {
  assert.equal(values.filter(row => row["reason"] === "build-finished" && row["success"] === true).length, 1);
  const artifacts = values.filter(row => row["reason"] === "compiler-artifact");
  const libraries = artifacts.filter(row => object(row["target"])["name"] === "vize_maestro" && object(row["profile"])["test"] === true && typeof row["executable"] === "string");
  assert.equal(libraries.length, 1); const library = libraries[0]; assert(library !== undefined);
  const features = library["features"]; assert(Array.isArray(features) && features.includes("native"));
  assert.ok(artifacts.every(row => { const f = row["features"]; return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling"); }));
  const meta = object(library["target"]); assert.deepEqual(meta["kind"], ["lib"]);
  assert.equal(library["manifest_path"], path.join(repo, "crates/vize_maestro/Cargo.toml"));
  assert.equal(meta["src_path"], path.join(repo, "crates/vize_maestro/src/lib.rs"));
  assert.equal(text(library["package_id"]).split("#")[0], "path+" + pathToFileURL(path.join(repo, "crates/vize_maestro")).href);
  const executable = text(library["executable"]); assert.ok(path.isAbsolute(executable));
  const relative = path.relative(target, executable); assert.ok(relative.startsWith("ci/deps/") && !relative.split(path.sep).includes(".."));
  const sdk = artifacts.filter(row => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual([...new Set(sdk.map(row => text(row["package_id"]).split("#")[1]))].sort(), SDK_CRATES.map(name => name + "@2.0.0-beta.4").sort());
  assert.ok(sdk.every(row => text(row["package_id"]).startsWith("registry+") && JSON.stringify(object(row["target"])["kind"]) === '["lib"]'));
  return library;
}

export function definitionEnvironment(environment: NodeJS.ProcessEnv): void {
  assert.equal(environment.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(environment.VIZE_TEST_DISABLE_TSGO, undefined, "explicit disable must be absent, including empty string");
  assert.equal(environment.VIZE_TEST_TSGO_PATH, undefined, "alternate test provider must be absent");
}
export function sdkArtifacts(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  const artifacts = rows.filter(row => row["reason"] === "compiler-artifact");
  assert.ok(artifacts.every(row => { const f = row["features"]; return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling"); }));
  const sdk = artifacts.filter(row => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual([...new Set(sdk.map(row => text(row["package_id"]).split("#")[1]))].sort(), SDK_CRATES.map(name => name + "@2.0.0-beta.4").sort());
  assert.ok(sdk.every(row => text(row["package_id"]).startsWith("registry+") && JSON.stringify(object(row["target"])["kind"]) === '["lib"]'));
  return sdk;
}
export { CASES as DEFINITION_CASES, execution as checkDefinitionExecution, definitionPairs, libraryArtifact, SDK_CRATES };
