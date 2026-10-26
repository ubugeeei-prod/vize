import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = new URL("../../", import.meta.url);
const source = "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown";
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(readFileSync(new URL(manifestPath, root), "utf8"));
type ProtocolCase = {
  id: string;
  inputs: Array<{ path: string; sha256: string }>;
  tests: Array<{ selector: string; source: string; sha256: string }>;
};
const row = manifest.protocolCases.find(
  (entry: ProtocolCase) => entry.id === "typechecker/protocol/native-beta-provider-pairs-10741",
) as ProtocolCase;
assert(row);
const rawPath = `tests/_fixtures/differential/typechecker/${row.inputs[0]!.path}`;

function analyze({
  paths = [`${source}.rs`, `${source}/beta_contract.rs`],
  current = row,
  previous = [],
  files = {},
}: {
  paths?: string[];
  current?: ProtocolCase;
  previous?: ProtocolCase[];
  files?: Record<string, Buffer | null>;
} = {}) {
  return analyzeLegacyFix({
    title: "fix(canon): bind pinned provider lifecycle pairs",
    paths,
    readFile: (side, path) => {
      if (path === manifestPath)
        return Buffer.from(
          JSON.stringify({ ...manifest, protocolCases: side === "base" ? previous : [current] }),
        );
      if (side === "base") return null;
      if (Object.hasOwn(files, path)) return files[path] ?? null;
      try {
        return readFileSync(new URL(path, root));
      } catch {
        return null;
      }
    },
  }).products[0];
}

void test("exact beta lifecycle protocol custody remains outside batch and dialect counts", () => {
  assert.equal(analyze()?.state, "new-inputs-registered");
  assert.deepEqual(analyze()?.cases, [row.id]);
  assert.equal(
    loadTypecheckerManifest(new URL(manifestPath, root).pathname).cases.length,
    manifest.cases.length,
  );
  assert.equal(analyze({ previous: [row] })?.state, "new-input-missing");
  assert.equal(
    analyze({ paths: [`${source}.rs`, "crates/vize_canon/src/batch/executor.rs"] })?.state,
    "new-input-missing",
  );
});

void test("beta custody rejects missing whole controls, forged selectors and absent ordinary modules", () => {
  for (const change of [
    (value: ProtocolCase) => value.inputs.pop(),
    (value: ProtocolCase) => value.tests.pop(),
    (value: ProtocolCase) => {
      value.tests[0]!.selector += "_forged";
    },
    (value: ProtocolCase) => {
      value.tests[0]!.sha256 = "0".repeat(64);
    },
  ]) {
    const current = structuredClone(row);
    change(current);
    assert.equal(analyze({ current })?.state, "invalid-manifest-or-input");
  }
  assert.equal(
    analyze({ files: { [`${source}/tests.rs`]: Buffer.from("// no module registration\n") } })
      ?.state,
    "invalid-manifest-or-input",
  );
  for (const value of [[], null, [{ id: "forged" }]]) {
    const bytes = Buffer.from(JSON.stringify(value));
    const current = structuredClone(row);
    current.inputs[0]!.sha256 = createHash("sha256").update(bytes).digest("hex");
    assert.equal(
      analyze({ current, files: { [rawPath]: bytes } })?.state,
      "invalid-manifest-or-input",
    );
  }
});
