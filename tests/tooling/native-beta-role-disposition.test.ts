import assert from "node:assert/strict";
import { test } from "node:test";
import { buildRoleDisposition } from "./support/native-beta-dto/role-disposition.ts";
import { historicalSource } from "./support/native-beta-dto/stable-watcher-boundary.ts";

const candidate = () => ({
  source: "e".repeat(40),
  role: "after",
  sdk: "7.0.2",
  baseline: false,
  stableRegistrationBaseline: false,
  registrationRefusals: 0,
  nativeOwners: 4,
  watcher: [],
  split: [],
  terminal: [],
  rootWarnings: [],
});
test("empty new observations preserve complete existing candidate and beta-baseline dispositions", () => {
  assert.deepEqual(buildRoleDisposition(candidate()), {
    source: "e".repeat(40),
    role: "after",
    sdk: "7.0.2",
    roleKind: "functional-source-control",
    observerCompleted: true,
    dtoFunctionalPass: true,
    lifecyclePass: true,
    registrationPass: true,
    nativeOwners: 4,
    allFramesClosed: true,
  });
  assert.deepEqual(
    buildRoleDisposition({
      ...candidate(),
      source: historicalSource,
      role: "before",
      sdk: "7.1.0-beta",
      baseline: true,
      nativeOwners: 7,
    }),
    {
      source: historicalSource,
      role: "before",
      sdk: "7.1.0-beta",
      roleKind: "baseline-observation-known-dto-refusal",
      observerCompleted: true,
      dtoFunctionalPass: false,
      lifecyclePass: false,
      registrationPass: true,
      nativeOwners: 7,
      allFramesClosed: true,
    },
  );
});
test("old unproven and terminal observations retain their distinct registration facts", () => {
  const before = {
    ...candidate(),
    source: historicalSource,
    role: "before",
    stableRegistrationBaseline: true,
  };
  const split = buildRoleDisposition({
    ...before,
    split: [{ observation: { registrationPass: false } }],
  });
  assert.equal(split.roleKind, "baseline-observation-unproven-eof-return");
  assert.equal(split.registrationPass, false);
  assert.equal(split.observedRegistrationAcknowledged, true);
  assert.equal(split.nativeQualification, false);
  const oldTerminal = buildRoleDisposition({
    ...before,
    terminal: [{ observation: { registrationPass: true } }],
  });
  assert.equal(oldTerminal.roleKind, "baseline-observation-unobserved-terminal-stderr");
  assert.equal(oldTerminal.registrationPass, true);
  assert.equal(oldTerminal.rawIOComplete, false);
  const unavailable = buildRoleDisposition({
    ...before,
    terminal: [{ observation: { registrationPass: false } }],
  });
  assert.equal(unavailable.roleKind, "baseline-observation-unavailable-stderr-before-exit-request");
  assert.equal(unavailable.registrationPass, false);
  assert.equal(unavailable.observedRootRegistrationAcknowledged, true);
});
test("both new historical transcripts deny all credit while retaining their separate byte completeness", () => {
  for (const stderrOutputComplete of [true, false]) {
    const input = {
      ...candidate(),
      source: historicalSource,
      role: "before",
      stableRegistrationBaseline: true,
      rootWarnings: [
        {
          observation: {
            kind: stderrOutputComplete
              ? "completed-warning-unanswered-sdk"
              : "unobserved-warning-attempt",
            stderrOutputComplete,
          },
        },
      ],
    };
    const result = buildRoleDisposition(input);
    for (const key of [
      "nativeQualification",
      "dtoFunctionalPass",
      "lifecyclePass",
      "registrationPass",
    ] as const)
      assert.equal(result[key], false);
    assert.equal(result.observedRootRegistrationAcknowledged, true);
    assert.equal(result.roleKind, "baseline-observation-root-watch-warning");
    assert.equal(result.rawIOComplete, stderrOutputComplete ? undefined : false);
    for (const mutation of [
      { role: "after" },
      { source: "0".repeat(40) },
      { sdk: "7.1.0-beta" },
      { stableRegistrationBaseline: false },
      { baseline: true },
    ])
      assert.throws(() => buildRoleDisposition({ ...input, ...mutation }));
  }
});
