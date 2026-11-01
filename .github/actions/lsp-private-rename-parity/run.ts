import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";

const BASE = "7ea42da746a0d6e40d8ab15cd201c2c0586afd6f";
const CANDIDATE = "9e5963e8156f1d357c79d908619b6491735fc9ca";
const SOURCE_PARENT = "b8597f70b644b318c7a4d3e71211dd3de35eaeb1";
const SOURCE_GRANDPARENT = "07145d9dac0dfcb2a730b13f6487081cdb904251";
const PREFIX = "ide::definition::service::private_setup_tests::";
const TESTS = ["private_template_definition_requires_exact_declaration_and_half_open_query", "private_template_definition_refuses_stale_foreign_and_provider_uncertain_owners", "private_template_definition_matches_the_complete_frozen_native_dispatcher"].map(name => PREFIX + name).sort();
const SOURCE_PATHS = ["crates/vize_maestro/src/ide/definition/service.rs", "crates/vize_maestro/src/ide/definition/service/private_setup_tests.rs", "crates/vize_maestro/src/ide/definition/service/private_setup_tests/original.rs"].sort();
const REUSED_PATHS = ["crates/vize_maestro/src/ide/references.rs", "crates/vize_maestro/src/ide/references/canonical.rs", "crates/vize_maestro/src/ide/references/canonical/local_binding.rs", "crates/vize_maestro/src/ide/references/structural.rs", "crates/vize_maestro/src/ide/references/structural/snapshot.rs", "crates/vize_maestro/src/ide/references/structural/guard.rs", "crates/vize_maestro/src/ide/references/structural/template.rs", "crates/vize_maestro/src/ide/references/structural/edits.rs", "crates/vize_maestro/src/ide/references/structural/style.rs", "crates/vize_maestro/src/ide/references/structural/snapshot/registry.rs", "crates/vize_maestro/src/document/store/structural_navigation.rs", "crates/vize_maestro/src/ide/context.rs", "crates/vize_maestro/src/ide/definition/service/locations.rs", "crates/vize_maestro/src/ide/definition/service/import_target.rs", "crates/vize_maestro/src/ide/definition/service/import_target/alias.rs", "crates/vize_maestro/src/ide/definition/service/template_props.rs", "crates/vize_maestro/src/ide/corsa_support/rename_scope.rs", "tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt"];
const FIXTURE_SHA256 = "1c384d581eeb402c4a0d25e9e93c76d02c37d038387cb8b163ab0dbbbdc15476";
// Finite authored rows derived from exact C's compiled Rust data and include_str input.
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
const SDK_PATHS = ["npm/cli/content-mapper-helpers.d.cts","npm/cli/content-mapper-helpers.d.mts","npm/cli/content-mapper-helpers.d.ts","npm/cli/generated/content-mapper-types.d.ts","npm/cli/virtual-ts-globals.d.ts","npm/cli/virtual-ts-patterns.d.ts","npm/cli/virtual-ts-slots.d.ts","npm/cli/virtual-ts.d.cts","npm/cli/virtual-ts.d.mts","npm/cli/virtual-ts.d.ts"];
const WORKFLOW_PATHS = [".github/actions/lsp-private-rename-parity/run.ts",".github/workflows/lsp-current-baseline.yml"];
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function text(value: unknown): string { assert(typeof value === "string"); return value; }
function env(name: string): string { return text(process.env[name]); }
function git(...args: string[]): string { return execFileSync("git", args, { encoding: "utf8" }).trim(); }
function completeBlob(revision: string, file: string, physical: Buffer): Buffer {
  const bytes = Number(git("cat-file", "-s", `${revision}:${file}`));
  assert(Number.isSafeInteger(bytes) && bytes >= 0 && bytes <= 4 * 1024 * 1024);
  assert.equal(physical.length, bytes);
  const immutable = execFileSync("git", ["show", `${revision}:${file}`], { maxBuffer: bytes + 1 });
  assert.deepEqual(physical, immutable);
  return immutable;
}
function parent(raw: string, expected: string): void {
  const end = raw.indexOf("\n\n"); assert(end > 0);
  const headers = raw.slice(0, end).split("\n");
  assert.match(headers[0] ?? "", /^tree [a-f0-9]{40}$/u);
  assert.deepEqual(headers.filter(line => line === "parent" || line.startsWith("parent ")), ["parent " + expected]);
}
function execution(stdout: string): void {
  assert.deepEqual([...stdout.matchAll(/^test (\S+) \.\.\. ok$/gm)].map(match => match[1]).sort(), TESTS);
  const summaries = [...stdout.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1); assert.deepEqual(summaries[0]?.slice(1), ["3", "0", "0", "0"]);
}
function configuration(environment: NodeJS.ProcessEnv): void {
  assert.equal(environment["VIZE_TEST_REQUIRE_TSGO"], "1");
  for (const name of ["VIZE_TEST_DISABLE_TSGO", "VIZE_TEST_TSGO_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE", "TSRS_PATH", "TSRS_EXECUTABLE", "VIZE_CORSA_PATH", "VIZE_TSGO_PATH", "VIZE_TSRS_PATH", "VIZE_LSP_BIN", "VIZE_TRACE_EDITOR_PREPARATION"]) assert.equal(environment[name], undefined, name + " must be absent");
}
function closure(candidate: string, previous: string, original: string, workflow: string, sources: string[], tooling: string[]): void {
  parent(candidate, SOURCE_PARENT); parent(previous, SOURCE_GRANDPARENT); parent(original, BASE); parent(workflow, CANDIDATE);
  assert.deepEqual(sources.sort(), SOURCE_PATHS); assert.deepEqual(tooling.sort(), [...WORKFLOW_PATHS].sort());
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
function providerBinding(provider: Record<string, unknown>, actual: Record<string, unknown>): void {
  assert.equal(provider["sourceSha"], CANDIDATE); assert.equal(provider["selection"], "checkout-locked-platform-package");
  const pkg = object(provider["package"]), selected = object(provider["executable"]);
  assert.equal(pkg["name"], "@typescript/typescript-linux-x64"); assert.equal(pkg["version"], "7.0.2");
  assert.equal(actual["path"], selected["path"]); assert.equal(actual["sha256"], selected["sha256"]);
  assert.deepEqual(selected["version"], { args: ["--version"], stdout: "Version 7.0.2\n", stderr: "", exitCode: 0, signal: null });
}
function sameImages(before: unknown, after: unknown): void { assert.deepEqual(after, before); }
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

if (process.argv[2] === "--controls") {
  let passed = 0;
  const check = (action: () => void): void => { action(); passed++; };
  const refuse = (action: () => void): void => check(() => assert.throws(action, assert.AssertionError));
  const good = "running 3 tests\n" + TESTS.map(name => "test " + name + " ... ok\n").join("") + "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n";
  check(() => execution(good));
  for (const bad of ["", good.replace(TESTS[0] ?? "", "foreign"), good.replace("3 passed", "0 passed"), good + good,
    good.replace("0 ignored", "1 ignored"), good.replace(" ... ok", " ... ignored"), good.replace("0 failed", "1 failed")]) refuse(() => execution(bad));
  check(() => configuration({ VIZE_TEST_REQUIRE_TSGO: "1" }));
  for (const bad of [{}, { VIZE_TEST_REQUIRE_TSGO: "0" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_DISABLE_TSGO: "1" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_TSGO_PATH: "/foreign" }, { VIZE_TEST_REQUIRE_TSGO: "1", TSGO_EXECUTABLE: "/foreign" }]) refuse(() => configuration(bad));
  for (const name of ["VIZE_TEST_DISABLE_TSGO", "VIZE_TEST_TSGO_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE", "TSRS_PATH", "TSRS_EXECUTABLE", "VIZE_CORSA_PATH", "VIZE_TSGO_PATH", "VIZE_TSRS_PATH", "VIZE_LSP_BIN", "VIZE_TRACE_EDITOR_PREPARATION"]) refuse(() => configuration({ VIZE_TEST_REQUIRE_TSGO: "1", [name]: "" }));
  const packetRows = CASES.map((row, index) => {
    const packet: unknown = row.private ? { uri: `file:///tmp/definition-case-${index}/App.vue`, range: expectedRange(row.source) } : null;
    return { ...row, case: index, admitted: row.private, old: packet, actual: packet };
  });
  const encode = (rows: unknown[]): string => rows.map(row => "VIZE_PRIVATE_DEFINITION_NATIVE " + JSON.stringify(row)).join("\n");
  check(() => { definitionPairs(encode(packetRows)); });
  const mutation = (index: number, replace: Record<string, unknown>): unknown[] => packetRows.map((row, i) => i === index ? { ...row, ...replace } : row);
  for (const rows of [packetRows.slice(1), [...packetRows, packetRows[0]], mutation(1, packetRows[0]!), mutation(0, { case: 1 }),
    mutation(0, { source: "foreign" }), mutation(0, { id: "foreign" }), mutation(0, { offset: 1 }), mutation(0, { admitted: false }),
    mutation(0, { private: false }), mutation(0, { old: null, actual: null }), mutation(6, { old: "scalar", actual: "scalar" }),
    mutation(0, { old: [packetRows[0]?.old], actual: [packetRows[0]?.actual] }), mutation(0, { extra: true }),
    mutation(0, { old: { uri: "file:///tmp/Other.vue", range: expectedRange(CASES[0]!.source) }, actual: { uri: "file:///tmp/Other.vue", range: expectedRange(CASES[0]!.source) } })]) refuse(() => { definitionPairs(encode(rows)); });
  const missing = packetRows.map((row, i) => { if (i !== 0) return row; const { admitted: _admitted, ...rest } = row; return rest; });
  refuse(() => { definitionPairs(encode(missing)); });
  refuse(() => { definitionPairs(encode(packetRows) + "\nVIZE_PRIVATE_DEFINITION_NATIVE_DISABLED no native credit"); });
  const opaque = { targetUri: "file:///tmp/foreign.ts", extension: { version: 4, flags: [true, null, { opaque: "kept" }] } };
  const expanded = mutation(6, { old: [opaque, { uri: "file:///tmp/other.ts", extra: 9 }], actual: [opaque, { uri: "file:///tmp/other.ts", extra: 9 }] });
  check(() => { definitionPairs(encode(expanded)); });
  for (const actual of [[{ ...opaque, extension: { version: 4, flags: [true, null, { opaque: "changed" }] } }, { uri: "file:///tmp/other.ts", extra: 9 }], [opaque], [{ uri: "file:///tmp/other.ts", extra: 9 }, opaque], { ...opaque }]) {
    const changed = expanded.map((row, i) => i === 6 ? { ...object(row), actual } : row);
    refuse(() => { definitionPairs(encode(changed)); });
  }
  const native = { path: "/native/tsc", bytes: 24, sha256: "a".repeat(64) };
  const provider = { sourceSha: CANDIDATE, selection: "checkout-locked-platform-package", package: { name: "@typescript/typescript-linux-x64", version: "7.0.2" }, executable: { ...native, version: { args: ["--version"], stdout: "Version 7.0.2\n", stderr: "", exitCode: 0, signal: null } } };
  check(() => providerBinding(provider, native));
  for (const value of [{ ...provider, sourceSha: BASE }, { ...provider, selection: "custom" }, { ...provider, package: { name: "foreign", version: "7.0.2" } }, { ...provider, package: { name: "@typescript/typescript-linux-x64", version: "7.0.1" } }]) refuse(() => providerBinding(value, native));
  refuse(() => providerBinding(provider, { ...native, sha256: "b".repeat(64) }));
  refuse(() => providerBinding(provider, { ...native, path: "/foreign/tsc" }));
  check(() => sameImages(native, { ...native })); refuse(() => sameImages(native, { ...native, sha256: "b".repeat(64) }));
  const target = "/owned", repo = "/repo", lib = { reason: "compiler-artifact", package_id: "path+file:///repo/crates/vize_maestro#0.1.0", manifest_path: "/repo/crates/vize_maestro/Cargo.toml", target: { name: "vize_maestro", kind: ["lib"], src_path: "/repo/crates/vize_maestro/src/lib.rs" }, profile: { test: true }, executable: "/owned/ci/deps/vize_maestro-hash", features: ["native"] };
  const sdk = SDK_CRATES.map(name => ({ reason: "compiler-artifact", package_id: `registry+https://github.com/rust-lang/crates.io-index#${name}@2.0.0-beta.4`, target: { name, kind: ["lib"] }, features: [] }));
  const cargoRows = [lib, ...sdk, { reason: "build-finished", success: true }];
  check(() => { libraryArtifact(cargoRows, target, repo); });
  check(() => { libraryArtifact([...cargoRows, sdk[0]!], target, repo); });
  for (const rows of [cargoRows.slice(1), [...cargoRows, lib], [{ ...lib, executable: "/foreign/lib" }, ...sdk, { reason: "build-finished", success: true }], [{ ...lib, manifest_path: "/foreign/Cargo.toml" }, ...sdk, { reason: "build-finished", success: true }], [{ ...lib, features: ["native", "profiling"] }, ...sdk, { reason: "build-finished", success: true }], [lib, ...sdk.slice(1), { reason: "build-finished", success: true }]]) refuse(() => { libraryArtifact(rows, target, repo); });
  const tree = "tree " + "b".repeat(40), sourceRaw = tree + "\nparent " + SOURCE_PARENT + "\n\nfixture", previousRaw = tree + "\nparent " + SOURCE_GRANDPARENT + "\n\nfixture", originalRaw = tree + "\nparent " + BASE + "\n\nfixture", workflowRaw = tree + "\nparent " + CANDIDATE + "\n\nfixture";
  check(() => closure(sourceRaw, previousRaw, originalRaw, workflowRaw, [...SOURCE_PATHS], [...WORKFLOW_PATHS]));
  for (const raw of [tree + "\nparent other\n\nfixture", tree + "\nparent " + BASE + "\nparent " + BASE + "\n\nfixture", tree + "\nparent\n\nfixture", tree + "\n\nroot", tree + "\nparent " + BASE]) refuse(() => parent(raw, BASE));
  refuse(() => closure(sourceRaw, previousRaw, originalRaw, sourceRaw, [...SOURCE_PATHS], [...WORKFLOW_PATHS]));
  refuse(() => closure(sourceRaw, previousRaw, originalRaw, workflowRaw, SOURCE_PATHS.slice(1), [...WORKFLOW_PATHS]));
  refuse(() => closure(sourceRaw, previousRaw, originalRaw, workflowRaw, [...SOURCE_PATHS], [...WORKFLOW_PATHS, "foreign"]));
  refuse(() => closure(sourceRaw, sourceRaw, originalRaw, workflowRaw, [...SOURCE_PATHS], [...WORKFLOW_PATHS]));
  refuse(() => closure(sourceRaw, previousRaw, previousRaw, workflowRaw, [...SOURCE_PATHS], [...WORKFLOW_PATHS]));
  const file = "pnpm-lock.yaml", size = Number(git("cat-file", "-s", `${CANDIDATE}:${file}`));
  assert(size > 1024 * 1024 && size <= 4 * 1024 * 1024);
  const actual = execFileSync("git", ["show", `${CANDIDATE}:${file}`], { maxBuffer: size + 1 });
  check(() => { completeBlob(CANDIDATE, file, actual); });
  const corrupted = Buffer.from(actual); corrupted[0] = (corrupted.at(0) ?? 0) ^ 1;
  refuse(() => { completeBlob(CANDIDATE, file, corrupted); });
  refuse(() => { completeBlob(CANDIDATE, file, actual.subarray(1)); });
  check(() => assert.equal(hash(execFileSync("git", ["show", `${CANDIDATE}:${SOURCE_PATHS[1]}`], { maxBuffer: 1024 * 1024 })), FIXTURE_SHA256));
  process.stdout.write(JSON.stringify({ passed, qualificationCredit: 0, corpus: 34, positives: 6, tests: 3, completeBlobBytes: size, completeBlobSha256: hash(actual) }) + "\n");
} else {
  const source = env("SOURCE_SHA"), workflow = env("WORKFLOW_SHA"), evidence = fs.realpathSync(env("EVIDENCE"));
  assert.equal(source, CANDIDATE); assert.match(workflow, /^[a-f0-9]{40}$/u); configuration(process.env);
  assert.equal(process.version, "v24.14.0"); assert.equal(process.platform, "linux");
  assert.equal(git("rev-parse", "HEAD"), source); assert.equal(env("EXECUTED_WORKFLOW_SHA"), workflow);
  assert.deepEqual(fs.readFileSync(text(process.argv[1])), execFileSync("git", ["show", `${workflow}:.github/actions/lsp-private-rename-parity/run.ts`]));
  closure(git("cat-file", "-p", source), git("cat-file", "-p", SOURCE_PARENT), git("cat-file", "-p", SOURCE_GRANDPARENT), git("cat-file", "-p", workflow), git("diff", "--name-only", BASE, source).split("\n"), git("diff", "--name-only", source, workflow).split("\n"));
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert.equal(env("CARGO_INCREMENTAL"), "0");
  function save(name: string, value: unknown): void {
    fs.writeFileSync(path.join(evidence, name + ".json"), JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 });
  }
  function image(file: string) {
    const realpath = fs.realpathSync(file), before = fs.statSync(realpath, { bigint: true });
    assert(before.isFile()); const bytes = fs.readFileSync(realpath), after = fs.statSync(realpath, { bigint: true });
    assert.equal(before.ino, after.ino); assert.equal(before.size, after.size); assert.equal(before.mtimeNs, after.mtimeNs);
    assert.equal(bytes.subarray(0, 4).toString("hex"), "7f454c46");
    return { path: realpath, bytes: bytes.length, sha256: hash(bytes) };
  }
  function run(name: string, executable: string, args: string[], environment = process.env) {
    const out = fs.openSync(path.join(evidence, name + ".stdout"), "wx", 0o600);
    const err = fs.openSync(path.join(evidence, name + ".stderr"), "wx", 0o600);
    try {
      const result = spawnSync(executable, args, { env: environment, stdio: ["ignore", out, err] });
      const status = { executable, args, code: result.status, signal: result.signal, error: result.error?.message ?? null };
      save(name + ".exit", status); return status;
    } finally { fs.closeSync(out); fs.closeSync(err); }
  }
  const provider = object(JSON.parse(fs.readFileSync("target/nextest/full/native-runtime-provider.json", "utf8")));
  const nativeImage = image(env("CORSA_PATH")); providerBinding(provider, nativeImage);
  for (const file of [...SDK_PATHS, ...REUSED_PATHS]) assert.equal(git("rev-parse", `${BASE}:${file}`), git("rev-parse", `${source}:${file}`));
  const bindingPaths = [...SOURCE_PATHS, ...REUSED_PATHS, ...SDK_PATHS, "crates/vize_maestro/Cargo.toml", "crates/vize_canon/Cargo.toml", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", "npm/cli/package.json", "npm/cli/scripts/generate-content-mapper-types.ts", "crates/vize_maestro/src/test_support.rs", "tools/support/compat/github/full-rust-runtime.ts"];
  const sourceBindings = () => bindingPaths.map(file => {
    const bytes = fs.readFileSync(file); completeBlob(source, file, bytes);
    return { path: file, sha256: hash(bytes) };
  });
  const bindings = sourceBindings();
  assert.equal(hash(fs.readFileSync(SOURCE_PATHS[1]!)), FIXTURE_SHA256);
  save("before", { source, workflow, base: BASE, sourceParent: SOURCE_PARENT, bindings, nativeImage, provider,
    sourceTree: git("rev-parse", source + "^{tree}"), rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    cargo: execFileSync("cargo", ["-V"], { encoding: "utf8" }), qualificationCredit: 0 });
  assert.ok(!fs.existsSync(path.join(env("CARGO_TARGET_DIR"), "ci")), "owned ci build outputs must initially be absent");
  const built = run("build", "cargo", ["test", "--locked", "--profile", "ci", "-p", "vize_maestro", "--lib", PREFIX,
    "--no-run", "--message-format=json-render-diagnostics"], process.env);
  assert.deepEqual([built.code, built.signal, built.error], [0, null, null]);
  const rows = fs.readFileSync(path.join(evidence, "build.stdout"), "utf8").split("\n").filter(Boolean).map(line => object(JSON.parse(line)));
  const library = libraryArtifact(rows, fs.realpathSync(env("CARGO_TARGET_DIR")), fs.realpathSync(process.cwd()));
  const libraryImage = image(text(library["executable"])); save("library", { source, artifact: library, libraryImage });
  const relativePhysical = path.relative(fs.realpathSync(env("CARGO_TARGET_DIR")), libraryImage.path);
  assert.ok(relativePhysical.startsWith("ci/deps/") && !relativePhysical.split(path.sep).includes(".."));
  const definitionResult = run("test-definition", libraryImage.path, [PREFIX, "--nocapture", "--test-threads=1"], process.env);
  const afterBindings = sourceBindings();
  save("after", { source, workflow, nativeImage: image(env("CORSA_PATH")), libraryImage: image(libraryImage.path), bindings: afterBindings, results: { definitionResult }, qualificationCredit: 0 });
  sameImages(nativeImage, image(env("CORSA_PATH"))); sameImages(libraryImage, image(libraryImage.path)); assert.deepEqual(afterBindings, bindings);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert.deepEqual([definitionResult.code, definitionResult.signal, definitionResult.error], [0, null, null]);
  execution(fs.readFileSync(path.join(evidence, "test-definition.stdout"), "utf8"));
  const nativeDefinitionPairs = definitionPairs(fs.readFileSync(path.join(evidence, "test-definition.stderr"), "utf8"));
  const sdkArtifacts = rows.filter(row => row["reason"] === "compiler-artifact" && SDK_CRATES.includes(text(object(row["target"])["name"])));
  save("parity", { source, workflow, selectedTests: TESTS, executed: 3, ignored: 0, completeWholeDefinitionPairs: nativeDefinitionPairs, sdkArtifacts, nativeImage, libraryImage,
    scope: "finite pinned native source unit parity only; earlier rename13/prepare32 remain archived, no new CLI/performance/public acceptance" });
}
