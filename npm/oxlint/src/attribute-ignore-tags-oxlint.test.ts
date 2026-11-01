import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mts";
import { asArray, asRecord, asString, parseJson } from "../../../tools/support/json-value.ts";
import { presentation } from "../../../tests/tooling/support/patina-attribute-ignore-contract.ts";
import {
  hash,
  literalRows,
  original,
  root,
  rowFor,
  rule,
  type ReferenceTagRow,
} from "../../../tests/tooling/support/patina-attribute-ignore-tags-contract.ts";
import { resetFixtureDir } from "./test-support/fixture-dir.ts";

const nativeDir = path.join(root, "npm/native");
const receiptBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
const receipt = asRecord(parseJson(receiptBytes.toString()));
validateNativeHistoryBuild(nativeDir, receipt);
if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
const image = fs.realpathSync(asString(asRecord(receipt.frozen).path));
const require = createRequire(path.join(root, "package.json"));
assert.equal(asRecord(require("oxlint/package.json")).version, "1.78.0");
const binary = path.join(path.dirname(require.resolve("oxlint/package.json")), "bin/oxlint");
const directory = path.join(root, "target/vize-tests/attribute-ignore-tags-oxlint");
resetFixtureDir(directory);
const packets: unknown[] = [];
const custody = {
  source: receipt.source,
  frozen: receipt.frozen,
  receiptSha256: hash(receiptBytes),
  providers: { oxlint: "1.78.0" },
  referenceProviders: original.versions,
  pluginSha256: hash(fs.readFileSync(path.join(root, "npm/oxlint/dist/index.mjs"))),
};
const proofPath = path.join(nativeHistoryDirectory(nativeDir), "attribute-ignore-tags-oxlint.json");
function persist(complete = false) {
  fs.writeFileSync(
    proofPath,
    JSON.stringify(
      {
        schema: "vize.attribute-ignore-tags.oxlint.v1",
        complete,
        custody,
        packets,
        limits:
          "Seven unchanged registered-script cases only; complete raw subprocesses and whole retained diagnostic objects. No direct-host precise template-range/fixer, scriptless-host, worker-image-identity or public artifact qualification.",
      },
      null,
      2,
    ) + "\n",
  );
}
function observe(row: ReferenceTagRow, options: unknown[]) {
  const sourcePath = path.join(directory, row.filename),
    configPath = path.join(directory, ".oxlintrc.json");
  const config = {
    plugins: ["vue"],
    jsPlugins: [path.join(root, "npm/oxlint/dist/index.mjs")],
    settings: { vize: { helpLevel: "none", preset: "incremental" } },
    rules: { "no-unused-vars": "off", ["vize/" + rule]: ["warn", ...options] },
  };
  fs.writeFileSync(sourcePath, row.source);
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + "\n");
  const args = ["--config", configPath, "--format", "json", row.filename];
  const result = spawnSync(binary, args, {
    cwd: directory,
    env: { ...process.env, VIZE_PREFER_WORKSPACE_BINDING: "1", NAPI_RS_NATIVE_LIBRARY_PATH: image },
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
  });
  const packet = {
    id: row.id,
    source: row.source,
    sourceSha256: hash(row.source),
    config,
    command: [binary, ...args],
    cwd: directory,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout,
    stderr: result.stderr,
    fileAfter: fs.readFileSync(sourcePath, "utf8"),
  };
  packets.push(packet);
  persist();
  console.log("vize-attribute-ignore-tags-oxlint " + JSON.stringify(packet));
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(packet.fileAfter, row.source);
  const payload = asRecord(parseJson(result.stdout));
  const diagnostics = asArray(payload.diagnostics).map((value) => asRecord(value));
  assert.deepEqual(
    diagnostics.map((finding) => ({
      code: finding.code,
      message: finding.message,
      severity: finding.severity,
      filename: finding.filename,
    })),
    row.verify[0]!.messages.map((finding) => ({
      code: "vize(vue/attribute-hyphenation)",
      severity: "warning",
      filename: row.filename,
      message: `${presentation(finding).message} (at <template>:${finding.line}:${finding.column})`,
    })),
    row.id,
  );
  return diagnostics;
}
const baselines = new Map(
  ["always", "never"].map((style) => {
    const row = rowFor(style === "always" ? "camel-empty" : "never-empty");
    return [style, { row, diagnostics: observe(row, [style, { ignoreTags: [] }]) }] as const;
  }),
);
const rows = literalRows.filter((row) => row.file.includes("/attribute-hyphenation-ignore-6097/"));
assert.equal(rows.length, 7);
for (const row of rows) {
  const baseline = baselines.get(row.style);
  assert(baseline);
  const indices = row.verify[0]!.messages.map((finding) => {
    const index = baseline.row.verify[0]!.messages.findIndex(
      (original) => JSON.stringify(original) === JSON.stringify(finding),
    );
    assert(index >= 0);
    return index;
  });
  const expected = indices.map((index) => baseline.diagnostics[index]);
  for (const options of [
    [row.style, { ignoreTags: row.ignoreTags }],
    [{ style: row.style, ignoreTags: row.ignoreTags }],
  ])
    assert.deepEqual(observe(row, options), expected, row.id);
}
assert.equal(packets.length, 16);
persist(true);
