import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  array,
  base,
  candidate,
  candidatePass,
  cargoMessages,
  exactUnits,
  inspectMetrics,
  object,
  order,
  postPassed,
  report,
  requiredTests,
  verifyGuardRoster,
  type Json,
  type Role,
} from "./laws.ts";
import {
  aliases,
  fileSha,
  git,
  here,
  inventoryEvidence,
  manifest,
  readJson,
  restoreFirst,
  retainFirst,
  run,
  sameSnapshot,
  saveAttempt,
  snapshot,
  success,
  writeJson,
} from "./custody.ts";

const diagnostic = "target/vize-tests/lsp-config-diagnostics";
const metric = "target/vize-tests/metrics/vben-lsp-incremental";
const wrapper = "tools/support/compat/github/run-lsp-config-diagnostics.ts";
const units = [
  "cargo",
  "test",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize_canon",
  "--features",
  "legacy",
  "--lib",
  "--message-format=json",
  "lsp_client::retained_text_tests::",
  "--",
  "--nocapture",
];

type State = {
  root: string;
  output: string;
  environment: NodeJS.ProcessEnv;
  first: Json | null;
  error: string | null;
};
const errorText = (error: unknown): string =>
  error instanceof Error ? (error.stack ?? error.message) : String(error);

function authenticate(controller: string): void {
  const h = git(controller, "rev-parse", "HEAD");
  assert.equal(h, process.env.RETAINED_CONTROLLER_SOURCE);
  assert.equal(h, process.env.GITHUB_SHA);
  assert.equal(git(controller, "show", "-s", "--format=%P", h), candidate);
  assert.equal(git(controller, "show", "-s", "--format=%P", candidate), base);
  const changed = git(controller, "diff", "--name-only", candidate, h).split("\n").sort();
  assert.deepEqual(changed, array(manifest.controllerPaths).map(String).sort());
  assert.ok(changed.every((name) => name.startsWith(".github/")));
  assert.deepEqual(
    git(controller, "diff", "--name-only", base, candidate).split("\n").sort(),
    array(manifest.changedSourcePaths).map(String).sort(),
  );
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  for (const name of aliases) assert.equal(process.env[name], undefined);
  assert.equal(process.env.CARGO_TARGET_DIR, undefined);
  assert.equal(process.env.RAYON_NUM_THREADS, "4");
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE, "1");
  assert.equal(process.env.VIZE_TRACE_EDITOR_PREPARATION, "1");
  assert.equal(process.env.CARGO_INCREMENTAL, "0");
}
function environment(role: Role): NodeJS.ProcessEnv {
  return {
    ...process.env,
    EXPECTED_SOURCE: role === "B" ? base : candidate,
    VIZE_LSP_BIN: "target/ci/vize",
  };
}
function prepare(state: State, role: Role, images: string): void {
  const { root, output, environment: env } = state;
  try {
    assert.equal(
      fs.existsSync(path.join(root, "target")),
      false,
      "role target must initially be absent",
    );
    for (const [label, command] of [
      [
        "fixture",
        [
          "git",
          "submodule",
          "update",
          "--init",
          "--depth",
          "1",
          "tests/_fixtures/_git/vue-vben-admin",
        ],
      ],
      ["install", ["vp", "install", "--frozen-lockfile", "--prefer-offline"]],
      ["cargo-version", ["cargo", "--version"]],
      ["compiler-version", ["rustc", "--version", "--verbose"]],
      ["vp-version", ["vp", "--version"]],
      [
        "build",
        [
          "cargo",
          "build",
          "--profile",
          "ci",
          "-p",
          "vize",
          "--features",
          "legacy",
          "--message-format=json",
        ],
      ],
      ["receipt", [process.execPath, "tests/differential/build-receipt.ts", "--legacy"]],
    ] as const)
      success(run(root, [...command], path.join(output, label), env));
    state.first = retainFirst(root, role, images, output);
    success(run(root, ["target/ci/vize", "--version"], path.join(output, "cli-version"), env));
    assert.equal(
      fs.readFileSync(path.join(output, "cli-version.stdout"), "utf8").trim(),
      object(state.first.receipt).cliVersion,
    );
    writeJson(path.join(output, "prepared.json"), snapshot(root, role));
  } catch (error) {
    state.error = errorText(error);
    writeJson(path.join(output, "producer-refusal.json"), {
      error: state.error,
      nativeOrMeasurementCredit: false,
    });
  }
}
function moveFinished(root: string, destination: string): void {
  fs.mkdirSync(destination, { recursive: true });
  for (const [relative, name] of [
    [diagnostic, "original"],
    [metric, "metrics"],
  ]) {
    const source = path.join(root, relative);
    if (!fs.existsSync(source)) continue;
    const target = path.join(destination, name);
    assert.equal(fs.existsSync(target), false);
    fs.renameSync(source, target);
  }
}
function attempt(state: State, role: Role, index: number, output: string, images: string): Json {
  const label = `${index + 1}-${role}`,
    directory = path.join(output, label);
  fs.mkdirSync(directory);
  const outcome: Json = {
    label,
    role,
    source: role === "B" ? base : candidate,
    comparable: false,
    originalPassed: false,
  };
  try {
    assert.ok(state.first, state.error ?? "first CLI is unavailable");
    for (const relative of [diagnostic, metric])
      assert.equal(
        fs.existsSync(path.join(state.root, relative)),
        false,
        `fresh original output required: ${relative}`,
      );
    restoreFirst(state.root, role, images, state.first);
    const before = snapshot(state.root, role);
    writeJson(path.join(directory, "before.json"), before);
    const result = run(
      state.root,
      [process.execPath, wrapper],
      path.join(directory, "wrapper"),
      state.environment,
    );
    // Preserve raw original output before ANY offline completeness assertion.
    saveAttempt(state.root, directory);
    outcome.commandResult = result;
    const after = snapshot(state.root, role);
    writeJson(path.join(directory, "after.json"), after);
    sameSnapshot(before, after);
    assert.equal(fileSha(path.join(state.root, "target/ci/vize")), state.first.sha256);
    const metrics = inspectMetrics(
      readJson(path.join(directory, "metrics/metrics.json")),
      String(outcome.source),
    );
    for (const raw of Object.values(object(metrics.processTreeSamples))) {
      const root = array(object(raw).members)
        .map(object)
        .find((row) => row.role === "root");
      assert.ok(root);
      assert.equal(object(root.identity).executableSha256, state.first.sha256);
    }
    if (role === "C") {
      const native = path.join(directory, "original/native-lsp.log");
      assert.ok(fs.statSync(native).size > 0, "original trace=1 native phases must be retained");
      assert.equal(fileSha(native), fileSha(path.join(directory, "original/process-lsp.log")));
    }
    const command = readJson(path.join(directory, "original/command.json"));
    assert.equal(object(command.buildReceipt).binarySha256, state.first.sha256);
    assert.equal(command.sourceRevision, outcome.source);
    assert.equal(object(command.nativeLogBefore).bytes, 0);
    const original = readJson(path.join(directory, "original/result.json"));
    assert.equal(original.exitStatus, result.status);
    assert.equal(original.signal, result.signal);
    assert.equal(original.processError, result.error);
    outcome.metrics = metrics;
    outcome.comparable = true;
    outcome.originalPassed = result.status === 0 && metrics.status === "passed";
    if (role === "C") candidatePass(result, metrics);
  } catch (error) {
    outcome.error = errorText(error);
    // Preserve available partial logs even when source/producer checks refuse.
    if (!fs.existsSync(path.join(directory, "original"))) saveAttempt(state.root, directory);
  } finally {
    moveFinished(state.root, path.join(images, "finished", label));
    writeJson(path.join(directory, "outcome.json"), outcome);
  }
  return outcome;
}
function postControls(state: State, role: Role, images: string): boolean {
  const results: Json[] = [];
  if (!state.first) {
    writeJson(path.join(state.output, "post-refusal.json"), { reason: state.error });
    return false;
  }
  const directory = path.join(state.root, diagnostic);
  fs.mkdirSync(directory, { recursive: true });
  const guards = verifyGuardRoster(
    readJson(path.join(here, "post-guards.json")),
    fs.readFileSync(path.join(state.root, ".github/workflows/lsp-config-diagnostics.yml"), "utf8"),
  );
  if (role === "C") guards.push({ label: "retained-units", argv: units, exactSeven: true });
  for (const guard of guards) {
    const label = String(guard.label),
      prefix = path.join(directory, label);
    const row: Json = { label, passed: false };
    try {
      restoreFirst(state.root, role, images, state.first);
      const before = snapshot(state.root, role);
      row.commandResult = run(state.root, array(guard.argv).map(String), prefix, state.environment);
      const stdout = fs.readFileSync(prefix + ".stdout", "utf8");
      // Original integration producer retention MUST precede first-CLI restoration.
      if (guard.retain)
        row.retainer = run(
          state.root,
          [process.execPath, wrapper, String(guard.retain)],
          prefix + ".retain",
          state.environment,
        );
      if (guard.requireRust) requiredTests(stdout);
      if (guard.exactSeven) {
        exactUnits(stdout);
        const artifacts = cargoMessages(stdout).filter(
          (message) =>
            message.reason === "compiler-artifact" &&
            object(message.target).name === "vize_canon" &&
            object(message.profile).test === true &&
            typeof message.executable === "string",
        );
        assert.equal(artifacts.length, 1);
        row.libraryContext = {
          cargoArtifact: artifacts[0],
          executableSha256: fileSha(String(artifacts[0].executable)),
          sameCliProtocolCredit: false,
        };
        writeJson(prefix + ".library-context.json", row.libraryContext);
      }
      success(object(row.commandResult));
      if (row.retainer) success(object(row.retainer));
      sameSnapshot(before, snapshot(state.root, role));
      row.passed = true;
    } catch (error) {
      row.error = errorText(error);
    } finally {
      restoreFirst(state.root, role, images, state.first);
      results.push(row);
      writeJson(prefix + ".verdict.json", row);
    }
  }
  // Copy only complete text receipts/logs. Wrapper-created test ELFs stay hosted.
  const destination = path.join(state.output, "post");
  fs.mkdirSync(destination);
  for (const name of fs.readdirSync(directory)) {
    if (!/\.(?:json|stdout|stderr|status)$/.test(name)) continue;
    const input = path.join(directory, name);
    assert.ok(fs.lstatSync(input).isFile());
    fs.copyFileSync(input, path.join(destination, name), fs.constants.COPYFILE_EXCL);
  }
  writeJson(path.join(state.output, "post-results.json"), results);
  return postPassed(
    results,
    guards.map((row) => String(row.label)),
  );
}
export function campaign(controller: string, b: string, c: string): void {
  const output = path.join(controller, "target/vize-tests/retained-text-campaign");
  fs.mkdirSync(output, { recursive: true });
  const outcomes: Json[] = [];
  let admitted = false;
  try {
    authenticate(controller);
    const images = path.join(
      String(process.env.RUNNER_TEMP),
      `retained-text-images-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`,
    );
    assert.ok(process.env.RUNNER_TEMP);
    assert.equal(fs.existsSync(images), false);
    fs.mkdirSync(images);
    writeJson(path.join(output, "controller.json"), {
      commit: git(controller, "rev-parse", "HEAD"),
      node: process.version,
      nodeExecutable: process.execPath,
      base,
      candidate,
      order,
      manifest,
      executableStorage: images,
    });
    const states = Object.fromEntries(
      (["B", "C"] as const).map((role) => {
        const state = {
          root: role === "B" ? b : c,
          output: path.join(output, role),
          environment: environment(role),
          first: null,
          error: null,
        };
        fs.mkdirSync(state.output);
        return [role, state];
      }),
    ) as Record<Role, State>;
    for (const role of ["B", "C"] as const) prepare(states[role], role, images);
    for (const [index, role] of order.entries())
      outcomes.push(attempt(states[role], role, index, output, images));
    const post = Object.fromEntries(
      (["B", "C"] as const).map((role) => [role, postControls(states[role], role, images)]),
    );
    writeJson(path.join(output, "post-admission.json"), post);
    assert.equal(outcomes.length, 4);
    assert.ok(
      outcomes.every((row) => row.comparable === true),
      "all four source-bound complete outcomes required",
    );
    assert.ok(
      outcomes
        .filter((row) => row.role === "C")
        .every((row) => row.originalPassed === true && row.error === undefined),
      "BOTH candidate attempts must pass EVERY original guard",
    );
    assert.deepEqual(post, { B: true, C: true });
    const corpora = outcomes.map((row) => object(object(row.metrics).corpus));
    for (const entry of corpora) assert.deepEqual(entry, corpora[0]);
    const providers = ["B", "C"].map((role) =>
      object(readJson(path.join(output, role, "prepared.json")).provider),
    );
    assert.equal(providers[0].sha256, providers[1].sha256);
    admitted = true;
  } catch (error) {
    writeJson(path.join(output, "campaign-refusal.json"), { error: errorText(error) });
  } finally {
    writeJson(path.join(output, "campaign.json"), {
      admitted,
      order,
      outcomes,
      interpretation:
        "Original instrumented legacy-ci fourteen-lane observations only. Base failures remain failures; no general/shipping latency, heap/allocation attribution or memory-win claim.",
      noRetriesOrFavorableSelection: true,
    });
    fs.writeFileSync(path.join(output, "summary.md"), report(outcomes, admitted), { flag: "wx" });
    writeJson(path.join(output, "members.json"), inventoryEvidence(output));
    process.exitCode = admitted ? 0 : 1;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [controller, b, c] = process.argv.slice(2);
  assert.ok(controller && b && c);
  campaign(controller, b, c);
}
