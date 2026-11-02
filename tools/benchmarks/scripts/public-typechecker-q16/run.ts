import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { assertCustody } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import {
  filesFor,
  limitations,
  profileFor,
  commandsFor,
  sha256,
  requireEnvironment,
  subsetFor,
  type SourceFile,
} from "./contract.ts";
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
import { runDeclaration } from "./declaration.ts";
import { runA } from "./a-capture.ts";

export function argumentsFor(argv: string[]) {
  assert.equal(argv.length, 6);
  const values = new Map<string, string>();
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i]!,
      value = argv[i + 1]!;
    assert(["--out", "--npm", "--npx"].includes(key) && !values.has(key) && isAbsolute(value));
    values.set(key, value);
  }
  return { out: values.get("--out")!, npm: values.get("--npm")!, npx: values.get("--npx")! };
}
export function requireOwnedDispatch(environment: NodeJS.ProcessEnv) {
  assert(
    environment.GITHUB_EVENT_NAME === "workflow_dispatch" &&
      environment.GITHUB_REPOSITORY === "ubugeeei-prod/vize",
    "owned manual event required",
  );
  assert(
    [
      "refs/heads/main",
      "refs/heads/test/published-q16-first-subset-20261102",
      "refs/heads/test/published-commented-vif-11378-20261102",
      "refs/heads/test/published-emits-11390-11391-20261102",
      "refs/heads/test/published-static-template-ref-11377-20261102",
    ].includes(environment.GITHUB_REF ?? ""),
    "owned source ref required",
  );
}
export function writeEmitsInputs(root: string, current: SourceFile[], next: SourceFile[]) {
  inputImages(root, current);
  const roster = (files: SourceFile[]) => files.map((file) => file.path).join("|");
  const allowed = [
    "package.json|tsconfig.json|src/Probe.vue",
    "package.json|tsconfig.json|src/Probe.vue|src/Listener.ts",
  ];
  assert(
    allowed.includes(roster(current)) && allowed.includes(roster(next)),
    "closed emit input population required",
  );
  assert(next[0]!.source === current[0]!.source, "registered package must stay unchanged");
  if (current.length === 4 && next.length === 3) unlinkSync(join(root, "src/Listener.ts"));
  for (const file of next)
    if (file.path !== "package.json") writeFileSync(join(root, file.path), file.source);
  inputImages(root, next);
}
export async function main(argv: string[]) {
  const options = argumentsFor(argv),
    profile = profileFor();
  // UNKNOWN release/fix/subset authority refuses before output allocation, network or install.
  requireOwnedDispatch(process.env);
  if (profile.subset === "A64") return runDeclaration(options, profile);
  if (profile.subset === "A59-65") return runA(options, profile);
  const selected = subsetFor(profile.subset),
    commands = commandsFor(profile.subset),
    emits = profile.subset === "E90-91";
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    cache = new Map<string, Buffer>();
  const own = sourceCustody();
  const checkSource = () => assertCustody(own);
  const captured: {
    id: string;
    state: (typeof selected.projects)[number]["states"][number];
    tsrs: Packet;
    reference: Packet | null;
  }[] = [];
  const final: { id: string; verify: () => void }[] = [];
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
      subset: selected.id,
      sourceAuthority: selected.sourceAuthority,
      own,
      environment: requireEnvironment(process.env),
      reportedCommands: commands,
      limitations,
    });
    const authorityOut = join(options.out, "authority");
    mkdirSync(authorityOut);
    const authority = await authenticate("after", recorder(authorityOut));
    const projectErrors: unknown[] = [];
    for (const project of selected.projects) {
      const out = join(options.out, project.id);
      mkdirSync(out);
      const owned = recorder(out),
        root = join(dirname(options.out), "q16-" + project.id);
      try {
        const installed = await install(
          root,
          filesFor(project.states[0]!, profile),
          options.npm,
          options.npx,
          authority,
          owned,
          cache,
          commands.reference === null ? ["tsrs"] : undefined,
        );
        let finalFiles = filesFor(project.states[0]!, profile);
        const verify = () => {
          checkSource();
          installed.verify();
          inputImages(root, finalFiles);
        };
        final.push({ id: project.id, verify });
        for (let i = 0; i < project.states.length; i++) {
          const state = project.states[i]!,
            next = filesFor(state, profile);
          verify();
          if (emits) writeEmitsInputs(root, finalFiles, next);
          else {
            assert.deepEqual(
              next.map((file) => file.path),
              finalFiles.map((file) => file.path),
            );
            for (const file of next) {
              if (file.path.startsWith("src/")) writeFileSync(join(root, file.path), file.source);
              else
                assert.equal(
                  file.source,
                  finalFiles.find((previous) => previous.path === file.path)!.source,
                );
            }
          }
          finalFiles = next;
          const before = inputImages(root, next),
            label = "state-" + i;
          owned.json(label + ".input.json", {
            id: state.id,
            phase: state.phase,
            files: before,
            ...(emits
              ? {
                  sourceInputSha256: sha256(JSON.stringify(state.files)),
                  actualInputSha256: sha256(JSON.stringify(next)),
                }
              : {}),
          });
          const tsrs = capture(
            owned,
            label + ".tsrs",
            [process.execPath, options.npx, ...commands.tsrs.slice(1)],
            root,
            verify,
          );
          const reference = commands.reference
            ? capture(
                owned,
                label + ".vue-tsc",
                [process.execPath, options.npx, ...commands.reference.slice(1)],
                root,
                verify,
              )
            : null;
          captured.push({ id: project.id, state, tsrs, reference });
          // Record the complete pair before input recheck; no semantic verdict here.
          owned.json(label + ".complete.json", {
            state: state.id,
            tsrs,
            reference,
            publicAcceptance: false,
          });
          inputImages(root, next);
        }
        projectErrors.push({ id: project.id, failure: null });
      } catch (error) {
        projectErrors.push({ id: project.id, failure: safeFailure(error) });
        owned.json("project-failure.json", safeFailure(error));
      }
    }
    record.json("all-complete-observations.json", {
      captured,
      projectErrors,
      publicAcceptance: false,
    });
    const custody = finalCustodiesBeforeAssessment(final, record);
    assert.equal(
      final.length,
      selected.projects.length,
      "all selected projects require final custody",
    );
    assert(
      custody.every((row) => row.retainedRouteAndInputs),
      "final custody refusal; all raw retained before aggregate",
    );
    assert(
      projectErrors.every((value) => (value as { failure: unknown }).failure === null),
      "one selected project incomplete",
    );
    assert.equal(
      captured.length,
      selected.projects.reduce((sum, project) => sum + project.states.length, 0),
    );
    // Reference semantics remain unowned, but its actual whole process must be retained.
    for (const row of captured) {
      if (commands.reference) {
        assert(row.reference);
        whole(row.reference);
      } else assert.equal(row.reference, null);
    }
    const outcomes = captured.map((row) => {
      try {
        return { id: row.state.id, result: assessTsrs(row.state, row.tsrs), failure: null };
      } catch (error) {
        return { id: row.state.id, result: null, failure: safeFailure(error) };
      }
    });
    record.json("result.json", {
      status: "COMPLETE_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      referenceOracleOwned: false,
      referencePacketsRetained: captured.filter((row) => row.reference !== null).length,
      limitations,
      accepted: false,
      publicAcceptance: false,
      issueQualified: false,
      productQualified: false,
      selectedCompilerPid: null,
    });
    assert(
      outcomes.every((row) => row.failure === null),
      "complete source-owned tsrs vectors differ; failed raw remains authoritative",
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
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv.slice(2)).catch(() => {
    writeFileSync(2, "Q16 public capture refused; retained safe evidence remains unqualified\n");
    process.exitCode = 1;
  });
