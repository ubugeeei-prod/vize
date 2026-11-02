import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { assertCustody } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import { requireEnvironment, type Profile } from "./contract.ts";
import {
  capture,
  finalCustodiesBeforeAssessment,
  inputImages,
  recorder,
  safeFailure,
  whole,
  type Packet,
} from "./execute.ts";
import {
  genericArrowArgv,
  genericArrowData,
  genericArrowFiles,
  type GenericArrowState,
} from "./generic-arrow-data.ts";
import { install } from "./install.ts";
import { sourceBin } from "./routes.ts";
import { sourceCustody } from "./source.ts";

export const genericArrowLimitations = {
  scope:
    "Fresh installed TSC generic-arrow ten-state minimal reproducer; authored strict config/package/argv completion",
  host: "Linux x64 Node26.13.0/npm11.19.1 adaptation; reported M4/macOS15.6.1/Node22.23.3 not reproduced",
  originalSourceOwned: "exact five-line App.vue and declared syntax/typed-negative controls",
  privateAppOrPerformanceQualified: false,
  officialAlpha3ParityQualified: false,
  sourceDev71OrOtherVueOutputsTransferred: false,
  wholeNodeModulesFilesetQualified: false,
  selectedCompilerPid: "UNKNOWN",
  selectedCompilerImageObserved: false,
};
export function assessGenericArrow(state: GenericArrowState, packet: Packet) {
  whole(packet);
  if (state.expectedTsc.exit === "ZERO") assert.equal(packet.status, 0);
  else
    assert(
      Number.isSafeInteger(packet.status) && Number(packet.status) > 0,
      "declared nonzero diagnostic outcome required",
    );
  assert.equal(packet.stdout, state.expectedTsc.stdout, "whole intended TSC stdout");
  assert.equal(packet.stderr, state.expectedTsc.stderr, "whole intended TSC stderr");
  return {
    state: state.id,
    actualExitStatus: packet.status,
    expectedExitCriterion: state.expectedTsc.exit,
    wholeDiagnosticStreamsMatchIntent: true,
    exactNonzeroNumericStatusOwned: false,
    publicAcceptance: false,
  };
}
export async function runGenericArrow(
  options: { out: string; npm: string; npx: string },
  profile: Profile,
) {
  assert.equal(profile.subset, "GArrowTsc");
  const data = genericArrowData();
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    own = sourceCustody(),
    captured: { state: GenericArrowState; tsc: Packet }[] = [];
  const finals: { id: string; verify: () => void }[] = [];
  let failure: ReturnType<typeof safeFailure> | null = null;
  try {
    requireEnvironment(process.env, (rows) =>
      record.json("initial-environment-presence.json", rows),
    );
    const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    assert.equal(head, process.env.GITHUB_SHA);
    record.json("controller.json", {
      head,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      profile,
      own,
      sourceAuthority: data.sourceAuthority,
      actualFrontend: "typescript@7.1.0-beta tsc",
      command: ["tsc", ...genericArrowArgv],
      authoredPackageConfigAndArgvCompletion: true,
      environment: requireEnvironment(process.env),
      limitations: genericArrowLimitations,
    });
    const authorityPath = join(options.out, "authority");
    mkdirSync(authorityPath);
    const authority = await authenticate("after", recorder(authorityPath));
    const unit = "generic-arrow-tsx",
      out = join(options.out, unit);
    mkdirSync(out);
    const owned = recorder(out),
      root = join(dirname(options.out), unit);
    try {
      let files = genericArrowFiles(data.states[0]!, profile);
      const installed = await install(
        root,
        files,
        options.npm,
        options.npx,
        authority,
        owned,
        new Map(),
        ["tsc"],
      );
      const binding = sourceBin(root, "tsc", installed.custody);
      const verify = () => {
        assertCustody(own);
        installed.verify();
        inputImages(root, files);
      };
      finals.push({ id: unit, verify });
      for (const [index, state] of data.states.entries()) {
        verify();
        const next = genericArrowFiles(state, profile);
        assert.deepEqual(
          next.map((file) => file.path),
          files.map((file) => file.path),
        );
        for (const [position, file] of next.entries()) {
          if (file.path === "src/App.vue") writeFileSync(join(root, file.path), file.source);
          else assert.equal(file.source, files[position]!.source);
        }
        files = next;
        const label = "state-" + index;
        owned.json(label + ".input.json", { id: state.id, files: inputImages(root, files) });
        const tsc = capture(
          owned,
          label + ".tsc",
          [process.execPath, binding.target, ...genericArrowArgv],
          root,
          verify,
        );
        captured.push({ state, tsc });
        owned.json(label + ".complete.json", { id: state.id, tsc, publicAcceptance: false });
        inputImages(root, files);
      }
    } catch (error) {
      failure = safeFailure(error);
      owned.json("project-failure.json", failure);
    }
    record.json("all-complete-observations.json", { captured, failure, publicAcceptance: false });
    const custody = finalCustodiesBeforeAssessment(finals, record);
    assert.equal(finals.length, 1);
    assert(custody.every((row) => row.retainedRouteAndInputs));
    assert.equal(failure, null);
    assert.equal(captured.length, data.states.length);
    const outcomes = captured.map(({ state, tsc }) => {
      try {
        return { id: state.id, result: assessGenericArrow(state, tsc), failure: null };
      } catch (error) {
        return { id: state.id, result: null, failure: safeFailure(error) };
      }
    });
    record.json("result.json", {
      status: "COMPLETE_TSC_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      actualFrontend: "TSC",
      tsrsOrVizeCheckExecuted: false,
      referencePacketsRetained: 0,
      limitations: genericArrowLimitations,
      accepted: false,
      publicAcceptance: false,
    });
    assert(
      outcomes.every((row) => row.failure === null),
      "declared whole TSC intent differs; actual raw remains authoritative",
    );
  } catch (error) {
    record.json("failure.json", { failure: safeFailure(error), publicAcceptance: false });
    throw error;
  }
}
