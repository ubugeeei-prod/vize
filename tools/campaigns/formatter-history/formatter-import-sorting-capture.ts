import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

export async function captureImportSorting(root: string, evidence: string) {
  const { expectedBuildIdentity, validateBuildReceipt } = await import(
    pathToFileURL(path.join(root, "tests/differential/build-receipt.mjs")).href
  );
  const hash = (b: Buffer) => crypto.createHash("sha256").update(b).digest("hex");
  const originalPath = "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue";
  const expectedPath = "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue";
  const witnessPath = "crates/vize/tests/fmt_import_sorting_cli.rs";
  const original = fs.readFileSync(path.join(root, originalPath));
  const canonical = fs.readFileSync(path.join(root, expectedPath));
  const witness = fs.readFileSync(path.join(root, witnessPath));
  const identity = expectedBuildIdentity(root);
  const binary = path.join(root, identity.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, identity);
  const script = (bytes: Buffer) => {
    const start = bytes.indexOf(10) + 1;
    const end = bytes.indexOf(Buffer.from("</script>"), start);
    assert(start > 0 && end > start && bytes.indexOf(Buffer.from("</script>"), end + 1) === -1);
    return { bytes: bytes.subarray(start, end), startByte: start, endByte: end };
  };
  const inputScript = script(original),
    outputScript = script(canonical);
  const enabled = '{"formatter":{"sortImports":{}}}';
  const plans = [
    {
      id: "no-config",
      file: "UserCard.vue",
      input: original,
      expected: original,
      config: enabled,
      argv: ["--no-config", "--check"],
      status: 0,
      changed: false,
      mode: "check",
    },
    {
      id: "configured-check",
      file: "UserCard.vue",
      input: original,
      expected: original,
      config: enabled,
      argv: ["--check"],
      status: 1,
      changed: true,
      mode: "check",
    },
    {
      id: "configured-write",
      file: "UserCard.vue",
      input: original,
      expected: canonical,
      config: enabled,
      argv: ["--write"],
      status: 0,
      changed: true,
      mode: "write",
    },
    {
      id: "canonical-recheck",
      file: "UserCard.vue",
      input: canonical,
      expected: canonical,
      config: enabled,
      argv: ["--check"],
      status: 0,
      changed: false,
      mode: "check",
    },
    {
      id: "disabled-write",
      file: "UserCard.vue",
      input: original,
      expected: original,
      config: '{"formatter":{"sortImports":false}}',
      argv: ["--write"],
      status: 0,
      changed: false,
      mode: "write",
    },
    {
      id: "standalone-script",
      file: "imports.ts",
      input: inputScript.bytes,
      expected: outputScript.bytes,
      config: enabled,
      argv: ["--write"],
      status: 0,
      changed: true,
      mode: "write",
    },
    {
      id: "contradictory-config",
      file: "UserCard.vue",
      input: original,
      expected: original,
      config: '{"formatter":{"sortImports":{"partitionByNewline":true}}}',
      argv: ["--write"],
      status: 2,
      changed: false,
      mode: "error",
    },
    {
      id: "malformed-config",
      file: "UserCard.vue",
      input: original,
      expected: original,
      config: '{"formatter":{"sortImports":{"groups":{}}}}',
      argv: ["--write"],
      status: 2,
      changed: false,
      mode: "error",
    },
  ];
  const directory = path.join(evidence, "import-sorting");
  assert(!fs.existsSync(directory), "do not replace prior sorting captures");
  fs.mkdirSync(directory, { recursive: true });
  const workspaceRoot = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-sort-history-"));
  const streams = (plan: (typeof plans)[number]) => {
    if (plan.mode === "error") return null;
    const changed = plan.changed
      ? `${plan.mode === "write" ? "Reformatted" : "Would reformat"}: ${plan.file}\n`
      : "";
    const summary =
      plan.mode === "write"
        ? `Formatted 1 file(s)\n  1 file(s) ${plan.changed ? "reformatted" : "unchanged"}\n`
        : `Checked 1 file(s)\n  1 file(s) ${plan.changed ? "would be reformatted" : "already formatted"}\n`;
    return Buffer.from(`Found 1 file(s)\n${changed}\n${summary}`);
  };
  const capturePlan = (plan: (typeof plans)[number]) => {
    const workspace = path.join(workspaceRoot, plan.id);
    const observations: any[] = [];
    const failures: string[] = [];
    fs.mkdirSync(workspace, { recursive: true });
    for (const repeat of [1, 2]) {
      const file = path.join(workspace, plan.file);
      fs.writeFileSync(file, plan.input);
      fs.writeFileSync(path.join(workspace, "vize.config.json"), plan.config);
      const argv = ["fmt", ...plan.argv, plan.file];
      const startedAtUtc = new Date().toISOString();
      const actual = spawnSync(binary, argv, {
        cwd: workspace,
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const stdout = actual.stdout ?? Buffer.alloc(0),
        stderr = actual.stderr ?? Buffer.alloc(0);
      const observation: any = {
        repeat,
        argv,
        cwd: workspace,
        startedAtUtc,
        finishedAtUtc: new Date().toISOString(),
        inputBase64: plan.input.toString("base64"),
        inputSha256: hash(plan.input),
        configBase64: Buffer.from(plan.config).toString("base64"),
        configSha256: hash(Buffer.from(plan.config)),
        outputBase64: null,
        outputSha256: null,
        outputReadState: "pending",
        outputReadError: null,
        stdoutBase64: stdout.toString("base64"),
        stderrBase64: stderr.toString("base64"),
        exitStatus: actual.status,
        signal: actual.signal,
        processError: actual.error?.message ?? null,
      };
      const journal = path.join(directory, `${plan.id}-repeat-${repeat}.json`);
      const persist = () => fs.writeFileSync(journal, `${JSON.stringify(observation, null, 2)}\n`);
      observations.push(observation);
      persist(); // Save actual complete process streams before accessing its output file.
      let output: Buffer | null = null;
      try {
        output = fs.readFileSync(file);
        observation.outputBase64 = output.toString("base64");
        observation.outputSha256 = hash(output);
        observation.outputReadState = "read";
      } catch (error) {
        observation.outputReadState = "failed";
        observation.outputReadError = String(error);
      }
      persist();
      try {
        assert.equal(actual.error, undefined);
        assert.equal(actual.signal, null);
        assert.equal(actual.status, plan.status);
        assert.equal(stdout.length, 0);
        assert(
          output && output.equals(plan.expected),
          "whole golden bytes or write protection changed",
        );
        const expectedStream = streams(plan);
        if (expectedStream)
          assert(stderr.equals(expectedStream), "complete authored CLI stream changed");
        else
          assert(stderr.length > 0, "error control must retain actual complete diagnostic bytes");
      } catch (error) {
        failures.push(`repeat ${repeat}: ${String(error)}`);
      }
    }
    const first = observations[0],
      second = observations[1];
    try {
      for (const facet of [
        "argv",
        "cwd",
        "inputBase64",
        "configBase64",
        "outputBase64",
        "outputReadState",
        "outputReadError",
        "stdoutBase64",
        "stderrBase64",
        "exitStatus",
        "signal",
        "processError",
      ])
        assert.deepEqual(first[facet], second[facet], `sorting repeat drift: ${facet}`);
    } catch (error) {
      failures.push(String(error));
    }
    return {
      id: `formatter/import-sorting/${plan.id}`,
      file: plan.file,
      expectedOutputBase64: plan.expected.toString("base64"),
      expectedOutputSha256: hash(plan.expected),
      observations,
      state: failures.length ? "failed" : "observed",
      failures,
      expectedStreamAuthority:
        plan.mode === "error"
          ? "complete raw repeated error observations only; no authored universal error reference or shared registration credit"
          : "existing repository CLI stream contract, fully compared",
      native: { state: "unsupported", reason: "native formatter adapter unavailable" },
      publicSharedCorpusCredit: 0,
    };
  };
  const rows: any[] = [];
  let captureError: string | null = null;
  try {
    for (const plan of plans) rows.push(capturePlan(plan));
  } catch (error) {
    captureError = String(error);
  }
  const report = {
    schema: "vize.formatter-import-sorting.cli-capture",
    version: 1,
    sourceRevision: identity.sourceRevision,
    sourceAuthority: {
      originalPath,
      originalSha256: hash(original),
      expectedPath,
      expectedSha256: hash(canonical),
      witnessPath,
      witnessSha256: hash(witness),
      witnessFunctions: [
        "configured_sfc_sorting_is_applied_by_write_and_detected_by_check",
        "standalone_script_receives_the_same_native_groups",
        "contradictory_sort_settings_cannot_write_authored_sources",
        "malformed_sort_settings_fail_closed_and_no_config_bypasses_them",
      ],
    },
    transport: {
      rule: "standalone script keeps exact bytes from first LF through first closing script tag, as authored Rust witness",
      input: { startByte: inputScript.startByte, endByte: inputScript.endByte },
      output: { startByte: outputScript.startByte, endByte: outputScript.endByte },
    },
    buildReceipt: receipt,
    captureError,
    rows,
    summary: {
      plannedControls: plans.length,
      actualObservedControls: rows.filter((r) => r.state === "observed").length,
      actualFailures: rows.filter((r) => r.state === "failed").length + plans.length - rows.length,
      sharedRegisteredCases: 0,
      nativeHandled: 0,
    },
    remaining7258: [
      "six other malformed-config forms and every invalid-config --no-config bypass",
      "shared public API option adapter and registration",
      "complete independent groups/comments/side-effect controls",
      "effectful config and ignored-file observation",
      "native-binding/Vite inheritance controls",
      "latest-history reconciliation and protected actual merge",
    ],
  };
  try {
    fs.writeFileSync(path.join(directory, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  } finally {
    fs.rmSync(workspaceRoot, { recursive: true, force: true });
  }
  assert.equal(captureError, null, "incomplete sorting capture retains process journals");
  assert.equal(
    report.summary.actualFailures,
    0,
    "sorting capture failures retain actual raw observations",
  );
  return report;
}
