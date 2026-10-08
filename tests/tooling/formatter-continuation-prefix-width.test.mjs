import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/manifest.mjs";
import { observeSource, providerIdentities } from "./support/formatter-continuation-semantics.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876",
);
const indexBytes = fs.readFileSync(path.join(fixtureRoot, "contracts-index.json"));
assert.equal(
  sha256(indexBytes),
  "cf6a10c97dcf83c411816432be301e3f30ba7f43cd406fbf2a6d891bcb33ddd1",
);
const index = JSON.parse(indexBytes);
assert.equal(index.contracts.length, 10);
const corpusBytes = fs.readFileSync(path.join(fixtureRoot, "corpus.json"));
assert.equal(
  sha256(corpusBytes),
  "f120d2fb5696bdd0c6dc9efad9d3a91e5ddd54bb97db3ad38d58a7a3c4a08457",
);
const corpus = JSON.parse(corpusBytes);
assert.equal(corpus.cases.length, 11);

void test("source CLI keeps complete authored DOM text SSR and event contracts", async (t) => {
  const identity = expectedBuildIdentity(root);
  const buildReceipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(buildReceipt, identity);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-continuation-prefix-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const report = {
    schema: "vize.formatter-continuation-prefix-observation",
    version: 1,
    buildReceipt,
    corpusSha256: sha256(corpusBytes),
    contractsSha256: sha256(indexBytes),
    providerIdentities,
    nativeSupported: false,
    nativeHandled: 0,
    nativeEquivalent: 0,
    rows: [],
    configuredCli: [],
  };
  const reportPath = path.join(
    root,
    "target/differential/formatter-continuation-prefix-width.json",
  );
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  for (const selected of index.contracts) {
    await t.test(selected.id, async () => {
      const bytes = fs.readFileSync(path.join(fixtureRoot, selected.file));
      assert.equal(selected.file, `${selected.id}.stock-contract.json`);
      assert.equal(sha256(bytes), selected.sha256);
      const contract = JSON.parse(bytes);
      assert.equal(contract.case, selected.id);
      assert.equal(contract.inputSha256, selected.inputSha256);
      assert.equal(contract.expectedSourceSha256, selected.expectedSourceSha256);
      assert.equal(contract.observations.length, 7);
      const fixture = corpus.cases.find((item) => item.id === selected.id);
      assert(fixture);
      const input = fs.readFileSync(path.join(fixtureRoot, fixture.input));
      const expected = fs.readFileSync(path.join(fixtureRoot, fixture.output));
      assert.equal(sha256(input), selected.inputSha256);
      assert.equal(sha256(expected), selected.expectedSourceSha256);
      const states = contract.observations.map((item) => item.state);
      const before = await observeSource(input.toString(), `${selected.id}-original`, states);
      const reference = await observeSource(
        expected.toString(),
        `${selected.id}-reference`,
        states,
      );
      assert.deepEqual(before.observations, contract.observations);
      assert.deepEqual(reference.observations, contract.observations);
      const row = { id: selected.id, before, reference, cli: [], after: [] };
      report.rows.push(row);
      persist();
      const file = path.join(directory, "Example.vue");
      const configPath = path.join(directory, "vize.config.json");
      const config = Buffer.from(`${JSON.stringify({ formatter: fixture.options })}\n`);
      fs.writeFileSync(file, input);
      fs.writeFileSync(configPath, config);
      for (const mode of ["--check", "--write", "--write", "--write", "--check"]) {
        const result = spawnSync(
          path.join(root, identity.binaryPath),
          ["fmt", mode, "Example.vue"],
          {
            cwd: directory,
            timeout: 30000,
            maxBuffer: 1048576,
          },
        );
        const after = fs.readFileSync(file);
        row.cli.push({
          mode,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout?.toString() ?? "",
          stderr: result.stderr?.toString() ?? "",
          stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
          stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
          after: after.toString(),
          afterBase64: after.toString("base64"),
          configuration: fs.readFileSync(configPath).toString(),
        });
        persist();
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(
          result.status,
          Number(row.cli.length === 1 && !input.equals(expected)),
          result.stderr?.toString(),
        );
        assert.deepEqual(result.stdout, Buffer.alloc(0));
        assert.deepEqual(after, row.cli.length === 1 ? input : expected);
        assert.deepEqual(fs.readFileSync(configPath), config);
        const observed = await observeSource(
          after.toString(),
          `${selected.id}-${row.cli.length}`,
          states,
        );
        row.after.push(observed);
        persist();
        assert.deepEqual(observed.observations, contract.observations);
      }
    });
  }
  assert.equal(report.rows.length, 10);
  assert.equal(
    report.rows.reduce((sum, row) => sum + row.cli.length, 0),
    50,
  );
  assert.equal(
    report.rows.reduce((sum, row) => sum + row.after.length, 0),
    50,
  );
  for (const configuration of [
    { id: "original-paragraph-json", case: "original-paragraph", name: "vize.config.json" },
    {
      id: "original-ts-discovered",
      case: "original-example",
      name: "vize.config.ts",
      fixture: "vize.config.ts.txt",
    },
    {
      id: "original-ts-explicit",
      case: "original-example",
      name: "explicit.ts",
      fixture: "vize.config.ts.txt",
      explicit: true,
    },
    {
      id: "original-json-explicit",
      case: "original-example",
      name: "explicit.json",
      fixture: "vize.config.json.txt",
      explicit: true,
    },
  ]) {
    const selected = corpus.cases.find((row) => row.id === configuration.case);
    assert(selected);
    const project = path.join(directory, configuration.id);
    fs.mkdirSync(project);
    const file = path.join(project, "Example.vue");
    const configPath = path.join(project, configuration.name);
    const input = fs.readFileSync(path.join(fixtureRoot, selected.input));
    const expected = fs.readFileSync(path.join(fixtureRoot, selected.output));
    const config = configuration.fixture
      ? fs.readFileSync(path.join(fixtureRoot, configuration.fixture))
      : Buffer.from(`${JSON.stringify({ formatter: selected.options })}\n`);
    fs.writeFileSync(file, input);
    fs.writeFileSync(configPath, config);
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const args = [
        "fmt",
        ...(configuration.explicit ? ["--config", configuration.name] : []),
        mode,
        "Example.vue",
      ];
      const result = spawnSync(path.join(root, identity.binaryPath), args, {
        cwd: project,
        timeout: 30000,
        maxBuffer: 1048576,
      });
      const after = fs.readFileSync(file);
      report.configuredCli.push({
        id: configuration.id,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout?.toString() ?? "",
        stderr: result.stderr?.toString() ?? "",
        stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
        stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
        after: after.toString(),
        afterBase64: after.toString("base64"),
        configuration: fs.readFileSync(configPath).toString(),
      });
      persist();
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(
        result.status,
        Number(pass === 0 && !input.equals(expected)),
        result.stderr?.toString(),
      );
      assert.deepEqual(result.stdout, Buffer.alloc(0));
      assert.deepEqual(after, pass === 0 ? input : expected);
      assert.deepEqual(fs.readFileSync(configPath), config);
    }
  }
  assert.equal(report.configuredCli.length, 20);
});
