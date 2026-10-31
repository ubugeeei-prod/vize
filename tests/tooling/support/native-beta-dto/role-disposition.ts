import assert from "node:assert/strict";
import { historicalSource } from "./stable-watcher-boundary.ts";

type RegistrationObservation = { observation: { registrationPass: boolean } };
interface DispositionInput {
  source: string;
  role: string;
  sdk: string;
  baseline: boolean;
  stableRegistrationBaseline: boolean;
  registrationRefusals: number;
  nativeOwners: number;
  watcher: readonly { boundary: string }[];
  split: readonly RegistrationObservation[];
  terminal: readonly RegistrationObservation[];
  rootWarnings: readonly { observation: { kind: string; stderrOutputComplete: boolean } }[];
}

/** Format already-verified observations without treating a baseline transcript as credit. */
export function buildRoleDisposition(input: DispositionInput) {
  const {
    source,
    role,
    sdk,
    baseline,
    stableRegistrationBaseline,
    registrationRefusals,
    nativeOwners,
  } = input;
  const splitWatchLogCount = input.split.length,
    terminalStderrCount = input.terminal.length;
  const rootWatchWarningCount = input.rootWarnings.length;
  const unprovenEofOrder = input.split.some((row) => !row.observation.registrationPass);
  const terminalBeforeExitRequest = input.terminal.some((row) => !row.observation.registrationPass);
  const lateRoot = input.watcher.some(
    (row) => row.boundary === "post-shutdown-root-and-sdk-program-files",
  );
  const rootWarningUnknown = input.rootWarnings.some(
    (row) => !row.observation.stderrOutputComplete,
  );
  if (rootWatchWarningCount) {
    assert.equal(source, historicalSource);
    assert.equal(role, "before");
    assert.equal(sdk, "7.0.2");
    assert.equal(stableRegistrationBaseline, true);
    assert.equal(baseline, false);
  }
  return {
    source,
    role,
    sdk,
    roleKind: rootWatchWarningCount
      ? "baseline-observation-root-watch-warning"
      : baseline
        ? "baseline-observation-known-dto-refusal"
        : stableRegistrationBaseline
          ? terminalStderrCount
            ? terminalBeforeExitRequest
              ? "baseline-observation-unavailable-stderr-before-exit-request"
              : "baseline-observation-unobserved-terminal-stderr"
            : splitWatchLogCount
              ? unprovenEofOrder
                ? "baseline-observation-unproven-eof-return"
                : "baseline-observation-split-watch-log"
              : lateRoot
                ? "baseline-observation-late-root-and-unanswered-sdk-watcher"
                : registrationRefusals === 1
                  ? "baseline-observation-known-post-shutdown-registration-refusal"
                  : "baseline-observation-registration-acknowledged"
          : "functional-source-control",
    observerCompleted: true,
    ...(lateRoot ? { nativeQualification: false } : {}),
    dtoFunctionalPass:
      !baseline &&
      registrationRefusals === 0 &&
      splitWatchLogCount === 0 &&
      terminalStderrCount === 0 &&
      rootWatchWarningCount === 0,
    lifecyclePass:
      !baseline &&
      registrationRefusals === 0 &&
      splitWatchLogCount === 0 &&
      terminalStderrCount === 0 &&
      rootWatchWarningCount === 0,
    registrationPass:
      registrationRefusals === 0 &&
      !unprovenEofOrder &&
      !terminalBeforeExitRequest &&
      rootWatchWarningCount === 0,
    nativeOwners,
    allFramesClosed: true,
    ...(unprovenEofOrder
      ? { nativeQualification: false, observedRegistrationAcknowledged: registrationRefusals === 0 }
      : {}),
    ...(terminalBeforeExitRequest ? { observedRootRegistrationAcknowledged: true } : {}),
    ...(rootWatchWarningCount
      ? {
          nativeQualification: false,
          observedRootRegistrationAcknowledged: true,
          rootWatchWarningKinds: input.rootWarnings.map((row) => row.observation.kind),
        }
      : {}),
    ...(terminalStderrCount || rootWarningUnknown
      ? { stderrOutputComplete: false, rawIOComplete: false, nativeQualification: false }
      : {}),
  };
}
