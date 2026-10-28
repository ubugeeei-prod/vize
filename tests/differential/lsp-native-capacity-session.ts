//! One closed historical admission. Generic differential parity remains unsupported.
import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
import { sha256 } from "./harness.mjs";
import type { FixtureData } from "./lsp-types.ts";
import {
  CapacityCoverage,
  capacitySource,
  type CapacityProbe,
} from "../tooling/support/lsp/diagnostic-capacity.ts";
import { CapacityNativeCoverage } from "../tooling/support/lsp/diagnostic-capacity-native.ts";
import {
  nativeIpcGrew,
  type IpcEvidence,
} from "../tooling/support/lsp/diagnostic-capacity-pipe.ts";
import { validateCapacityWire, type HistoricalReport } from "./lsp-native-capacity-wire.ts";
import {
  NATIVE_CAPACITY_ID,
  NATIVE_CAPACITY_FILES,
  NATIVE_CAPACITY_PINS,
  NATIVE_CAPACITY_SESSION,
  NATIVE_CAPACITY_INITIALIZATION,
  NATIVE_CAPACITY_PLAN,
  NATIVE_CAPACITY_PLAN_SHA256,
} from "./lsp-native-capacity-contract.ts";

export function validateNativeCapacitySourceSession(
  data: FixtureData,
  files: ReadonlyMap<string, Buffer>,
  plan: unknown,
): void {
  assert.deepEqual(
    Object.keys(data).sort(),
    [
      "version",
      "id",
      "provenance",
      "documentVersion",
      "openDocument",
      "initializationOptions",
      "files",
      "entry",
      "expected",
      "method",
      "sourceSession",
    ].sort(),
  );
  assert.equal(data.version, 1);
  assert.equal(data.id, NATIVE_CAPACITY_ID);
  assert.deepEqual(data.provenance, {
    fixCommit: NATIVE_CAPACITY_PLAN.after.revision,
    witness: {
      path: NATIVE_CAPACITY_SESSION.executor,
      sha256: NATIVE_CAPACITY_PINS["executor-capacity.ts.txt"].sha256,
    },
  });
  assert.deepEqual(data.expected, {
    source: "response.expected.json",
    sha256: NATIVE_CAPACITY_PLAN_SHA256,
  });
  assert.deepEqual(
    data.files,
    Object.entries(NATIVE_CAPACITY_FILES).map(([source, runtimePath]) => ({
      source,
      runtimePath,
      sha256: NATIVE_CAPACITY_PINS[source as keyof typeof NATIVE_CAPACITY_PINS].sha256,
    })),
  );
  assert.equal(data.entry, "Seed.vue");
  assert.equal(data.method, "source-session/native-diagnostic-capacity");
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, false);
  assert.deepEqual(data.sourceSession, NATIVE_CAPACITY_SESSION);
  assert.deepEqual(data.initializationOptions, NATIVE_CAPACITY_INITIALIZATION);
  assert.deepEqual(
    plan,
    NATIVE_CAPACITY_PLAN,
    "the operation plan and evidence identities are closed",
  );
  assert.deepEqual([...files.keys()], Object.values(NATIVE_CAPACITY_FILES));
  const read = (name: keyof typeof NATIVE_CAPACITY_FILES): Buffer => {
    const compressed = files.get(NATIVE_CAPACITY_FILES[name]);
    assert.ok(compressed);
    const pin = NATIVE_CAPACITY_PINS[name];
    assert.equal(sha256(compressed), pin.sha256);
    const raw = name.endsWith(".gz") ? gunzipSync(compressed) : compressed;
    assert.equal(raw.length, pin.rawBytes);
    assert.equal(sha256(raw), pin.rawSha256);
    return raw;
  };
  for (const name of Object.keys(NATIVE_CAPACITY_FILES) as (keyof typeof NATIVE_CAPACITY_FILES)[])
    read(name);
  const project = JSON.parse(read("project-input.json.gz").toString("utf8")) as {
    schema: string;
    version: number;
    count: number;
    probes: Omit<CapacityProbe, "uri">[];
  };
  assert.equal(project.schema, "vize.lsp.native-capacity.project");
  assert.equal(project.version, 1);
  assert.equal(project.count, 129);
  assert.equal(project.probes.length, 130);
  const omitUri = ({ uri: _uri, ...probe }: CapacityProbe) => probe;
  assert.equal(
    sha256(Buffer.from(JSON.stringify(project.probes))),
    NATIVE_CAPACITY_PLAN.normalizedInputSha256,
  );
  for (let index = 0; index < 130; index++) {
    const name = index === 0 ? "Seed.vue" : "Coverage" + String(index).padStart(4, "0") + ".vue";
    const expected = { ...omitUri(capacitySource(index, "file:///fixture/" + name)), file: name };
    assert.deepEqual(
      project.probes[index],
      expected,
      "whole invalid/repair sources and exact UTF16 probes",
    );
  }
  assert.equal(read("Seed.vue.txt").toString("utf8"), project.probes[0]?.invalid);
  const reopenInput = JSON.parse(read("reopen-input.json.gz").toString("utf8")) as {
    schema: string;
    version: number;
    probes: Record<"warm" | "old" | "next", Omit<CapacityProbe, "uri">>;
  };
  assert.equal(reopenInput.schema, "vize.lsp.native-capacity.reopen");
  assert.equal(reopenInput.version, 1);
  assert.equal(
    sha256(Buffer.from(JSON.stringify(reopenInput.probes))),
    NATIVE_CAPACITY_PLAN.normalizedReopenSha256,
  );
  const checkNative = (
    report: HistoricalReport,
    probes: CapacityProbe[],
    expected: readonly number[],
    repaired: readonly boolean[],
  ) => {
    assert.equal(report.cleanup.complete, true);
    assert.deepEqual(report.cleanup.errors, []);
    const coverage = new CapacityCoverage(probes);
    for (const row of report.packets) coverage.record(row.packet, row.atMs);
    const native = new CapacityNativeCoverage(coverage, () => report.stderr);
    for (let index = 0; index < 3; index++) {
      const rows = native.rows(index + 1, repaired[index] === true);
      assert.equal(rows.filter((row) => row.complete).length, expected[index]);
      assert.equal(
        rows.reduce((sum, row) => sum + row.nativeSendsOutsidePass, 0),
        0,
      );
    }
    assert.equal(native.idle(), true);
    return native;
  };
  const checkHeldRequest = (
    report: HistoricalReport,
    stage: string,
    beforeKey: string,
    afterKey: string,
  ) => {
    const event = report.events.find((event) => event.stage === stage);
    assert.ok(event);
    const evidence = event.value as Record<string, IpcEvidence>;
    const before = evidence[beforeKey],
      after = evidence[afterKey];
    assert.ok(before && after);
    assert.ok(before.processes.length > 0 && after.processes.length > 0);
    for (const process of [...before.processes, ...after.processes])
      assert.equal(process.state, "T");
    assert.equal(nativeIpcGrew(before, after), true);
  };
  for (const role of ["before", "after"] as const) {
    const report = JSON.parse(
      read(role === "before" ? "before-result.json.gz" : "after-result.json.gz").toString("utf8"),
    ) as HistoricalReport;
    const expected = NATIVE_CAPACITY_PLAN[role];
    assert.equal(report.producer.schema, "vize.differential.build");
    assert.equal(report.producer.version, 1);
    assert.equal(report.producer.sourceRevision, expected.revision);
    assert.equal(report.producer.binarySha256, expected.cliSha256);
    const probes = report.input as CapacityProbe[];
    assert.deepEqual(probes.map(omitUri), project.probes);
    assert.equal(report.qualified, role === "after");
    assert.equal(report.packets.length, expected.rawDiagnostics);
    assert.equal(report.reverseDependentCounts?.length, expected.zeroReverseDependentReceipts);
    assert.ok(
      report.reverseDependentCounts?.every((row) =>
        row.includes("refreshing 0 open typecheck dependents"),
      ),
    );
    checkHeldRequest(report, "seed-pass-inflight", "ipcBefore", "ipcAfter");
    const accepted = report.events.find(
      (event) => event.stage === "all-open-sync-jobs-accepted-native-still-blocked",
    );
    assert.ok(accepted);
    const burst = accepted.value as {
      accepted: { uri: string; syncPackets: number }[];
      startsBefore: number;
      startsAfter: number;
    };
    assert.deepEqual(
      burst.accepted.map((row) => row.uri),
      probes.slice(1).map((probe) => probe.uri),
    );
    assert.ok(burst.accepted.every((row) => row.syncPackets > 0));
    assert.equal(burst.startsBefore, 2);
    assert.equal(burst.startsAfter, 2);
    const native = checkNative(report, probes.slice(1), expected.coverage, [false, true, true]);
    if (role === "before") {
      const missing = native.rows(1, false).filter((row) => !row.complete);
      assert.deepEqual(
        missing.map((row) => row.file),
        project.probes.slice(1, 67).map((probe) => probe.file),
      );
      assert.ok(
        missing.every(
          (row) => row.nativeStarted === 0 && row.nativeSent === 0 && row.nativeFinished === 0,
        ),
      );
    }
    validateCapacityWire(
      report,
      read(role === "before" ? "before-client.bin.gz" : "after-client.bin.gz"),
      read(role === "before" ? "before-server.bin.gz" : "after-server.bin.gz"),
      false,
    );
  }
  const reopened = JSON.parse(read("reopen-result.json.gz").toString("utf8")) as HistoricalReport;
  const roles = reopened.input as Record<"warm" | "old" | "next", CapacityProbe>;
  assert.deepEqual(
    Object.fromEntries(Object.entries(roles).map(([key, value]) => [key, omitUri(value)])),
    reopenInput.probes,
  );
  assert.equal(reopened.producer.sourceRevision, NATIVE_CAPACITY_PLAN.reopen.revision);
  assert.equal(reopened.producer.binarySha256, NATIVE_CAPACITY_PLAN.after.cliSha256);
  assert.equal(reopened.qualified, true);
  assert.equal(reopened.packets.length, 10);
  checkHeldRequest(reopened, "old-version1-native-inflight", "before", "after");
  checkNative(reopened, [roles.next], [1, 1, 1], [true, false, true]);
  const event = reopened.events.find(
    (event) => event.stage === "same-version-reopen-local-buffer-confirmed",
  );
  assert.ok(event);
  const { reopenedOffset } = event.value as { reopenedOffset: number };
  assert.ok(Number.isSafeInteger(reopenedOffset) && reopenedOffset >= 0);
  assert.deepEqual(
    reopened.packets
      .slice(reopenedOffset)
      .filter(
        ({ packet }) =>
          packet.uri === roles.old.uri &&
          packet.diagnostics.some(
            (row) =>
              row.source === "vize/types" &&
              (packet.version === 1 || row.message?.includes(roles.old.token)),
          ),
      ),
    [],
  );
  validateCapacityWire(reopened, read("reopen-client.bin.gz"), read("reopen-server.bin.gz"), true);
}
