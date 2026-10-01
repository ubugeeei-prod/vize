import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

function asset(root: string, declared: any) {
  assert.equal(typeof declared.path, "string");
  assert(!path.isAbsolute(declared.path));
  const file = fs.realpathSync(path.resolve(root, declared.path));
  const relative = path.relative(fs.realpathSync(root), file);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const bytes = fs.readFileSync(file);
  assert.equal(sha256(bytes), declared.sha256, "captured asset changed");
  return bytes;
}

export function loadFormatterCaptureReceipt(root: string, manifest: any) {
  if (!manifest.captureReceipt) return null;
  const receipt = JSON.parse(asset(root, manifest.captureReceipt).toString());
  assert.equal(receipt.schema, "vize.formatter-history-capture");
  assert.equal(receipt.version, 2);
  assert.equal(receipt.source.sourceRevision, manifest.source.revision);
  assert.equal(receipt.source.formatterSourceTree, manifest.source.formatterSourceTree);
  assert.equal(receipt.source.cargoLockSha256, manifest.source.cargoLockSha256);
  assert.match(receipt.originalCaptureSha256, /^[a-f0-9]{64}$/);
  assert.match(receipt.fullEvidenceArchiveSha256, /^[a-f0-9]{64}$/);
  assert.deepEqual(
    receipt.rows.map((row: any) => row.id),
    manifest.cases.map((row: any) => row.id),
  );
  const tables = JSON.parse(asset(root, receipt.tables).toString());
  assert.equal(tables.schema, "vize.formatter-capture-tables");
  assert.equal(tables.version, 2);
  const build = JSON.parse(asset(root, tables.build).toString());
  assert.equal(build.schema, "vize.formatter-capture-build");
  assert.equal(build.version, 2);
  assert.deepEqual(build.source, receipt.source);
  assert.equal(build.fullEvidenceArchiveSha256, receipt.fullEvidenceArchiveSha256);
  assert.match(build.originalReceiptSha256, /^[a-f0-9]{64}$/);
  assert.match(build.artifact.sha256, /^[a-f0-9]{64}$/);
  assert.equal(build.artifact.name, "formatter_observe");
  assert.deepEqual(build.artifact.kind, ["example"]);
  assert.equal(build.artifact.profile.test, false);
  assert.deepEqual(build.artifact.features, []);
  assert.equal(build.exitStatus, 0);
  assert.deepEqual(build.command, [
    "cargo",
    "build",
    "--locked",
    ...(build.offline ? ["--offline"] : []),
    "--profile",
    build.profileName,
    "-p",
    "vize_glyph",
    "--example",
    "formatter_observe",
    "--message-format=json-render-diagnostics",
  ]);
  for (const entry of build.options) {
    assert.equal(entry.exitStatus, 0);
    asset(root, entry.stdout);
  }
  return { receipt, tables, build };
}

// Identical recorded repetitions share a trace with an explicit multiplicity.
// Complete stdout bytes live in the immutable expected asset; its recorded hash
// binds every occurrence. Short stderr and full option-probe bytes are shared
// immutable assets. Full old raw JSON is retained by the named archive hash.
export function validateFormatterCapturedCase(
  root: string,
  capture: any,
  fixture: any,
  options: any,
  api: string,
  input: Buffer,
  output: Buffer,
) {
  if (!capture) return;
  const { receipt, tables } = capture;
  const row = receipt.rows.find((item: any) => item.id === fixture.id);
  assert(row);
  assert.equal(row.inputSha256, sha256(input));
  assert.equal(row.outputSha256, sha256(output));
  const error = fixture.outcome === "error";
  const internal = fixture.profile === "skip_script_stabilization";

  function process(ref: string, status: number) {
    assert(Object.hasOwn(tables.processes, ref), "missing actual process state");
    assert.deepEqual(tables.processes[ref], {
      exitStatus: status,
      signal: null,
      processError: null,
    });
  }
  function stderr(ref: string) {
    assert(Object.hasOwn(tables.streams, ref), "missing captured stream");
    return asset(root, tables.streams[ref]);
  }
  function trace(observed: any, argv: string[], status: number, expectedStderr: Buffer) {
    assert(Object.hasOwn(tables.arguments, observed.argv), "missing actual argv");
    assert.deepEqual(tables.arguments[observed.argv], argv);
    process(observed.process, status);
    assert(stderr(observed.stderr).equals(expectedStderr), "captured stderr changed");
  }
  function expectedStderr(bytes: Buffer) {
    return error
      ? Buffer.from(`error=${fixture.typedError}\n`)
      : fixture.api === "format_sfc"
        ? Buffer.from(`changed=${!bytes.equals(output)}\n`)
        : Buffer.alloc(0);
  }
  assert(Object.hasOwn(tables.probes, row.probe), "missing actual options probe");
  const probe = tables.probes[row.probe];
  assert.deepEqual(probe.argv, ["--options-json", ...options.flags]);
  process(probe.process, 0);
  assert.equal(stderr(probe.stderr).length, 0);
  assert.deepEqual(JSON.parse(asset(root, probe.stdout).toString()), options.effective);
  if (error) {
    assert.equal(row.initial.stdout, "empty");
    const firstError = stderr(row.initial.stderr);
    assert(firstError.toString().startsWith(`Error: ${fixture.typedError}(`));
    trace(row.initial, [api, ...options.flags], 1, firstError);
  } else {
    assert.equal(row.initial.stdout, "output");
    trace(row.initial, [api, ...options.flags], 0, expectedStderr(input));
  }
  assert.equal(row.repeat.count, 2);
  trace(
    row.repeat,
    [api, ...options.flags, ...(error ? ["--expect-error"] : [])],
    error ? 1 : 0,
    expectedStderr(input),
  );
  if (error || internal) assert.equal(row.fixedPoint, undefined);
  else {
    assert.equal(row.fixedPoint.count, 2);
    trace(row.fixedPoint, [api, ...options.flags], 0, expectedStderr(output));
  }
}
