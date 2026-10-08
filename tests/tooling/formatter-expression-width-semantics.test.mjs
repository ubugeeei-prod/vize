import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { sha256 } from "../differential/manifest.mjs";
import {
  observeSource,
  providerIdentities,
  validateProviders,
} from "./support/formatter-expression-width-semantics.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876",
);
const corpusBytes = fs.readFileSync(path.join(fixtureRoot, "corpus.json"));
assert.equal(
  sha256(corpusBytes),
  "b986b95eecfa3f2df1d653695f27dc330f3229b22ac7c335c15a2ecb4bb44523",
);
const corpus = JSON.parse(corpusBytes);
assert.equal(corpus.cases.length, 16);
const contractBytes = fs.readFileSync(path.join(fixtureRoot, "stock-contracts.json"));
assert.equal(
  sha256(contractBytes),
  "eb1abce7663daaddbecfe4364eae8f164ade1d30bd24f2865becdcdd36b20217",
);
const contracts = JSON.parse(contractBytes);
assert.equal(contracts.schema, "vize.independent-expression-width-semantics");
assert.equal(contracts.version, 1);
assert.equal(contracts.corpusSha256, sha256(corpusBytes));
assert.equal(contracts.rows.length, 16);
assert.deepEqual(
  contracts.rows.map((row) => row.id),
  corpus.cases.map((row) => row.id),
);
assert.equal(new Set(contracts.rows.map((row) => row.id)).size, 16);
validateProviders(contracts);

const selectedCases = corpus.cases.map((fixture, index) => {
  const contract = contracts.rows[index];
  const read = (relative) => {
    const file = fs.realpathSync(path.join(fixtureRoot, relative));
    assert(file.startsWith(fs.realpathSync(fixtureRoot) + path.sep));
    return fs.readFileSync(file);
  };
  const input = read(fixture.input);
  const expected = read(fixture.output);
  assert.equal(sha256(input), fixture.inputSha256);
  assert.equal(sha256(expected), fixture.expectedSha256);
  assert.equal(contract.inputSha256, fixture.inputSha256);
  assert.equal(contract.expectedSha256, fixture.expectedSha256);
  assert.deepEqual(
    contract.observations.map((row) => row.state),
    [false, true],
  );
  return { fixture, contract, input, expected };
});

void test("independent whole references retain all stock DOM SSR calls and click observations", async () => {
  let states = 0;
  for (const { fixture, contract, input, expected } of selectedCases) {
    for (const [label, bytes] of [
      ["input", input],
      ["reference", expected],
    ]) {
      const observed = await observeSource(bytes.toString(), `${fixture.id}-${label}`, [
        false,
        true,
      ]);
      assert.deepEqual(observed.observations, contract.observations, `${fixture.id}: ${label}`);
      states += observed.observations.length;
    }
  }
  assert.equal(states, 64);
});

void test("source CLI preserves eighty complete processes and stock semantics at three fixed points", async (t) => {
  const identity = expectedBuildIdentity(root);
  const buildReceipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(buildReceipt, identity);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-expression-width-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const report = {
    schema: "vize.formatter-expression-width-observation",
    version: 1,
    buildReceipt,
    corpusSha256: sha256(corpusBytes),
    contractsSha256: sha256(contractBytes),
    providerIdentities,
    nativeSupported: false,
    nativeHandled: 0,
    nativeEquivalent: 0,
    rows: [],
  };
  const reportPath = path.join(
    root,
    "target/differential/formatter-expression-width-semantics.json",
  );
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  persist();
  for (const { fixture, contract, input, expected } of selectedCases) {
    await t.test(fixture.id, async () => {
      const row = { id: fixture.id, cli: [], after: [] };
      report.rows.push(row);
      const file = path.join(directory, "Example.vue");
      const configPath = path.join(directory, "vize.config.json");
      const config = Buffer.from(`${JSON.stringify({ formatter: fixture.options })}\n`);
      fs.writeFileSync(file, input);
      fs.writeFileSync(configPath, config);
      for (const [pass, mode] of [
        "--check",
        "--write",
        "--write",
        "--write",
        "--check",
      ].entries()) {
        const before = fs.readFileSync(file);
        const args = ["fmt", mode, "Example.vue"];
        const result = spawnSync(path.join(root, identity.binaryPath), args, {
          cwd: directory,
          timeout: 30000,
          maxBuffer: 1048576,
        });
        const after = fs.readFileSync(file);
        const configuration = fs.readFileSync(configPath);
        row.cli.push({
          args,
          cwd: directory,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout?.toString() ?? "",
          stderr: result.stderr?.toString() ?? "",
          stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
          stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
          before: before.toString(),
          beforeBase64: before.toString("base64"),
          after: after.toString(),
          afterBase64: after.toString("base64"),
          configuration: configuration.toString(),
          configurationBase64: configuration.toString("base64"),
        });
        persist();
        const changed = pass < 2 && !input.equals(expected);
        const checked = mode === "--check";
        const detail = changed
          ? `${checked ? "Would reformat" : "Reformatted"}: Example.vue\n`
          : "";
        const outcome = checked
          ? changed
            ? "would be reformatted"
            : "already formatted"
          : changed
            ? "reformatted"
            : "unchanged";
        const stderr = `Found 1 file(s)\n${detail}\n${checked ? "Checked" : "Formatted"} 1 file(s)\n  1 file(s) ${outcome}\n`;
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(result.status, Number(checked && changed), result.stderr?.toString());
        assert.deepEqual(result.stdout, Buffer.alloc(0));
        assert.deepEqual(result.stderr, Buffer.from(stderr));
        if (checked) assert.deepEqual(after, before);
        assert.deepEqual(after, pass === 0 ? input : expected);
        assert.deepEqual(configuration, config);
        const observed = await observeSource(after.toString(), `${fixture.id}-pass${pass}`, [
          false,
          true,
        ]);
        row.after.push(observed);
        persist();
        assert.deepEqual(observed.observations, contract.observations);
      }
    });
  }
  assert.equal(report.rows.length, 16);
  assert.equal(
    report.rows.reduce((sum, row) => sum + row.cli.length, 0),
    80,
  );
  assert.equal(
    report.rows.reduce((sum, row) => sum + row.after.length, 0),
    80,
  );
  assert.equal(
    report.rows.reduce(
      (sum, row) => sum + row.after.reduce((n, after) => n + after.observations.length, 0),
      0,
    ),
    160,
  );
});
