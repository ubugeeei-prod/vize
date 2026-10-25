import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { nativeShutdownProtocolSource } from "../../tools/support/compat/github/native-shutdown-protocol-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = new URL("../../", import.meta.url);
const source = "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown";
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(readFileSync(new URL(manifestPath, root), "utf8"));
const row = manifest.protocolCases[0];
const rawPath = `tests/_fixtures/differential/typechecker/${row.inputs[0].path}`;
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function analyze({
  paths = [`${source}.rs`, `${source}/completion.rs`, `${source}/peer.ts.txt`],
  rows = [row],
  previous = [],
  cases = [],
  files = {},
}: {
  paths?: string[];
  rows?: unknown;
  previous?: unknown;
  cases?: object[];
  files?: Record<string, Buffer | null>;
} = {}) {
  return analyzeLegacyFix({
    title: "fix(canon): complete native shutdown",
    paths,
    readFile: (side, path) => {
      if (path === manifestPath)
        return Buffer.from(
          JSON.stringify({
            ...manifest,
            cases: side === "base" ? [] : cases,
            protocolCases: side === "base" ? previous : rows,
          }),
        );
      if (side !== "head") return null;
      if (Object.hasOwn(files, path)) return files[path] ?? null;
      try {
        return readFileSync(new URL(path, root));
      } catch {
        return null;
      }
    },
  }).products[0];
}

void test("the actual protocol registration pins full wire selectors without changing batch accounting", () => {
  assert.deepEqual(analyze(), {
    product: "typechecker",
    state: "new-inputs-registered",
    cases: [row.id],
  });
  const loaded = loadTypecheckerManifest(new URL(manifestPath, root).pathname);
  assert.equal(loaded.cases.length, manifest.cases.length);
  assert(!loaded.cases.some((entry) => entry.id === row.id));
});

void test("the protocol source boundary admits only ordinary owned modules and the exact wire support fixture", () => {
  for (const path of [
    `${source}.rs`,
    `${source}/completion.rs`,
    `${source}/tests/completion.rs`,
    `${source}/peer.ts.txt`,
  ]) {
    assert(nativeShutdownProtocolSource(path), path);
  }
  for (const path of [
    `${source}_other.rs`,
    `${source}/../outside.rs`,
    `${source}/other.ts.txt`,
    `${source}/completion.json`,
    `${source}/nested/../../outside.rs`,
    "crates/vize_canon/src/check.rs",
  ]) {
    assert(!nativeShutdownProtocolSource(path), path);
    assert.equal(analyze({ paths: [path] })?.state, "new-input-missing", path);
  }
});

void test("mixed Canon changes still need newly registered ordinary batch inputs", () => {
  const paths = [`${source}.rs`, "crates/vize_canon/src/batch.rs"];
  assert.equal(analyze({ paths })?.state, "new-input-missing");
  const input = Buffer.from("<template><div /></template>\n");
  const file = "native-shutdown-test/App.vue.txt";
  const batch = {
    id: "typechecker/native-shutdown-test/default",
    state: "active",
    targets: ["batch"],
    inputs: [{ path: file, sha256: hash(input) }],
  };
  assert.equal(
    analyze({
      paths,
      cases: [batch],
      files: { [`tests/_fixtures/differential/typechecker/${file}`]: input },
    })?.state,
    "new-inputs-registered",
  );
  assert.equal(
    analyze({
      paths,
      rows: [],
      cases: [batch],
      files: { [`tests/_fixtures/differential/typechecker/${file}`]: input },
    })?.state,
    "new-input-missing",
  );
});

void test("stale, inactive, duplicate and unregistered protocol identities do not qualify", () => {
  assert.equal(analyze({ previous: [row] })?.state, "new-input-missing");
  assert.equal(analyze({ rows: [] })?.state, "new-input-missing");
  for (const rows of [
    [{ ...row, state: "draft" }],
    [row, row],
    [{ ...row, id: `${row.id}-forged` }],
    [{ ...row, adapter: "other" }],
    [{ ...row, dialectCoverage: { state: "verified" } }],
  ]) {
    assert.equal(analyze({ rows })?.state, "invalid-manifest-or-input");
  }
});

void test("empty, malformed and hash-drifted raw envelope packs fail closed", () => {
  assert.equal(analyze({ files: { [rawPath]: null } })?.state, "invalid-manifest-or-input");
  assert.equal(
    analyze({ files: { [rawPath]: Buffer.from("drift") } })?.state,
    "invalid-manifest-or-input",
  );
  for (const value of [
    "[]",
    "null",
    "{",
    '[{"input":null,"expected":{"kind":"complete"}}]',
    '[{"input":null,"expected":{"kind":"forged"}}]',
  ]) {
    const bytes = Buffer.from(value);
    const changed = structuredClone(row);
    changed.inputs[0].sha256 = hash(bytes);
    assert.equal(
      analyze({ rows: [changed], files: { [rawPath]: bytes } })?.state,
      "invalid-manifest-or-input",
    );
  }
  assert.equal(analyze({ rows: [{ ...row, inputs: [] }] })?.state, "invalid-manifest-or-input");
});

void test("missing or forged wire selectors, source bodies and module registration cannot qualify", () => {
  for (const change of [
    (value) => value.tests.pop(),
    (value) => (value.tests[1].selector += "_forged"),
    (value) => (value.tests[1].source = `${source}/other.rs`),
    (value) => (value.tests[1].sha256 = "0".repeat(64)),
  ]) {
    const changed = structuredClone(row);
    change(changed);
    assert.equal(analyze({ rows: [changed] })?.state, "invalid-manifest-or-input");
  }
  const registered = row.tests[1];
  const bytes = Buffer.from("// no registered test body\n");
  const changed = structuredClone(row);
  for (const item of changed.tests)
    if (item.source === registered.source) item.sha256 = hash(bytes);
  assert.equal(
    analyze({ rows: [changed], files: { [registered.source]: bytes } })?.state,
    "invalid-manifest-or-input",
  );
  assert.equal(
    analyze({ files: { [`${source}/completion.rs`]: Buffer.from("// no mod tests;\n") } })?.state,
    "invalid-manifest-or-input",
  );
});
