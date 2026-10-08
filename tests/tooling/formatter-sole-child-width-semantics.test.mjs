import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { createChildWidthStockObserver } from "./support/formatter-child-width-stock.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const authority =
  "tests/_fixtures/differential/formatter-regressions/sole-child-width-7876/references.json";
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const snapshot = (bytes) => ({
  sha256: sha(bytes),
  bytes: bytes.length,
  base64: bytes.toString("base64"),
});

void test("actual source CLI preserves sealed sole-child DOM/text/SSR/event contracts", async (t) => {
  const raw = fs.readFileSync(path.join(root, authority));
  assert.equal(sha(raw), "ebe294c117dc2e5e055575a110f68e2b5195beed5f2328f6693b3650efb27fe4");
  const corpus = JSON.parse(raw);
  assert.equal(corpus.cases.length, 16);
  assert.equal(
    corpus.provider.entrySha256,
    "a601ee08f9ad479ee1ec8eac8f34e51880626c8be6bdfed04c75b086caeb055c",
  );
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  const binary = path.join(root, identity.binaryPath);
  const stock = await createChildWidthStockObserver(root);
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sole-child-width-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const report = {
    identity,
    receipt,
    referenceSha256: sha(raw),
    commandCount: 0,
    observations: [],
  };
  const reportPath = path.join(root, "target/differential/formatter-sole-child-width.json");
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  for (const fixture of corpus.cases) {
    assert.equal(sha(Buffer.from(fixture.input)), fixture.inputSha256);
    assert.equal(sha(Buffer.from(fixture.expected)), fixture.expectedSha256);
    const row = { id: fixture.id, before: {}, reference: {}, cli: [] };
    report.observations.push(row);
    persist();
    await stock.qualify(fixture.input, fixture, "original", row.before, persist);
    await stock.qualify(fixture.expected, fixture, "expected", row.reference, persist);
    const project = path.join(scratch, fixture.id);
    fs.mkdirSync(project);
    const filename = "App.vue";
    const file = path.join(project, filename);
    const configPath = path.join(project, "vize.config.json");
    const config = Buffer.from(`${JSON.stringify({ formatter: fixture.options })}\n`);
    fs.writeFileSync(configPath, config);
    fs.writeFileSync(file, fixture.input);
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const changed = pass < 2 && fixture.input !== fixture.expected;
      const expected = pass === 0 ? fixture.input : fixture.expected;
      const args = ["fmt", mode, filename];
      const output = spawnSync(binary, args, { cwd: project, timeout: 60000 });
      const command = {
        args,
        status: output.status,
        signal: output.signal,
        error: output.error?.message,
        stdout: snapshot(output.stdout ?? Buffer.alloc(0)),
        stderr: snapshot(output.stderr ?? Buffer.alloc(0)),
        after: {},
      };
      row.cli.push(command);
      report.commandCount++;
      persist();
      const actual = fs.readFileSync(file);
      const actualConfig = fs.readFileSync(configPath);
      command.file = snapshot(actual);
      command.config = snapshot(actualConfig);
      persist();
      assert.ifError(output.error);
      assert.equal(output.signal, null);
      assert.equal(output.status, Number(changed && mode === "--check"), fixture.id);
      assert.deepEqual(output.stdout, Buffer.alloc(0));
      const detail = changed
        ? `${mode === "--check" ? "Would reformat" : "Reformatted"}: ${filename}\n`
        : "";
      const summary =
        mode === "--check"
          ? `Checked 1 file(s)\n  1 file(s) ${changed ? "would be reformatted" : "already formatted"}\n`
          : `Formatted 1 file(s)\n  1 file(s) ${changed ? "reformatted" : "unchanged"}\n`;
      assert.deepEqual(output.stderr, Buffer.from(`Found 1 file(s)\n${detail}\n${summary}`));
      assert.deepEqual(actual, Buffer.from(expected));
      assert.deepEqual(actualConfig, config);
      await stock.qualify(
        actual.toString(),
        fixture,
        pass === 0 ? "original" : "expected",
        command.after,
        persist,
      );
    }
  }
  assert.equal(report.commandCount, 80);
});
