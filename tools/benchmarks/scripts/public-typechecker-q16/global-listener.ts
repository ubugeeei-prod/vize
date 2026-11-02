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
  healthy,
  inputImages,
  recorder,
  safeFailure,
  whole,
  type Packet,
} from "./execute.ts";
import {
  globalListenerArgs,
  globalListenerData,
  globalListenerFiles,
  globalListenerReferenceArgs,
  listenerNeedle,
  type ListenerState,
} from "./global-listener-data.ts";
import { install } from "./install.ts";
import { sourceCustody } from "./source.ts";

export const globalListenerLimitations = {
  scope:
    "Fresh installed fourteen-state global component listener reproducer; exact six reported files and source-owned controls",
  host: "Linux x64 Node26.12.1 adaptation; reported macOS26.4.1 arm64 is not reproduced",
  manager:
    "npm11.19.1/npm i/npx adaptation of pnpm12.3.4 install/exec; original inner checker argv retained",
  originalReference:
    "Only original joint project vue-tsc status0; whole reference printer text unclaimed",
  missingRequiredPrinterTextOwned: false,
  wholeNodeModulesFilesetQualified: false,
  sourceObservationsTransferred: false,
  selectedCompilerPid: "UNKNOWN",
  selectedCompilerImageObserved: false,
};
export function writeGlobalListenerInputs(
  root: string,
  current: SourceFile[],
  next: SourceFile[],
  profile: Profile,
) {
  inputImages(root, current);
  assert.equal(current[0]!.path, "package.json");
  assert.equal(next[0]!.path, "package.json");
  assert.equal(current[0]!.source, next[0]!.source);
  assert(
    globalListenerData().states.some(
      (state) => JSON.stringify(globalListenerFiles(state, profile)) === JSON.stringify(next),
    ),
    "next source state must equal one frozen complete input",
  );
  const approved = new Set(globalListenerData().states.flatMap((s) => s.files.map((f) => f.path)));
  for (const file of [...current, ...next]) assert(approved.has(file.path));
  for (const file of current)
    if (!next.some((f) => f.path === file.path)) {
      assert(file.path.startsWith("src/") && file.path.endsWith(".d.ts"));
      unlinkSync(join(root, file.path));
    }
  for (const file of next)
    if (file.path !== "package.json") writeFileSync(join(root, file.path), file.source);
  inputImages(root, next);
}
export function assessGlobalListener(state: ListenerState, packet: Packet) {
  whole(packet);
  assert.equal(typeof packet.stdout, "string");
  assert.equal(packet.status, state.status);
  assert.equal(packet.stderr, "");
  if (!state.diagnostics.length) {
    assert.equal(packet.stdout, "");
    return {
      state: state.id,
      status: packet.status,
      cleanWholeStreams: true,
      publicAcceptance: false,
    };
  }
  assert.equal(state.diagnostics.length, 1);
  const expected = state.diagnostics[0]!,
    lines = packet.stdout!.split("\n");
  assert.equal(lines.pop(), "");
  assert(lines.length > 0 && lines.length <= 32);
  const header =
    /^([A-Za-z0-9_./-]+)\(([1-9][0-9]*),([1-9][0-9]*)\): error TS([0-9]+): (.+)$/u.exec(lines[0]!);
  assert(header);
  assert.equal(header[1], expected.file);
  assert.equal(Number(header[4]), expected.code);
  const needle = listenerNeedle(state, expected);
  assert.equal(Number(header[2]), needle.line);
  assert(Number(header[3]) >= needle.startColumn && Number(header[3]) < needle.endColumnExclusive);
  if (expected.missingRequiredProperty === null) {
    assert.equal(lines.length, 1);
    assert.equal(header[5], expected.message);
  } else {
    assert.equal(state.id, "missing-required-global-prop");
    assert.equal(expected.code, 2345);
    assert(
      /^Argument of type '.+' is not assignable to parameter of type '.+'\.$/u.test(header[5]!),
    );
    assert(lines.length >= 2);
    assert(lines.slice(1).every((line) => /^ {2,}\S/u.test(line)));
    assert(
      lines
        .slice(1)
        .some(
          (line) =>
            line.startsWith("  Property 'to' is missing in type '") &&
            line.includes("' but required in type '") &&
            line.endsWith("'."),
        ),
    );
  }
  return {
    state: state.id,
    status: packet.status,
    code: expected.code,
    authoredNeedle: needle,
    wholePrimitiveMessageOwned: expected.message !== null,
    missingRequiredToOwned: expected.missingRequiredProperty === "to",
    generatedReceiverAndOutputEndOwned: false,
    wholeActualStreamsRetained: true,
    publicAcceptance: false,
  };
}
export async function runGlobalListener(
  options: { out: string; npm: string; npx: string },
  profile: Profile,
) {
  assert.equal(profile.subset, "GlobalListener11373");
  const data = globalListenerData();
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    own = sourceCustody(),
    captured: { state: ListenerState; tsrs: Packet; reference: Packet | null }[] = [],
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
      limitations: globalListenerLimitations,
    });
    const authorityPath = join(options.out, "authority");
    mkdirSync(authorityPath);
    const authority = await authenticate("after", recorder(authorityPath));
    const unit = "global-component-listener",
      out = join(options.out, unit);
    mkdirSync(out);
    const owned = recorder(out),
      root = join(dirname(options.out), unit);
    try {
      let files = globalListenerFiles(data.states[0]!, profile);
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
        const next = globalListenerFiles(state, profile);
        writeGlobalListenerInputs(root, files, next, profile);
        files = next;
        const label = "state-" + index;
        owned.json(label + ".input.json", { id: state.id, files: inputImages(root, files) });
        const tsrs = capture(
          owned,
          label + ".tsrs",
          [process.execPath, options.npx, "tsrs", ...globalListenerArgs],
          root,
          verify,
        );
        const reference =
          index === 0
            ? capture(
                owned,
                label + ".reference",
                [process.execPath, options.npx, "vue-tsc", ...globalListenerReferenceArgs],
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
    assert.equal(captured.length, 14);
    const outcomes = captured.map(({ state, tsrs, reference }, index) => {
      try {
        const result = assessGlobalListener(state, tsrs);
        if (index === 0) {
          assert(reference);
          healthy(reference, 0);
          assert(
            !/(?:^|\n).+\([0-9]+,[0-9]+\): error TS[0-9]+:/u.test(
              reference.stdout + "\n" + reference.stderr,
            ),
            "original reference must remain diagnostic-free",
          );
        } else assert.equal(reference, null);
        return {
          id: state.id,
          result,
          referenceStatusOwned: index === 0,
          referenceWholeTextOwned: false,
          failure: null,
        };
      } catch (error) {
        return { id: state.id, result: null, failure: safeFailure(error) };
      }
    });
    record.json("result.json", {
      status: "COMPLETE_LISTENER_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      referencePacketsRetained: 1,
      limitations: globalListenerLimitations,
      accepted: false,
      publicAcceptance: false,
    });
    assert(
      outcomes.every((row) => row.failure === null),
      "source-owned listener intent differs; complete actual streams remain authoritative",
    );
  } catch (error) {
    record.json("failure.json", { failure: safeFailure(error), publicAcceptance: false });
    throw error;
  }
}
