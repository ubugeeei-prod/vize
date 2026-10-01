import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.mjs";
import { loadFormatterApiManifest } from "./formatter-api.mjs";
import { sha256 } from "./manifest.mjs";

export function formatterHistoryCliExpected(mode: string, changed: boolean) {
  assert(["check", "dry", "write"].includes(mode));
  const change = changed ? `${mode === "write" ? "Reformatted" : "Would reformat"}: App.vue\n` : "";
  const summary =
    mode === "check"
      ? `Checked 1 file(s)\n  1 file(s) ${changed ? "would be reformatted" : "already formatted"}\n`
      : mode === "write"
        ? `Formatted 1 file(s)\n  1 file(s) ${changed ? "reformatted" : "unchanged"}\n`
        : `Checked 1 file(s) (use --write to apply changes)\n${changed ? "  1 file(s) would be reformatted\n" : ""}`;
  return {
    argv: ["fmt", "--no-config", ...(mode === "dry" ? [] : [`--${mode}`]), "App.vue"],
    exitStatus: mode === "check" && changed ? 1 : 0,
    stdoutBase64: "",
    stderrBase64: Buffer.from(`Found 1 file(s)\n${change}\n${summary}`).toString("base64"),
  };
}

// Expected stream bytes are repository-authored from the existing CLI contract,
// not described as captured observations. Only actual source-built execution
// earns a matched row. The retained raw streams make that distinction auditable.
export function runFormatterHistoryCli({
  repoRoot,
  binaryPath,
}: {
  repoRoot: string;
  binaryPath: string;
}) {
  const build = expectedBuildIdentity(repoRoot);
  assert.equal(fs.realpathSync(binaryPath), fs.realpathSync(path.join(repoRoot, build.binaryPath)));
  const receipt = JSON.parse(fs.readFileSync(`${binaryPath}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, build);
  const loaded = loadFormatterApiManifest(
    path.join(repoRoot, "tests/_fixtures/differential/formatter-history/script-manifest.json"),
    repoRoot,
  );
  const original = loaded.cases.find((item: { id: string }) => item.id === "sfc/signature");
  const singlePass = loaded.cases.find((item: { id: string }) => item.id === "sfc/signature-check");
  const additional = loadFormatterApiManifest(
    path.join(
      repoRoot,
      "tests/_fixtures/differential/formatter-history/capture-extra-manifest.json",
    ),
    repoRoot,
  );
  const small = additional.cases.find(
    (item: { id: string }) => item.id === "capture/small-const-check-default/sfc/0",
  );
  assert(original && singlePass && small);
  const plans = [
    { id: "formatter/history/check-authored-signature", input: original.input, changed: true },
    {
      id: "formatter/history/check-intermediate-signature",
      input: singlePass.expected,
      changed: true,
    },
    { id: "formatter/history/check-canonical-signature", input: original.expected, changed: false },
    { id: "formatter/history/check-authored-const", input: small.input, changed: true },
    { id: "formatter/history/check-canonical-const", input: small.expected, changed: false },
  ].map((fixture) => ({
    ...fixture,
    expectedOutput: fixture.id.endsWith("-const") ? small.expected : original.expected,
  }));
  const rows = plans.map((fixture) => {
    const passes: Record<string, unknown>[] = [];
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-history-cli-"));
    const file = path.join(workspace, "App.vue");
    let failure: string | undefined;
    try {
      fs.writeFileSync(file, fixture.input);
      for (const [index, mode] of ["check", "dry", "write", "check"].entries()) {
        const input = fs.readFileSync(file);
        const expected = formatterHistoryCliExpected(mode, index < 3 && fixture.changed);
        const result = spawnSync(binaryPath, expected.argv, {
          cwd: workspace,
          timeout: 30_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const output = fs.readFileSync(file);
        const stdout = result.stdout ?? Buffer.alloc(0);
        const stderr = result.stderr ?? Buffer.alloc(0);
        passes.push({
          step: index + 1,
          mode,
          argv: expected.argv,
          inputBase64: input.toString("base64"),
          inputSha256: sha256(input),
          outputBase64: output.toString("base64"),
          outputSha256: sha256(output),
          stdoutBase64: stdout.toString("base64"),
          stderrBase64: stderr.toString("base64"),
          exitStatus: result.status,
          signal: result.signal,
          processError: result.error?.message ?? null,
          expected,
        });
        assert.equal(result.error, undefined, result.error?.message);
        assert.equal(result.signal, null);
        assert.equal(result.status, expected.exitStatus, stderr.toString());
        assert.equal(stdout.toString("base64"), expected.stdoutBase64);
        assert.equal(stderr.toString("base64"), expected.stderrBase64);
        assert(
          output.equals(mode === "write" ? fixture.expectedOutput : input),
          "check/dry must preserve the file; write must produce the full canonical output",
        );
      }
    } catch (error) {
      failure = error instanceof Error ? error.message : String(error);
    } finally {
      fs.rmSync(workspace, { recursive: true, force: true });
    }
    return {
      id: fixture.id,
      target: "fmt",
      legacy: {
        state: failure ? "failed" : "completed",
        passes,
        ...(failure ? { error: failure } : {}),
      },
      native: { state: "unsupported", reason: "native formatter adapter unavailable" },
      comparison: { state: "not-compared", reason: "native formatter adapter unavailable" },
    };
  });
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "formatter",
    contract: "formatter-cli-history-verdict-streams-v1",
    sourceRevision: build.sourceRevision,
    buildReceipt: receipt,
    binary: { path: build.binaryPath, sha256: build.binarySha256, version: build.cliVersion },
    manifestSha256: loaded.manifestSha256,
    additionalManifestSha256: additional.manifestSha256,
    provenance: {
      fix: "d25a270fe1d8e8ce8d15b345d6874caa63079374",
      issue: 6882,
      witness:
        "crates/vize_glyph/tests/sfc_check_mode.rs::check_mode_still_flags_script_needing_second_stabilization_pass",
      expectedStreams: "repository-authored; actual observations retained separately",
    },
    rows,
    summary: {
      plannedCases: rows.length,
      legacyMatches: rows.filter((row) => row.legacy.state === "completed").length,
      legacyFailures: rows.filter((row) => row.legacy.state === "failed").length,
      nativeUnsupported: rows.length,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
}
