import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath =
  "tests/_fixtures/differential/formatter-regressions/document-support-7867/cases.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);
const comparePaths = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

void test("formatter refuses unsupported document syntax and excludes it from defaults", (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 7867);
  assert.equal(corpus.cases.length, 6);
  const report = {
    schema: "vize.formatter-document-support-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    rows: [],
  };
  try {
    for (const fixture of corpus.cases) {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-format-document-support-"));
      const snapshot = () =>
        Object.fromEntries(
          Object.keys(corpus.files).map((file) => [
            file,
            fs.readFileSync(path.join(directory, file), "utf8"),
          ]),
        );
      const row = { id: fixture.id, attempts: [] };
      report.rows.push(row);
      try {
        fs.mkdirSync(path.join(directory, ".git"));
        for (const [file, source] of Object.entries(corpus.files)) {
          fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
          fs.writeFileSync(path.join(directory, file), source);
        }
        const expected = { ...corpus.files };
        if (fixture.selected.includes("src/main.ts"))
          expected["src/main.ts"] = corpus.formattedScript;
        const invoke = (mode) => {
          const argv = ["fmt", mode, "--no-config", ...fixture.patterns];
          const before = snapshot();
          const result = spawnSync(path.join(root, identity.binaryPath), argv, {
            cwd: directory,
            env: { ...process.env, NO_COLOR: "1" },
            timeout: 30000,
            maxBuffer: 1048576,
          });
          const after = snapshot();
          const stderr = result.stderr?.toString() ?? "";
          row.attempts.push({
            argv,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdout: result.stdout?.toString() ?? "",
            stderr,
            before,
            after,
          });
          assert.equal(result.error, undefined, fixture.id);
          assert.equal(result.signal, null, fixture.id);
          assert.equal(result.stdout?.toString(), "", fixture.id);
          assert.match(
            stderr,
            new RegExp(`Found ${fixture.selected.length} file\\(s\\)`),
            fixture.id,
          );
          const errors = [...stderr.matchAll(/^Error formatting (.+): (.+)$/gm)]
            .map((match) => {
              assert.equal(match[2], corpus.error, fixture.id);
              return match[1].split(path.sep).join("/");
            })
            .sort(comparePaths);
          assert.deepEqual(
            errors,
            [...fixture.errors].sort(comparePaths),
            `${fixture.id}: complete error vector`,
          );
          return { result, before, after };
        };
        const check = invoke("--check");
        assert.equal(
          check.result.status,
          fixture.errors.length || fixture.selected.includes("src/main.ts") ? 1 : 0,
          fixture.id,
        );
        assert.deepEqual(check.after, corpus.files, `${fixture.id}: read-only check`);
        for (let pass = 1; pass <= 3; pass++) {
          const write = invoke("--write");
          assert.equal(write.result.status, fixture.errors.length ? 1 : 0, fixture.id);
          assert.deepEqual(write.after, expected, `${fixture.id}: whole project pass ${pass}`);
        }
        const recheck = invoke("--check");
        assert.equal(recheck.result.status, fixture.errors.length ? 1 : 0, fixture.id);
        assert.deepEqual(recheck.after, expected, fixture.id);
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  } finally {
    const artifact = path.join(
      root,
      "target/differential/formatter-document-support-7867/cli-report.json",
    );
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Complete project and unsupported format observations: ${artifact}`);
  }
});
