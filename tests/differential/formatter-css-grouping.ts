// Whole-output #7826 regressions use the existing actual formatter build.
// These authored expectations are distinct from the frozen historical pack.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.mjs";
import { validateObserverReceipt } from "./formatter-api-build.mjs";
import { formatterHistoryCliExpected } from "./formatter-history-cli.ts";
import { sha256 } from "./manifest.mjs";

export const groupingCorpus = "crates/vize_glyph/tests/fixtures/style_rule_blank_lines_7826.json";

type Case = {
  id: string;
  api: "sfc" | "style";
  input: string;
  expected: string;
  options: Record<string, unknown>;
  cli: boolean;
};

function observe(binary: string, argv: string[], cwd: string, input?: Buffer) {
  const result = spawnSync(binary, argv, { cwd, input, timeout: 30_000 });
  return {
    argv,
    inputBase64: input?.toString("base64") ?? null,
    inputSha256: input ? sha256(input) : null,
    stdoutBase64: result.stdout?.toString("base64") ?? "",
    stdoutSha256: sha256(result.stdout ?? Buffer.alloc(0)),
    stderrBase64: result.stderr?.toString("base64") ?? "",
    exitStatus: result.status,
    signal: result.signal,
    processError: result.error?.message ?? null,
  };
}

function successfulTransport(observation: ReturnType<typeof observe>) {
  assert.equal(observation.signal, null);
  assert.equal(observation.processError, null);
}

export function runCssGroupingRegressions({
  repoRoot,
  binaryPath,
  receipt,
  evidenceDir,
}: {
  repoRoot: string;
  binaryPath: string;
  receipt: unknown;
  evidenceDir: string;
}) {
  validateObserverReceipt(receipt, repoRoot, binaryPath);
  const corpusBytes = fs.readFileSync(path.join(repoRoot, groupingCorpus));
  const corpus = JSON.parse(corpusBytes.toString());
  assert.equal(corpus.schema, "vize.formatter-css-grouping-regressions");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.issue, 7826);
  assert.equal(corpus.cases.length, 27);
  assert.equal(new Set(corpus.cases.map((row: Case) => row.id)).size, 27);
  const cliIdentity = expectedBuildIdentity(repoRoot);
  const cli = path.join(repoRoot, cliIdentity.binaryPath);
  const cliReceipt = JSON.parse(fs.readFileSync(cli + ".differential-build.json", "utf8"));
  validateBuildReceipt(cliReceipt, cliIdentity);
  const report = {
    schema: "vize.formatter-css-grouping-execution",
    version: 1,
    issue: 7826,
    corpus: { path: groupingCorpus, sha256: sha256(corpusBytes) },
    apiBuildReceipt: receipt,
    cliBuildReceipt: cliReceipt,
    rows: [] as unknown[],
    failure: null as string | null,
  };
  try {
    for (const fixture of corpus.cases as Case[]) {
      assert(["sfc", "style"].includes(fixture.api));
      assert.equal(typeof fixture.input, "string");
      assert.equal(typeof fixture.expected, "string");
      const flags = Object.keys(fixture.options).length
        ? ["--options", JSON.stringify(fixture.options)]
        : [];
      const row = { id: fixture.id, api: [] as unknown[], cli: [] as unknown[] };
      report.rows.push(row);
      const options = observe(binaryPath, ["--options-json", ...flags], repoRoot);
      row.api.push({ phase: "options", ...options });
      successfulTransport(options);
      assert.equal(options.exitStatus, 0);
      assert.equal(options.stderrBase64, "");
      const effective = JSON.parse(Buffer.from(options.stdoutBase64, "base64").toString());
      for (const [key, value] of Object.entries(fixture.options))
        assert.deepEqual(effective[key], value);
      let input = Buffer.from(fixture.input);
      for (let pass = 1; pass <= 3; pass++) {
        const output = observe(binaryPath, ["--" + fixture.api, ...flags], repoRoot, input);
        row.api.push({ pass, ...output });
        successfulTransport(output);
        assert.equal(output.exitStatus, 0, fixture.id);
        assert.equal(
          output.stdoutBase64,
          Buffer.from(fixture.expected).toString("base64"),
          fixture.id,
        );
        const stderr =
          fixture.api === "sfc" ? `changed=${input.toString() !== fixture.expected}\n` : "";
        assert.equal(output.stderrBase64, Buffer.from(stderr).toString("base64"), fixture.id);
        input = Buffer.from(output.stdoutBase64, "base64");
      }
      if (!fixture.cli) continue;
      assert.equal(fixture.api, "sfc");
      assert.deepEqual(fixture.options, {});
      const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-grouping-7826-"));
      const file = path.join(cwd, "App.vue");
      try {
        fs.writeFileSync(file, fixture.input);
        let current = fixture.input;
        for (const mode of ["check", "dry", "write", "check"] as const) {
          const changed = current !== fixture.expected;
          const expected = formatterHistoryCliExpected(mode, changed);
          const before = fs.readFileSync(file);
          const output = observe(cli, expected.argv, cwd);
          const after = fs.readFileSync(file);
          row.cli.push({
            mode,
            beforeBase64: before.toString("base64"),
            afterBase64: after.toString("base64"),
            beforeSha256: sha256(before),
            afterSha256: sha256(after),
            ...output,
          });
          successfulTransport(output);
          assert.equal(output.exitStatus, expected.exitStatus, fixture.id);
          assert.equal(output.stdoutBase64, expected.stdoutBase64, fixture.id);
          assert.equal(output.stderrBase64, expected.stderrBase64, fixture.id);
          assert.equal(before.toString(), current, fixture.id);
          current = mode === "write" ? fixture.expected : current;
          assert.equal(after.toString(), current, fixture.id);
        }
      } finally {
        fs.rmSync(cwd, { recursive: true, force: true });
      }
    }
    validateObserverReceipt(receipt, repoRoot, binaryPath);
    assert.deepEqual(
      expectedBuildIdentity(repoRoot),
      cliIdentity,
      "CLI changed during observations",
    );
  } catch (error) {
    report.failure = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    fs.writeFileSync(
      path.join(evidenceDir, "css-rule-blank-lines-7826-report.json"),
      JSON.stringify(report, null, 2) + "\n",
    );
  }
  assert.equal(report.rows.length, 27);
  return report;
}
