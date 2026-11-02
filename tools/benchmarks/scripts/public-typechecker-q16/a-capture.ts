import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { assertCustody } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import {
  filesFor,
  limitations,
  reportedCommands,
  requireEnvironment,
  subsetFor,
  type RegressionProfile,
  type SourceFile,
  type State,
} from "./contract.ts";
import { a65Data, a65Inputs, type A65State } from "./a-data.ts";
import {
  assessA65,
  resetA65Outputs,
  retainA65Outputs,
  verifyA65Outputs,
  requireSameA65Declarations,
  type A65Snapshot,
} from "./a65-results.ts";
import {
  assessTsrs,
  capture,
  finalCustodiesBeforeAssessment,
  inputImages,
  recorder,
  safeFailure,
  whole,
  type Packet,
} from "./execute.ts";
import { install } from "./install.ts";
import { sourceCustody } from "./source.ts";
export async function runA(
  options: { out: string; npm: string; npx: string },
  profile: RegressionProfile,
) {
  assert.equal(profile.subset, "A59-65");
  assert.equal(profile.fix, a65Data.fixMerge);
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    own = sourceCustody(),
    checkSource = () => assertCustody(own),
    cache = new Map<string, Buffer>(),
    final: { id: string; verify: () => void }[] = [];
  const a59: { state: State; tsrs: Packet; reference: Packet }[] = [],
    a65: { state: A65State; packet: Packet; outputs: A65Snapshot }[] = [],
    errors: { id: string; failure: ReturnType<typeof safeFailure> | null }[] = [];
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
      subset: "A59-65",
      own,
      environment: requireEnvironment(process.env),
      a65SourceAuthority: a65Data.sourceAuthority,
      limitations,
      sourceDeclarationsUsedAsPublicInput: false,
    });
    const authorityOut = join(options.out, "authority");
    mkdirSync(authorityOut);
    const authority = await authenticate("after", recorder(authorityOut));
    {
      const project = subsetFor("A59").projects[0]!,
        out = join(options.out, "issue-11359"),
        root = join(dirname(options.out), "q16-issue-11359");
      mkdirSync(out);
      const saved = recorder(out);
      try {
        let current = filesFor(project.states[0]!, profile);
        const installed = await install(
            root,
            current,
            options.npm,
            options.npx,
            authority,
            saved,
            cache,
          ),
          verify = () => {
            checkSource();
            installed.verify();
            inputImages(root, current);
          };
        final.push({ id: "issue-11359", verify });
        for (const [index, state] of project.states.entries()) {
          verify();
          const next = filesFor(state, profile);
          assert.deepEqual(
            next.map((file) => file.path),
            current.map((file) => file.path),
          );
          for (const file of next)
            if (file.path.startsWith("src/")) writeFileSync(join(root, file.path), file.source);
            else assert.equal(file.source, current.find((row) => row.path === file.path)!.source);
          current = next;
          const label = "state-" + index;
          saved.json(label + ".input.json", { id: state.id, files: inputImages(root, current) });
          const tsrs = capture(
              saved,
              label + ".tsrs",
              [process.execPath, options.npx, ...reportedCommands.tsrs.slice(1)],
              root,
              verify,
            ),
            reference = capture(
              saved,
              label + ".vue-tsc",
              [process.execPath, options.npx, ...reportedCommands.reference.slice(1)],
              root,
              verify,
            );
          a59.push({ state, tsrs, reference });
          saved.json(label + ".complete.json", {
            id: state.id,
            tsrs,
            reference,
            publicAcceptance: false,
          });
        }
        errors.push({ id: "issue-11359", failure: null });
      } catch (error) {
        const failure = safeFailure(error);
        errors.push({ id: "issue-11359", failure });
        saved.json("project-failure.json", failure);
      }
    }
    {
      const out = join(options.out, "issue-11365"),
        root = join(dirname(options.out), "q16-issue-11365");
      mkdirSync(out);
      const saved = recorder(out);
      try {
        let current: SourceFile[] = a65Inputs(a65Data.states[0]!, profile),
          last: A65Snapshot = { names: [], files: [], safe: true, present: false };
        const installed = await install(
            root,
            current,
            options.npm,
            options.npx,
            authority,
            saved,
            cache,
            ["vize"],
          ),
          verify = () => {
            checkSource();
            installed.verify();
            inputImages(root, current);
            verifyA65Outputs(root, last);
          };
        final.push({ id: "issue-11365", verify });
        for (const [index, state] of a65Data.states.entries()) {
          verify();
          resetA65Outputs(root, last);
          last = { names: [], files: [], safe: true, present: false };
          const next = a65Inputs(state, profile);
          assert.deepEqual(
            next.map((file) => file.path),
            current.map((file) => file.path),
          );
          for (const file of next) {
            if (file.path === "package.json")
              assert.equal(file.source, current.find((row) => row.path === file.path)!.source);
            else writeFileSync(join(root, file.path), file.source);
          }
          current = next;
          const label = "state-" + index;
          saved.json(label + ".input.json", {
            id: state.id,
            files: inputImages(root, current),
            outputAbsent: true,
            authoredJsonControl: state.json,
            originalCommand: !state.json,
          });
          const args = state.json ? a65Data.jsonArgs : a65Data.originalArgs,
            packet = capture(
              saved,
              label + ".vize",
              [process.execPath, options.npx, ...args.slice(1)],
              root,
              verify,
            );
          last = retainA65Outputs(root, saved, label);
          a65.push({ state, packet, outputs: last });
          saved.json(label + ".complete.json", {
            id: state.id,
            packet,
            outputs: last.files.map((file) => ({ name: file.name, image: file.image })),
            safeOutputPopulation: last.safe,
            publicAcceptance: false,
            fullDeclarationByteGoldenOwned: false,
          });
        }
        errors.push({ id: "issue-11365", failure: null });
      } catch (error) {
        const failure = safeFailure(error);
        errors.push({ id: "issue-11365", failure });
        saved.json("project-failure.json", failure);
      }
    }
    record.json("all-complete-observations.json", {
      a59: a59.map((row) => ({ id: row.state.id, tsrs: row.tsrs, reference: row.reference })),
      a65: a65.map((row) => ({
        id: row.state.id,
        packet: row.packet,
        outputs: row.outputs.files.map((file) => ({ name: file.name, image: file.image })),
      })),
      errors,
      publicAcceptance: false,
    });
    const custody = finalCustodiesBeforeAssessment(final, record);
    assert.equal(final.length, 2);
    assert(custody.every((row) => row.retainedRouteAndInputs));
    assert(errors.every((row) => row.failure === null));
    assert.equal(a59.length, 3);
    assert.equal(a65.length, 7);
    const outcomes: unknown[] = [];
    let all = true;
    for (const row of a59) {
      try {
        whole(row.reference);
        outcomes.push({ id: row.state.id, result: assessTsrs(row.state, row.tsrs), failure: null });
      } catch (error) {
        all = false;
        outcomes.push({ id: row.state.id, result: null, failure: safeFailure(error) });
      }
    }
    for (const row of a65) {
      try {
        outcomes.push({
          id: row.state.id,
          result: assessA65(row.state, row.packet, row.outputs),
          failure: null,
        });
      } catch (error) {
        all = false;
        outcomes.push({ id: row.state.id, result: null, failure: safeFailure(error) });
      }
    }
    try {
      for (const index of [1, 6]) requireSameA65Declarations(a65[0]!.outputs, a65[index]!.outputs);
    } catch (error) {
      all = false;
      outcomes.push({ id: "11365/identical-input-emission", failure: safeFailure(error) });
    }
    record.json("result.json", {
      status: "COMPLETE_A59_A65_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      accepted: false,
      publicAcceptance: false,
      issueQualified: false,
      fullDeclarationByteGoldenOwned: false,
      declarationRestorationByteEqualityAssessed: true,
      referenceSemanticOracleOwned: false,
      sourceDeclarationsUsedAsPublicInput: false,
      selectedCompilerPid: null,
      wholeNodeModulesFilesetQualified: false,
      limitations,
    });
    assert(
      all,
      "source-owned A intent differs; preserve complete raw and declarations without fitting",
    );
  } catch (error) {
    record.json("failure.json", {
      failure: safeFailure(error),
      accepted: false,
      publicAcceptance: false,
    });
    throw error;
  }
}
