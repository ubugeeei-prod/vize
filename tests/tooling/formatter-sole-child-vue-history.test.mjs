import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { soleChildVueAuthority } from "../differential/formatter-sole-child-width-reference.mjs";
import { createVueHistoryStockObserver } from "./support/formatter-vue-history-stock.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const snapshot = (bytes) => ({
  sha256: createHash("sha256").update(bytes).digest("hex"),
  bytes: bytes.length,
  base64: bytes.toString("base64"),
});

void test("actual CLI retains complete Vue filter and bitwise history semantics", async (t) => {
  const corpus = soleChildVueAuthority(root);
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  const binary = path.join(root, identity.binaryPath);
  const stock = await createVueHistoryStockObserver(root);
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sole-child-vue-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const report = {
    identity,
    receipt,
    nativeAdmission: "unsupported; no native adapter credit",
    commandCount: 0,
    references: [],
    cli: [],
    rejectionControls: [],
  };
  const reportPath = path.join(root, "target/differential/formatter-sole-child-vue-history.json");
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  for (const fixture of corpus.records) {
    const row = { id: fixture.id, original: {}, historical: {}, current: {} };
    report.references.push(row);
    persist();
    for (const phase of ["original", "historical", "current"]) {
      const source =
        phase === "original"
          ? fixture.input.content
          : phase === "historical"
            ? fixture.historical.content
            : fixture.expected;
      await stock.qualify(source, fixture, phase, row[phase], persist);
    }
  }
  for (const fixture of corpus.records.slice(0, 3)) {
    const project = path.join(scratch, fixture.id);
    fs.mkdirSync(project);
    const file = path.join(project, "App.vue");
    const configPath = path.join(project, "vize.config.json");
    const config = Buffer.from(fixture.config.content);
    fs.writeFileSync(file, fixture.input.content);
    fs.writeFileSync(configPath, config);
    const row = { id: fixture.id, commands: [] };
    report.cli.push(row);
    persist();
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const changed = pass < 2 && fixture.input.content !== fixture.expected;
      const expected = pass === 0 ? fixture.input.content : fixture.expected;
      const args = ["fmt", "--config", "vize.config.json", mode, "App.vue"];
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
      row.commands.push(command);
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
        ? `${mode === "--check" ? "Would reformat" : "Reformatted"}: App.vue\n`
        : "";
      const summary =
        mode === "--check"
          ? `Checked 1 file(s)\n  1 file(s) ${changed ? "would be reformatted" : "already formatted"}\n`
          : `Formatted 1 file(s)\n  1 file(s) ${changed ? "reformatted" : "unchanged"}\n`;
      assert.deepEqual(output.stderr, Buffer.from(`Found 1 file(s)\n${detail}\n${summary}`));
      assert.deepEqual(actual, Buffer.from(expected), fixture.id);
      assert.deepEqual(actualConfig, config);
      await stock.qualify(
        actual.toString(),
        fixture,
        pass === 0 ? "original" : "current",
        command.after,
        persist,
      );
    }
  }
  assert.equal(report.commandCount, 15);
  // Keep negative controls separate from execution/cardinality acceptance.
  // A trailing carrier LF leaves the exact template intact and must still fail.
  for (const version of ["2", "2.7", "3"]) {
    const fixture = corpus.records.find((row) => row.version === version);
    const row = { id: `whole-source-drift-${version}`, observation: {} };
    report.rejectionControls.push(row);
    persist();
    await assert.rejects(
      stock.qualify(`${fixture.input.content}\n`, fixture, "original", row.observation, persist),
      /complete original source bytes/,
    );
    assert.equal(row.observation.qualified, false);
    assert.equal(row.observation.runtime.length, 4);
  }
  const fixture = structuredClone(corpus.records[0]);
  fixture.stock.packets.original.errors = [];
  const row = { id: "missing-original-multi-root-diagnostic", observation: {} };
  report.rejectionControls.push(row);
  persist();
  await assert.rejects(
    stock.qualify(fixture.input.content, fixture, "original", row.observation, persist),
    /full sealed original compiler packet/,
  );
  assert.equal(row.observation.qualified, false);
  assert.equal(row.observation.packet.errors.length, 1);
  assert.equal(row.observation.runtime.length, 4);
  assert.equal(report.rejectionControls.length, 4);
});
