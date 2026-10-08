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
const corpusPath = "tests/_fixtures/differential/formatter-regressions/discovery-7869/cases.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);
const comparePaths = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

void test("formatter discovery never writes installed dependencies and reads standalone gitignore", (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 7869);
  assert.equal(corpus.cases.length, 8);
  const report = {
    schema: "vize.formatter-discovery-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    rows: [],
  };
  try {
    for (const fixture of corpus.cases) {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-format-discovery-"));
      const snapshot = () =>
        Object.fromEntries(
          Object.keys(corpus.files).map((file) => [
            file,
            fs.readFileSync(path.join(directory, file), "utf8"),
          ]),
        );
      const row = { id: fixture.id, selected: fixture.selected, attempts: [] };
      report.rows.push(row);
      try {
        for (const [file, source] of Object.entries(corpus.files)) {
          fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
          fs.writeFileSync(path.join(directory, file), source);
        }
        if (fixture.git) {
          const init = spawnSync("git", ["init", "--quiet"], { cwd: directory, encoding: "utf8" });
          assert.equal(init.status, 0, init.stderr);
        }
        const patterns = fixture.patterns.map((pattern) =>
          fixture.absolute ? path.join(directory, pattern) : pattern,
        );
        const expected = { ...corpus.files };
        for (const selected of fixture.selected) {
          if (expected[selected] === "const   a=1\n") expected[selected] = corpus.formatted;
        }
        const changed = fixture.selected.filter(
          (selected) => expected[selected] !== corpus.files[selected],
        );
        const invoke = (mode) => {
          const argv = ["fmt", mode, "--no-config", ...patterns];
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
          if (fixture.selected.length)
            assert.match(
              stderr,
              new RegExp(`Found ${fixture.selected.length} file\\(s\\)`),
              fixture.id,
            );
          else assert.match(stderr, /^No .* files found matching the patterns\n$/, fixture.id);
          return { result, before, after, stderr };
        };
        const check = invoke("--check");
        assert.equal(
          check.result.status,
          !fixture.selected.length || changed.length ? 1 : 0,
          fixture.id,
        );
        assert.deepEqual(check.after, corpus.files, `${fixture.id}: whole check custody`);
        const observed = [...check.stderr.matchAll(/^Would reformat: (.+)$/gm)]
          .map((match) =>
            path.relative(directory, path.resolve(directory, match[1])).split(path.sep).join("/"),
          )
          .sort(comparePaths);
        assert.deepEqual(
          observed,
          [...changed].sort(comparePaths),
          `${fixture.id}: complete changed selection`,
        );
        for (let pass = 1; pass <= 3; pass++) {
          const write = invoke("--write");
          assert.equal(
            write.result.status,
            fixture.selected.length ? 0 : 1,
            `${fixture.id}: pass ${pass}`,
          );
          assert.deepEqual(write.after, expected, `${fixture.id}: whole project pass ${pass}`);
        }
        const recheck = invoke("--check");
        assert.equal(recheck.result.status, fixture.selected.length ? 0 : 1, fixture.id);
        assert.deepEqual(recheck.after, expected, fixture.id);
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  } finally {
    const artifact = path.join(
      root,
      "target/differential/formatter-discovery-7869/cli-report.json",
    );
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Complete project and raw CLI observations: ${artifact}`);
  }
});
