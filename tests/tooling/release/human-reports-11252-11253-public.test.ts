import assert from "node:assert/strict";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { manifestBytes } from "../../../tools/support/release/human_reports_11252_11253_public/installation.ts";
import { loadOriginalFixture } from "../../../tools/support/release/human_reports_11252_11253_public/fixtures.ts";
import { authenticateRelease } from "../../../tools/support/release/human_reports_11252_11253_public/authority.ts";
import {
  beforeCommands,
  requireCaptureReady,
  requireComplete,
} from "../../../tools/support/release/human_reports_11252_11253_public/recipes.ts";
import {
  validateCapture,
  validateObservation,
} from "../../../tools/support/release/human_reports_11252_11253_public/processes.ts";
import {
  bytes,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import { assertRenderedTimings } from "../../../tools/support/release/human_reports_11252_11253_public/timing-oracle.ts";
import type {
  CommandPlan,
  InstallReceipt,
  Observation,
  Request,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
const encode = (v: string) => Buffer.from(v).toString("base64");
const nullFixture = () => loadOriginalFixture("runtime-prop-null-11252");
test("whole pinned owner fixtures are consumed without report re-extraction", () => {
  const original = nullFixture();
  assert.equal(original.files.length, 3);
  const mixed = loadOriginalFixture("imports-vue-ts-11253");
  assert.equal(mixed.files.length, 5);
  assert.equal(mixed.issue, 11253);
  for (const file of [...original.files, ...mixed.files])
    assert.equal(sha256(bytes(file.base64)), file.sha256);
});
test("original package is byte-exact before and has only the selected Vize version changed after", () => {
  const fixture = nullFixture(),
    raw = bytes(fixture.files.find((v) => v.path === "package.json")!.base64);
  assert(manifestBytes(fixture, "0.627.0", false).equals(raw));
  // Synthetic version exercises substitution only; no publication/version claim.
  assert.equal(
    manifestBytes(fixture, "9.0.0", false).toString(),
    raw.toString().replace('"vize": "0.627.0"', '"vize": "9.0.0"'),
  );
  const supplement = JSON.parse(manifestBytes(fixture, "0.627.0", true).toString()) as {
    devDependencies: Record<string, string>;
  };
  assert.equal(supplement.devDependencies.tsrs, "0.20.1-ts7.1.0-beta");
});
test("missing full source-owned oracles cannot become an executable request", () => {
  assert.throws(() =>
    requireComplete({
      schema: "vize.humanReports11252_11253.request.v1",
      role: "runtime-prop-null-11252",
      fixture: nullFixture(),
      commands: [],
      sourceAdmission: null,
    } as unknown as Request),
  );
});
test("missing actual merged after descriptors refuses before any GitHub/network adapter read", async () => {
  let reads = 0;
  await assert.rejects(
    authenticateRelease(
      {
        issue: 11252,
        phase: "after",
        version: "9.0.0",
        releaseRun: "1",
        capsuleBlob: "a".repeat(40),
        fixes: [],
      },
      async () => {
        reads++;
        return {};
      },
    ),
  );
  assert.equal(reads, 0);
});
test("CAPTURE_BEFORE requires no future merge/oracle and does not become qualification", () => {
  const request = {
    schema: "vize.humanReports11252_11253.request.v1",
    mode: "CAPTURE_BEFORE",
    reviewedController: {
      head: "a".repeat(40),
      files: [{ path: "synthetic", bytes: 1, sha256: "b".repeat(64), gitBlob: "c".repeat(40) }],
    },
    role: "runtime-prop-null-11252",
    fixture: nullFixture(),
    release: { issue: 11252, phase: "before" },
    commands: beforeCommands["runtime-prop-null-11252"].map((id) => ({ id, oracle: null })),
    sourceAdmission: null,
    oracleOwnerReceipt: null,
    publicBefore: null,
  } as unknown as Request;
  assert.equal(requireCaptureReady(request).issue, 11252);
  assert.throws(() => requireComplete({ ...request, mode: "ACCEPT_AFTER" }));
});
test("a selected issue's actual descriptor is independent from the other's pending fix", async () => {
  let reads = 0;
  await assert.rejects(
    authenticateRelease(
      {
        issue: 11252,
        phase: "after",
        version: "9.0.0",
        releaseRun: "1",
        capsuleBlob: "a".repeat(40),
        fixes: [
          {
            issue: 11252,
            pull: 11257,
            originalHead: "b".repeat(40),
            merge: "c".repeat(40),
            paths: ["synthetic-only"],
          },
        ],
      },
      async () => {
        reads++;
        return {};
      },
    ),
  );
  assert.equal(
    reads,
    1,
    "synthetic request reached the first tag read without requiring the other issue; it never authenticated a fake merge",
  );
});
function synthetic() {
  const image = { path: "/synthetic/native.node", bytes: 1, sha256: "a".repeat(64) };
  const checker = {
    state: "UNKNOWN",
    reason: "Synthetic contract only; no backend claim",
  } as const;
  const plan: CommandPlan = {
    id: "synthetic",
    consumer: "original",
    executable: "/synthetic/node",
    argv: ["synthetic-only"],
    sourceFiles: [],
    purpose: "control",
    adaptation: "synthetic contract test only",
    checker,
    requiresNativeLoad: true,
    oracle: {
      id: "synthetic",
      status: 1,
      stdout: { kind: "json", value: { errors: ["owned"] } },
      stderr: { kind: "bytes", base64: "" },
      sourceOwnedBy: { commit: "a".repeat(40), path: "synthetic", sha256: "b".repeat(64) },
    },
  };
  const install = {
    root: "/synthetic/consumer",
    native: { ...image, identityProbeOnly: true },
  } as InstallReceipt;
  const row: Observation = {
    id: plan.id,
    consumer: "original",
    pid: 10,
    executable: plan.executable,
    argv: plan.argv,
    cwd: install.root,
    status: 1,
    signal: null,
    error: null,
    stdoutBase64: encode('{"errors":["owned"]}'),
    stderrBase64: "",
    sourceFiles: [],
    environment: {
      NODE_OPTIONS: `--import=${pathToFileURL(fileURLToPath(new URL("../../../tools/support/release/human_reports_11252_11253_public/native-observer.ts", import.meta.url))).href}`,
    },
    instrumentation: "load-observed-companion",
    companionOf: plan.id,
    backend: "UNKNOWN",
    checker,
    nativeEvents: [
      {
        event: "initialized",
        commandId: plan.id,
        pid: 11,
        parentPid: 10,
        argv: ["actual-child"],
        nativePath: image.path,
        sha256: image.sha256,
      },
      {
        event: "attempt",
        commandId: plan.id,
        pid: 11,
        parentPid: 10,
        argv: ["actual-child"],
        nativePath: image.path,
        sha256: image.sha256,
      },
      {
        event: "returned",
        commandId: plan.id,
        pid: 11,
        parentPid: 10,
        argv: ["actual-child"],
        nativePath: image.path,
        sha256: image.sha256,
      },
    ],
  };
  return { plan, install, row };
}
test("synthetic direct process join can be checked; an independent image probe grants no child load credit", () => {
  const { plan, install, row } = synthetic();
  validateObservation(plan, row, install, true);
  assert.throws(
    () => validateObservation(plan, { ...row, nativeEvents: [] }, install, true),
    /actual product-process load join/,
  );
});
test("unobserved indirect ancestry, source overrides, and failed native load are refused", () => {
  const { plan, install, row } = synthetic();
  assert.throws(() =>
    validateObservation(
      plan,
      { ...row, nativeEvents: row.nativeEvents.map((v) => ({ ...v, parentPid: 99 })) },
      install,
      true,
    ),
  );
  assert.throws(() =>
    validateObservation(
      plan,
      { ...row, environment: { ...row.environment, CORSA_PATH: "/different/provider" } },
      install,
      true,
    ),
  );
  assert.throws(() =>
    validateObservation(
      plan,
      { ...row, nativeEvents: [...row.nativeEvents, { ...row.nativeEvents[0]!, event: "failed" }] },
      install,
      true,
    ),
  );
});
test("complete output/status oracle refuses extra diagnostics, stderr, or success substituted for error", () => {
  const { plan, install, row } = synthetic();
  assert.throws(() =>
    validateObservation(
      plan,
      { ...row, stdoutBase64: encode('{"errors":["owned","extra"]}') },
      install,
      true,
    ),
  );
  assert.throws(() =>
    validateObservation(plan, { ...row, stderrBase64: encode("warning") }, install, true),
  );
  assert.throws(() => validateObservation(plan, { ...row, status: 0 }, install, true));
});
test("before capture checks real process custody separately from an unprovided outcome oracle", () => {
  const { plan, install, row } = synthetic();
  const noOracle = { ...plan, oracle: null };
  const uninstrumented: Observation = {
    ...row,
    instrumentation: "none",
    nativeEvents: [],
    environment: {},
    companionOf: undefined,
  };
  validateCapture(noOracle, uninstrumented, install);
  validateCapture(noOracle, row, install, true);
  assert.throws(() => validateObservation(noOracle, uninstrumented, install), /no guessed/);
  assert.throws(() =>
    validateCapture(noOracle, { ...uninstrumented, nativeEvents: row.nativeEvents }, install),
  );
});
test("observed indirect PID edges join; a missing edge or different image refuses", () => {
  const { plan, install, row } = synthetic();
  const events = row.nativeEvents.map((event) => ({
    ...event,
    pid: 33,
    parentPid: 44,
    ancestry: [
      { pid: 33, parentPid: 44, source: "process.ppid" as const },
      { pid: 44, parentPid: 10, source: "ps" as const },
    ],
  }));
  validateCapture(plan, { ...row, nativeEvents: events }, install, true);
  assert.throws(() =>
    validateCapture(
      plan,
      {
        ...row,
        nativeEvents: events.map((event) => ({ ...event, ancestry: event.ancestry.slice(0, 1) })),
      },
      install,
      true,
    ),
  );
  assert.throws(() =>
    validateCapture(
      plan,
      { ...row, nativeEvents: events.map((event) => ({ ...event, sha256: "b".repeat(64) })) },
      install,
      true,
    ),
  );
});
test("whole ordered CLI summary cannot mask extra diagnostics, counts, unknown slots or placement", () => {
  const literals = [
    "src/main.ts\n  error:4:7 [TS2322] owned\nType checked 3 files in ",
    " (collect: ",
    ", imports: ",
    ", gen: ",
    ", corsa: ",
    ")\n  2 error(s)\n",
  ];
  const fields = ["total", "collect", "imports", "gen", "corsa"].map((label) => ({
    label,
    maxNanoseconds: 600_000_000_000,
  })) as Parameters<typeof assertRenderedTimings>[2];
  const rendered =
    literals[0] +
    "132.54ms" +
    literals[1] +
    "8.69ms" +
    literals[2] +
    "0.00ns" +
    literals[3] +
    "5.84ms" +
    literals[4] +
    "116.64ms" +
    literals[5];
  assertRenderedTimings(Buffer.from(rendered), literals, fields);
  assert.throws(() =>
    assertRenderedTimings(
      Buffer.from(rendered.replace("Type checked", "error TS9999: message Type checked")),
      [literals[0]!.replace("Type checked", "error TS9999: message Type checked"), ...literals.slice(1)],
      fields,
    ),
  );
  assert.throws(() =>
    assertRenderedTimings(
      Buffer.from(rendered.replace("2 error(s)", "1 error(s)")),
      literals,
      fields,
    ),
  );
  assert.throws(() => assertRenderedTimings(Buffer.from(rendered + "EXTRA"), literals, fields));
  assert.throws(() =>
    assertRenderedTimings(Buffer.from(rendered), literals, [
      { ...fields[0]!, label: "diagnosticCount" },
      ...fields.slice(1),
    ] as unknown as typeof fields),
  );
  assert.throws(() =>
    assertRenderedTimings(Buffer.from(rendered), ["diagnostic: ", ...literals.slice(1)], fields),
  );
});
