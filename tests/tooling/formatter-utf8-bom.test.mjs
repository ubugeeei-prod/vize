import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath = "tests/_fixtures/differential/formatter-regressions/utf8-bom-7870/cases.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);

void test("original UTF-8 BOM formatter corpus preserves complete document bytes", (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 7870);
  assert.equal(corpus.cases.length, 14);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-format-bom-"));
  const report = {
    schema: "vize.formatter-bom-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    rows: [],
  };
  try {
    fs.mkdirSync(path.join(directory, ".git"));
    for (const fixture of corpus.cases) {
      const file = path.join(directory, fixture.file);
      fs.writeFileSync(file, fixture.source);
      const row = {
        id: fixture.id,
        inputSha256: sha256(Buffer.from(fixture.source)),
        attempts: [],
      };
      report.rows.push(row);
      if (fixture.eol) {
        fs.writeFileSync(
          path.join(directory, "vize.config.json"),
          `${JSON.stringify({ formatter: { endOfLine: fixture.eol } })}\n`,
        );
      }
      const invoke = (mode) => {
        const argv = [
          "fmt",
          mode,
          fixture.file,
          ...(fixture.eol ? ["--config", "vize.config.json"] : ["--no-config"]),
        ];
        const before = fs.readFileSync(file);
        const result = spawnSync(path.join(root, identity.binaryPath), argv, {
          cwd: directory,
          env: { ...process.env, NO_COLOR: "1" },
          timeout: 30000,
          maxBuffer: 1048576,
        });
        const after = fs.readFileSync(file);
        row.attempts.push({
          argv,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout?.toString() ?? "",
          stderr: result.stderr?.toString() ?? "",
          before: before.toString(),
          after: after.toString(),
          beforeSha256: sha256(before),
          afterSha256: sha256(after),
        });
        assert.equal(result.error, undefined, fixture.id);
        assert.equal(result.signal, null, fixture.id);
        return { result, before, after };
      };
      const initial = invoke("--check");
      assert.deepEqual(initial.after, initial.before, `${fixture.id}: check is read-only`);
      if (fixture.outcome === "error") {
        assert.equal(initial.result.status, 1, fixture.id);
        const refused = invoke("--write");
        assert.equal(refused.result.status, 1, fixture.id);
        assert.deepEqual(refused.after, refused.before, fixture.id);
        continue;
      }
      assert.equal(initial.result.status, fixture.source === fixture.expected ? 0 : 1, fixture.id);
      for (let pass = 1; pass <= 3; pass++) {
        const { result, after } = invoke("--write");
        assert.equal(result.status, 0, `${fixture.id}: pass ${pass}`);
        assert.deepEqual(after, Buffer.from(fixture.expected), `${fixture.id}: whole pass ${pass}`);
      }
      const final = invoke("--check");
      assert.equal(final.result.status, 0, fixture.id);
      assert.deepEqual(final.after, final.before, `${fixture.id}: final check is read-only`);
    }
  } finally {
    const artifact = path.join(root, "target/differential/formatter-bom-7870/cli-report.json");
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Complete source-built CLI observations: ${artifact}`);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
