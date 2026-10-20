import { vueConfigFixtureSource } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import {
  compareTemplateAstEquivalence,
  templateAstSemanticSignature,
} from "./support/sfc-equivalence.ts";
import { formatterCliTarget } from "../differential/formatter-cli-target.ts";
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
  "091ba20a0035ca8bd08433e5d63a94b89301abf98856651d69a52262b7eff35a",
);
const index = JSON.parse(indexBytes);
assert.equal(index.contracts.length, 13);
const corpusBytes = fs.readFileSync(path.join(fixtureRoot, "corpus.json"));
assert.equal(
  sha256(corpusBytes),
  "01110cd1256ab49fe407547567b43300f41b32be5859cd6fc3b81c9c8b4b6b62",
);
const corpus = JSON.parse(corpusBytes);
assert.equal(corpus.cases.length, 14);
const currentReferences = new Map([
  [
    "exact-40",
    {
      file: "exact-40.hugged-inline-current.expected",
      sha256: "4fc1cad052f79800c50d2b06f0726239d4a5b7be8a09f1c514c4534fdd69fc7f",
    },
  ],
]);

const framingIds = new Set([
  "dynamic-bind-brackets-arrow-value-lf",
  "dynamic-event-brackets-modifier-lf",
  "dynamic-template-key-assignment-crlf",
]);
const { parse } = createRequire(import.meta.url)(providerIdentities.compilerSfc.entry);
function stockTemplate(source, id) {
  const parsed = parse(source, { filename: id + ".vue", sourceMap: false });
  assert.deepEqual(parsed.errors, []);
  assert.ok(parsed.descriptor.template?.ast);
  return parsed.descriptor.template.ast;
}
function templateEvidence(source, id) {
  const ast = stockTemplate(source, id);
  return { ast, signature: templateAstSemanticSignature(ast) };
}

void test("source CLI keeps complete authored DOM text SSR and event contracts", async (t) => {
  const target = await formatterCliTarget(root, "formatter-continuation-prefix-width.json");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-continuation-prefix-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const report = {
    schema: "vize.formatter-continuation-prefix-observation",
    version: 1,
    buildReceipt: target.kind === "source-built-cli" ? target.receipt : null,
    publicCliReceipt: target.kind === "installed-public-cli" ? target.receipt : null,
    corpusSha256: sha256(corpusBytes),
    contractsSha256: sha256(indexBytes),
    providerIdentities,
    nativeSupported: false,
    nativeHandled: 0,
    nativeEquivalent: 0,
    rows: [],
    configuredCli: [],
  };
  const reportPath = target.reportPath;
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
      const archivedExpected = fs.readFileSync(path.join(fixtureRoot, fixture.output));
      assert.equal(sha256(input), selected.inputSha256);
      assert.equal(sha256(archivedExpected), selected.expectedSourceSha256);
      const currentReference = currentReferences.get(selected.id) ?? null;
      const expected = currentReference
        ? fs.readFileSync(path.join(fixtureRoot, currentReference.file))
        : archivedExpected;
      if (currentReference) assert.equal(sha256(expected), currentReference.sha256);
      const states = contract.observations.map((item) => item.state);
      const before = await observeSource(input.toString(), `${selected.id}-original`, states);
      const reference = await observeSource(
        expected.toString(),
        `${selected.id}-reference`,
        states,
      );
      assert.deepEqual(before.observations, contract.observations);
      assert.deepEqual(reference.observations, contract.observations);
      const row = {
        id: selected.id,
        currentReference,
        before,
        reference,
        cli: [],
        after: [],
      };
      if (framingIds.has(selected.id)) {
        row.template = {
          before: templateEvidence(input.toString(), selected.id),
          reference: templateEvidence(expected.toString(), selected.id),
          after: [],
        };
        assert.deepEqual(
          compareTemplateAstEquivalence(row.template.before.ast, row.template.reference.ast),
          [],
        );
        assert.equal(row.template.before.signature, row.template.reference.signature);
        assert.equal(before.domCode, reference.domCode);
        assert.equal(before.ssrCode, reference.ssrCode);
      }
      report.rows.push(row);
      persist();
      const file = path.join(directory, "Example.vue");
      const configPath = path.join(directory, "vite.config.mjs");
      const config = Buffer.from(vueConfigFixtureSource({ formatter: fixture.options }));
      fs.writeFileSync(file, input);
      fs.writeFileSync(configPath, config);
      for (const mode of ["--check", "--write", "--write", "--write", "--check"]) {
        const result = spawnSync(target.binary, ["fmt", mode, "Example.vue"], {
          cwd: directory,
          timeout: 30000,
          maxBuffer: 1048576,
        });
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
        if (row.template) {
          const evidence = templateEvidence(after.toString(), selected.id);
          row.template.after.push(evidence);
          persist();
          assert.deepEqual(
            compareTemplateAstEquivalence(row.template.before.ast, evidence.ast),
            [],
          );
          assert.equal(evidence.signature, row.template.before.signature);
        }
        row.after.push(observed);
        persist();
        assert.deepEqual(observed.observations, contract.observations);
      }
    });
  }
  assert.equal(report.rows.length, 13);
  const originalRows = report.rows.slice(0, 10);
  const framingRows = report.rows.slice(10);
  assert.deepEqual(
    framingRows.map((row) => row.id),
    [...framingIds],
  );
  assert.equal(
    framingRows.reduce((sum, row) => sum + row.cli.length, 0),
    15,
  );
  assert.equal(
    framingRows.reduce((sum, row) => sum + row.after.length, 0),
    15,
  );
  assert.equal(
    originalRows.reduce((sum, row) => sum + row.cli.length, 0),
    50,
  );
  assert.equal(
    originalRows.reduce((sum, row) => sum + row.after.length, 0),
    50,
  );
  for (const configuration of [
    { id: "original-paragraph-json", case: "original-paragraph", name: "vite.config.mjs" },
    {
      id: "original-ts-discovered",
      case: "original-example",
      name: "vite.config.ts",
      fixture: "vize.config.ts.txt",
    },
    {
      id: "original-ts-explicit",
      case: "original-example",
      name: "vite.config.mts",
      fixture: "vize.config.ts.txt",
      explicit: true,
    },
    {
      id: "original-json-explicit",
      case: "original-example",
      name: "vite.config.mjs",
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
    // Frozen native carriers retain their original bytes; only the active Vite
    // module changes representation. TypeScript archives here are plain exports.
    const originalConfig = configuration.fixture
      ? fs.readFileSync(path.join(fixtureRoot, configuration.fixture))
      : Buffer.from(`${JSON.stringify({ formatter: selected.options })}\n`);
    const document = configuration.fixture?.endsWith(".ts.txt")
      ? (await import(`data:text/javascript,${encodeURIComponent(originalConfig.toString())}`))
          .default
      : JSON.parse(originalConfig.toString());
    const config = Buffer.from(vueConfigFixtureSource(document));
    fs.writeFileSync(file, input);
    fs.writeFileSync(configPath, config);
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const args = [
        "fmt",
        ...(configuration.explicit ? ["--config", configuration.name] : []),
        mode,
        "Example.vue",
      ];
      const result = spawnSync(target.binary, args, {
        cwd: project,
        timeout: 30000,
        maxBuffer: 1048576,
      });
      const after = fs.readFileSync(file);
      report.configuredCli.push({
        id: configuration.id,
        originalConfigurationBase64: originalConfig.toString("base64"),
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
  target.assertUnchanged();
});
