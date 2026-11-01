import assert from "node:assert/strict";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { CommandPlan, InstallReceipt, Observation } from "./model.ts";
import type { Oracle } from "./model.ts";
import { bytes, wholeJson } from "./value.ts";
import { assertRenderedTimings } from "./timing-oracle.ts";
import { nativeActors } from "./native-law.ts";
export const prohibited = [
  "NODE_OPTIONS",
  "NODE_PATH",
  "VIZE_NATIVE_BINDING_PATH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "TSRS_BINARY",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
] as const;
export function validateCapture(
  plan: CommandPlan,
  row: Observation,
  install: InstallReceipt,
  companion = false,
  archivedObserverPath?: string,
): void {
  assert.equal(row.id, plan.id);
  assert.equal(row.consumer, plan.consumer);
  assert.equal(row.executable, plan.executable);
  assert.deepEqual(row.argv, plan.argv);
  assert.equal(row.cwd, install.root);
  assert.equal(row.error, null);
  assert.equal(row.signal, null);
  assert.equal(row.backend, "UNKNOWN", "this adapter does not infer a selected backend");
  assert(Number.isSafeInteger(row.pid) && row.pid > 0);
  assert(Number.isInteger(row.status), "an actual completed process status is required");
  assert.deepEqual(row.sourceFiles, plan.sourceFiles);
  assert.deepEqual(row.checker, plan.checker);
  for (const key of prohibited) {
    const allowed =
      companion && key === "NODE_OPTIONS"
        ? `--import=${pathToFileURL(archivedObserverPath ?? fileURLToPath(new URL("./native-observer.ts", import.meta.url))).href}`
        : "";
    assert.equal(row.environment[key] ?? "", allowed, `provider/source override: ${key}`);
  }
  for (const [key, value] of Object.entries(install.packageManagerEnvironment ?? {}))
    assert.equal(row.environment[key], value);
  const initialized = row.nativeEvents.filter((v) => v.event === "initialized");
  const completed = nativeActors(row.nativeEvents, plan.id);
  for (const event of row.nativeEvents) {
    assert(Number.isSafeInteger(event.pid) && event.pid > 0);
    assert(Number.isSafeInteger(event.parentPid) && event.parentPid > 0);
    assert.equal(event.nativePath, install.native.path);
    assert.equal(event.sha256, install.native.sha256);
  }
  if (!companion) {
    assert.equal(row.instrumentation ?? "none", "none");
    assert.deepEqual(row.nativeEvents, []);
    return;
  }
  assert.equal(row.instrumentation, "load-observed-companion");
  assert.equal(row.companionOf, row.id);
  if (plan.requiresNativeLoad) {
    // A separate getNativeBuildToken()/require probe has no relationship to this
    // compiler/CLI process and never grants runtime load/backend credit here.
    assert(
      initialized.length > 0 && completed.length > 0,
      `${row.id}: actual product-process load join required`,
    );
    for (const event of completed) {
      const start = initialized.find((v) => v.pid === event.pid);
      assert(start);
      assert.deepEqual(start.argv, event.argv);
      assert.equal(start.parentPid, event.parentPid);
      if (event.pid !== row.pid && event.parentPid !== row.pid) {
        assert(event.ancestry && event.ancestry.length > 0 && event.ancestry.length <= 17);
        let pid = event.pid,
          joined = false;
        const seen = new Set<number>();
        for (const edge of event.ancestry) {
          assert.equal(edge.pid, pid);
          assert(!seen.has(pid));
          seen.add(pid);
          assert(["process.ppid", "ps"].includes(edge.source));
          assert(Number.isSafeInteger(edge.parentPid) && edge.parentPid > 0);
          if (edge.pid === row.pid || edge.parentPid === row.pid) {
            joined = true;
            break;
          }
          pid = edge.parentPid;
        }
        assert(joined, "indirect/unobserved process ancestry is not inferred");
      }
    }
    assert.equal(
      row.nativeEvents.some((v) => v.event === "failed"),
      false,
    );
  } else
    assert.equal(
      row.nativeEvents.length,
      0,
      "non-native reference process cannot inherit another command's load proof",
    );
}
export function validateObservation(
  plan: CommandPlan,
  row: Observation,
  install: InstallReceipt,
  companion = false,
  archivedObserverPath?: string,
): void {
  validateCapture(plan, row, install, companion, archivedObserverPath);
  assert(plan.oracle, "no guessed/partial oracle is permitted");
  compareOutcome(plan.oracle, row);
}
export function compareOutcome(oracle: Omit<Oracle, "sourceOwnedBy">, row: Observation): void {
  assert.equal(row.id, oracle.id);
  assert.equal(row.status, oracle.status);
  const stdout = bytes(row.stdoutBase64),
    stderr = bytes(row.stderrBase64);
  assert(stderr.equals(bytes(oracle.stderr.base64)), `${row.id}: whole stderr differs`);
  if (oracle.stdout.kind === "bytes")
    assert(stdout.equals(bytes(oracle.stdout.base64)), `${row.id}: whole stdout differs`);
  else if (oracle.stdout.kind === "json")
    assert.deepEqual(
      wholeJson(stdout),
      oracle.stdout.value,
      `${row.id}: whole JSON vector differs`,
    );
  else {
    assert.equal(oracle.stdout.kind, "rendered-timings", "unknown stdout oracle kind");
    assertRenderedTimings(stdout, oracle.stdout.literals, oracle.stdout.fields);
  }
}
