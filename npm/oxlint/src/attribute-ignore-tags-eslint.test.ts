import assert from "node:assert/strict";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mts";
import { asRecord, asString, parseJson } from "../../../tools/support/json-value.ts";
import {
  original,
  literalRows,
  expected,
  hash,
  root,
  rule,
  sourceFor,
  type ReferenceTagRow,
} from "../../../tests/tooling/support/patina-attribute-ignore-tags-contract.ts";

interface Host {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
const directory = join(root, "npm/native");
const receiptBytes = readFileSync(nativeHistoryReceipt(directory));
const receipt = asRecord(parseJson(receiptBytes.toString()));
validateNativeHistoryBuild(directory, receipt);
if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
const image = realpathSync(asString(asRecord(receipt.frozen).path));
const require = createRequire(import.meta.url);
const keys = [
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
] as const;
const previous = keys.map((key) => [key, process.env[key]] as const);
const observations: unknown[] = [];

function messages(row: ReferenceTagRow): unknown[] {
  const source = sourceFor(row);
  const utf16 = (offset: number) => Buffer.from(source).subarray(0, offset).toString().length;
  return expected(row).diagnostics.map((diagnostic) => {
    const edits = diagnostic.fix!.edits;
    assert.equal(edits.length, 1);
    const edit = edits[0]!;
    return {
      ruleId: "vize/" + rule,
      severity: 1,
      message: diagnostic.message,
      line: diagnostic.location.start.line,
      column: diagnostic.location.start.column,
      endLine: diagnostic.location.end.line,
      endColumn: diagnostic.location.end.column,
      fix: { range: [utf16(edit.start), utf16(edit.end)], text: edit.newText },
    };
  });
}

try {
  process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  delete process.env.NAPI_RS_FORCE_WASI;
  const entry: unknown = await import(new URL("../dist/eslint.mjs", import.meta.url).href);
  const plugin = asRecord(entry).default;
  assert(require.cache[image], "the packed public export must load the authenticated source addon");
  const workspace = createRequire(new URL("../../../package.json", import.meta.url));
  const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const Linter = asRecord(oracle("eslint")).Linter as new () => Host;
  const parser: unknown = oracle("vue-eslint-parser");
  // The declared Nuxt consumer resolves ESLint 10.13.0 in the current lock.
  // The frozen independent reference used 10.4.1; retain that separate authority.
  const targetProviders = { eslint: "10.13.0", "vue-eslint-parser": "10.4.1" };
  const providers = Object.fromEntries(
    Object.entries(targetProviders).map(([name, targetVersion]) => {
      const version = asRecord(oracle(name + "/package.json")).version;
      assert.equal(version, targetVersion);
      return [name, { version, path: realpathSync(oracle.resolve(name + "/package.json")) }];
    }),
  );
  const custody = {
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    exportSha256: hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url))),
    providers,
    referenceProviders: original.versions,
    sourceFiles: Object.fromEntries(
      ["eslint.ts", "rule-options.ts", "attribute-options.ts"].map((path) => [
        path,
        hash(readFileSync(new URL(path, import.meta.url))),
      ]),
    ),
  };
  console.log("vize-attribute-ignore-tags-eslint-source " + JSON.stringify(custody));
  const host = new Linter();
  for (const row of literalRows) {
    const original = sourceFor(row),
      reference = row;
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: { parser },
        plugins: { vize: plugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules: { ["vize/" + rule]: ["warn", { style: row.style, ignoreTags: row.ignoreTags }] },
      },
    ];
    const filename = { filename: row.filename };
    const verify = host.verify(original, config, filename);
    const fixResult = host.verifyAndFix(original, config, filename);
    const output = asString(asRecord(fixResult).output);
    const requery = host.verify(output, config, filename);
    const secondFixResult = host.verifyAndFix(output, config, filename);
    const packet = { id: row.id, original, verify, fixResult, requery, secondFixResult };
    observations.push(packet);
    console.log("vize-attribute-ignore-tags-eslint-host " + JSON.stringify(packet));
    assert.deepEqual(verify, messages(row), row.id);
    assert.deepEqual(
      fixResult,
      {
        fixed: row.verify[0].messages.length > 0,
        messages: [],
        output: reference.fixedSource,
      },
      row.id,
    );
    assert.deepEqual(requery, [], row.id);
    assert.deepEqual(
      secondFixResult,
      { fixed: false, messages: [], output: reference.fixedSource },
      row.id,
    );
  }
  const proof = {
    schema: "vize.patina.attribute-ignore-tags-eslint-host.v1",
    complete: true,
    node: process.version,
    ...custody,
    observations,
    limits:
      "Sixteen independent literal tag SFC cases through the actual packed ESLint export. Existing Vize name-only fixes retain authored directive prefixes; complete fixed source and both subsequent queries are compared to the frozen official reference.",
  };
  writeFileSync(
    join(nativeHistoryDirectory(directory), "attribute-ignore-tags-eslint-host.json"),
    JSON.stringify(proof, null, 2) + "\n",
    { flag: "wx" },
  );
  console.log("vize-attribute-ignore-tags-eslint-custody " + JSON.stringify(proof));
} finally {
  for (const [key, value] of previous) {
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}
