import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertApiReport,
  assertSummary,
  type ApiCapture,
  type VrtReport,
  type VrtSummary,
} from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";

// Inert envelope laws: no product, compiler, renderer, HTTP API or native provider is substituted.
function envelopes(status: "error" | "new" | "passed" | "failed" = "passed") {
  const shared = {
    artPath: "/guard-only/src/left/Button.art.vue",
    variantName: "Default",
    viewport: "authored-compact",
    snapshotPath: "/guard-only/baselines/left.png",
    ...(status === "error"
      ? { error: "guard-only capture error" }
      : { currentPath: "/guard-only/current/left.png" }),
    ...(status === "failed" ? { diffPath: "/guard-only/diff/left.png" } : {}),
    ...(["passed", "failed"].includes(status)
      ? { diffPercentage: status === "passed" ? 0 : 1 }
      : {}),
  };
  const summary: VrtSummary = {
    total: 1,
    passed: Number(status === "passed"),
    failed: Number(status === "failed"),
    new: Number(status === "new"),
    errors: Number(status === "error"),
    skipped: 0,
    duration: 17,
  };
  const packet: ApiCapture = {
    success: true,
    summary: { ...summary },
    results: [
      {
        ...shared,
        passed: status === "passed" || status === "new",
        ...(status === "new" ? { isNew: true } : {}),
      },
    ],
    artifacts: {
      snapshotDir: "/guard-only/baselines",
      jsonReportPath: "/guard-only/report.json",
      htmlReportPath: "/guard-only/report.html",
    },
  };
  const report: VrtReport = {
    summary: { ...summary },
    results: [
      {
        ...shared,
        art: "Button",
        variant: "Default",
        status,
        ...(["passed", "failed"].includes(status)
          ? { diffPixels: status === "passed" ? 0 : 576, totalPixels: 57600 }
          : {}),
      },
    ],
  };
  return { packet, report };
}
const rejects = (report: VrtReport, packet: ApiCapture) =>
  assert.throws(() => assertApiReport(report, packet, 320, 180), { name: "AssertionError" });

test("whole physical JSON summaries and each shared field bind exactly to the parsed packet", async (t) => {
  const directory = await mkdtemp(path.join(os.tmpdir(), "vize-vrt-report-law-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  for (const status of ["error", "new", "passed", "failed"] as const) {
    const { packet, report } = envelopes(status);
    const file = path.join(directory, `${status}.json`);
    await writeFile(file, JSON.stringify(report));
    const written = JSON.parse(await readFile(file, "utf8"));
    assert.doesNotThrow(() =>
      assertApiReport(written, JSON.parse(JSON.stringify(packet)), 320, 180),
    );
    assert.equal(written.results[0].status, status);
  }
  for (const key of [
    "total",
    "new",
    "passed",
    "failed",
    "errors",
    "skipped",
    "duration",
  ] as const) {
    const { packet, report } = envelopes();
    report.summary[key]++;
    rejects(report, packet);
  }
  for (const [key, changed] of Object.entries({
    artPath: "/guard-only/src/right/Button.art.vue",
    variantName: "Wrong",
    viewport: "different",
    snapshotPath: "/foreign.png",
    currentPath: "/foreign-current.png",
    diffPath: "/unexpected-diff.png",
    diffPercentage: 2,
    error: "unexpected capture error",
  })) {
    const { packet, report } = envelopes();
    Reflect.set(report.results[0], key, changed);
    rejects(report, packet);
  }
  const omitted = envelopes();
  delete omitted.report.results[0].currentPath;
  rejects(omitted.report, omitted.packet);
});

test("report display aliases, explicit flag status and physical pixel arithmetic are independent guards", () => {
  for (const [key, changed] of Object.entries({
    art: "Left",
    variant: "Wrong",
    status: "failed",
    diffPixels: 1,
    totalPixels: 57599,
  })) {
    const { packet, report } = envelopes();
    Reflect.set(report.results[0], key, changed);
    rejects(report, packet);
  }
  const normalized = envelopes();
  const objectViewport = { name: "authored-compact", width: 320, height: 180 };
  Reflect.set(normalized.packet.results[0], "viewport", objectViewport);
  Reflect.set(normalized.report.results[0], "viewport", objectViewport);
  rejects(normalized.report, normalized.packet);
  for (const status of ["error", "new", "passed", "failed"] as const) {
    const { packet, report } = envelopes(status);
    report.results[0].status = status === "failed" ? "passed" : "failed";
    rejects(report, packet);
  }
  for (const changed of [-1, 57601, 0.5, Number.NaN]) {
    const { packet, report } = envelopes();
    report.results[0].diffPixels = changed;
    rejects(report, packet);
  }
});

test("new API and CLI phases pin every counter and exact new/passed result flags", () => {
  const expected = { new: 1, passed: 0, failed: 0 };
  const { packet, report } = envelopes("new");
  assert.doesNotThrow(() => assertSummary(packet, 1, expected));
  assert.doesNotThrow(() => assertSummary(report, 1, expected));
  for (const key of ["total", "new", "passed", "failed", "errors", "skipped"] as const) {
    for (const envelope of [structuredClone(packet), structuredClone(report)]) {
      envelope.summary[key]++;
      assert.throws(() => assertSummary(envelope, 1, expected), { name: "AssertionError" });
    }
  }
  for (const [key, changed] of [
    ["passed", false],
    ["isNew", false],
    ["diffPercentage", 0],
  ] as const) {
    const altered = structuredClone(packet);
    Reflect.set(altered.results[0], key, changed);
    assert.throws(() => assertSummary(altered, 1, expected), { name: "AssertionError" });
  }
  const missingFlag = structuredClone(packet);
  delete missingFlag.results[0].isNew;
  assert.throws(() => assertSummary(missingFlag, 1, expected), { name: "AssertionError" });
  const falseStatus = structuredClone(report);
  falseStatus.results[0].status = "passed";
  assert.throws(() => assertSummary(falseStatus, 1, expected), { name: "AssertionError" });
});
