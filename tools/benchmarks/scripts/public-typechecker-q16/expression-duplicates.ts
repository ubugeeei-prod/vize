import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { assertCustody } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import { requireEnvironment, type Profile, type SourceFile } from "./contract.ts";
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
  expressionDuplicatesArgs,
  expressionDuplicatesData,
  expressionDuplicatesFiles,
  expressionDuplicatesReferenceArgs,
  type ExpressionState,
} from "./expression-duplicates-data.ts";
import { install } from "./install.ts";
import { sourceCustody } from "./source.ts";

export const expressionDuplicatesLimitations = {
  scope:
    "Fresh installed four-state expression duplicate reproducer; exact six reported files, authored repairs and one independent TS2322",
  host: "Linux x64 Node26.12.1 adaptation; reported macOS26.4.1 arm64 is not reproduced",
  manager:
    "npm11.19.1/npm i/npx adaptation of pnpm12.3.4 install/exec; original inner checker argv retained",
  statusLaw:
    "Prospectively adopted published TSRS command-profile clean0/diagnostic2/empty stderr; reporter numeric status and stderr unstated",
  originalReference:
    "Only original joint project vue-tsc full raw; three diagnostics are reporter intent, status and whole reference text unassessed",
  wholeNodeModulesFilesetQualified: false,
  sourceObservationsTransferred: false,
  selectedCompilerPid: "UNKNOWN",
  selectedCompilerImageObserved: false,
};
export function writeExpressionDuplicatesInputs(
  root: string,
  current: SourceFile[],
  next: SourceFile[],
  profile: Profile,
) {
  inputImages(root, current);
  assert.equal(current[0]!.path, "package.json");
  assert.equal(next[0]!.path, "package.json");
  assert(current[0]!.source === next[0]!.source, "registered package must stay unchanged");
  assert(
    expressionDuplicatesData().states.some(
      (state) => JSON.stringify(expressionDuplicatesFiles(state, profile)) === JSON.stringify(next),
    ),
    "next source state must equal one frozen complete input",
  );
  const approved = new Set(
    expressionDuplicatesData().states.flatMap((s) => s.files.map((f) => f.path)),
  );
  for (const file of [...current, ...next]) assert(approved.has(file.path));
  for (const file of current)
    if (!next.some((f) => f.path === file.path)) {
      assert.equal(file.path, "src/Control.ts");
      unlinkSync(join(root, file.path));
    }
  for (const file of next)
    if (file.path !== "package.json") writeFileSync(join(root, file.path), file.source);
  inputImages(root, next);
}
export function assessExpressionDuplicates(state: ExpressionState, packet: Packet) {
  whole(packet);
  assert.equal(packet.status, state.status);
  assert.equal(packet.stdout, state.stdout);
  assert.equal(packet.stderr, "");
  return {
    state: state.id,
    status: packet.status,
    wholeStreamsOwned: true,
    publicAcceptance: false,
  };
}
export async function runExpressionDuplicates(
  options: { out: string; npm: string; npx: string },
  profile: Profile,
) {
  assert.equal(profile.subset, "ExpressionDuplicates11374");
  const data = expressionDuplicatesData();
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    own = sourceCustody(),
    captured: { state: ExpressionState; tsrs: Packet; reference: Packet | null }[] = [],
    finals: { id: string; verify: () => void }[] = [];
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
      environment: requireEnvironment(process.env),
      limitations: expressionDuplicatesLimitations,
    });
    const authorityPath = join(options.out, "authority");
    mkdirSync(authorityPath);
    const authority = await authenticate("after", recorder(authorityPath));
    const unit = "expression-duplicates",
      out = join(options.out, unit);
    mkdirSync(out);
    const owned = recorder(out),
      root = join(dirname(options.out), unit);
    try {
      let files = expressionDuplicatesFiles(data.states[0]!, profile);
      const installed = await install(
        root,
        files,
        options.npm,
        options.npx,
        authority,
        owned,
        new Map(),
      );
      const verify = () => {
        assertCustody(own);
        installed.verify();
        inputImages(root, files);
      };
      finals.push({ id: unit, verify });
      for (const [index, state] of data.states.entries()) {
        verify();
        const next = expressionDuplicatesFiles(state, profile);
        writeExpressionDuplicatesInputs(root, files, next, profile);
        files = next;
        const label = "state-" + index;
        owned.json(label + ".input.json", { id: state.id, files: inputImages(root, files) });
        const tsrs = capture(
          owned,
          label + ".tsrs",
          [process.execPath, options.npx, "tsrs", ...expressionDuplicatesArgs],
          root,
          verify,
        );
        const reference =
          index === 0
            ? capture(
                owned,
                label + ".reference",
                [process.execPath, options.npx, "vue-tsc", ...expressionDuplicatesReferenceArgs],
                root,
                verify,
              )
            : null;
        captured.push({ state, tsrs, reference });
        owned.json(label + ".complete.json", {
          id: state.id,
          tsrs,
          reference,
          publicAcceptance: false,
        });
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
    assert.equal(captured.length, 4);
    const outcomes = captured.map(({ state, tsrs, reference }, index) => {
      try {
        const result = assessExpressionDuplicates(state, tsrs);
        if (index === 0) {
          assert(reference);
          whole(reference);
        } else assert.equal(reference, null);
        return {
          id: state.id,
          result,
          referenceStatusOwned: false,
          referenceWholeTextOwned: false,
          failure: null,
        };
      } catch (error) {
        return { id: state.id, result: null, failure: safeFailure(error) };
      }
    });
    record.json("result.json", {
      status: "COMPLETE_EXPRESSION_DUPLICATE_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      referencePacketsRetained: 1,
      limitations: expressionDuplicatesLimitations,
      accepted: false,
      publicAcceptance: false,
    });
    assert(
      outcomes.every((row) => row.failure === null),
      "source-owned expression intent differs; complete actual streams remain authoritative",
    );
  } catch (error) {
    record.json("failure.json", { failure: safeFailure(error), publicAcceptance: false });
    throw error;
  }
}
