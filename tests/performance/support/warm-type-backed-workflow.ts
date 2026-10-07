import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { finiteCut } from "./warm-type-backed-cut.ts";

import {
  driverRoot,
  git,
  inputAuthority,
  sha256,
  sourceIdentity,
} from "./warm-type-backed-source.ts";

const output = path.join(process.env.RUNNER_TEMP!, "warm-pair");
const before = path.join(process.env.RUNNER_TEMP!, "warm-before");
assert.equal(process.platform, "linux");
assert.ok(process.env.RUNNER_TEMP && process.env.GITHUB_WORKSPACE);
assert.equal(fs.realpathSync(process.env.GITHUB_WORKSPACE), driverRoot);

if (process.argv[2] === "prepare") {
  const driver = sourceIdentity(driverRoot);
  assert.ok(
    !fs.existsSync(before) && !fs.existsSync(output),
    "fresh owned directories are required",
  );
  fs.mkdirSync(output);
  fs.writeFileSync(
    path.join(output, "prepare-inputs.json"),
    `${JSON.stringify({ driver, inputs: Object.fromEntries(["ORIGINAL_CONTROL_SHA", "ORIGINAL_AFTER_SHA", "ORIGINAL_AFTER_TREE", "ORIGINAL_MANIFEST_SHA256"].map((key) => [key, process.env[key] ?? ""])) }, null, 2)}\n`,
  );
  let cut: ReturnType<typeof finiteCut>;
  try {
    cut = finiteCut(output);
  } catch (error) {
    fs.writeFileSync(
      path.join(output, "prepare-failure.json"),
      `${JSON.stringify({ error: error instanceof Error ? error.stack : String(error) }, null, 2)}\n`,
    );
    throw error;
  }
  assert.equal(driver.dirty, "");
  if (cut) {
    assert.equal(driver.revision, process.env.GITHUB_SHA, "the dispatch driver must be literal");
    assert.equal(
      driver.revision,
      process.env.GITHUB_WORKFLOW_SHA,
      "the loaded workflow must match",
    );
  }
  const head = cut?.after ?? process.env.HEAD_SHA!;
  const base = cut?.control ?? process.env.BASE_SHA!;
  for (const sha of [head, base]) assert.match(sha, /^[a-f0-9]{40}$/u);
  if (!cut) assert.equal(driver.revision, head);
  const baseline = cut?.control ?? git(driverRoot, ["merge-base", base, head]);
  const afterRoot = cut ? path.join(process.env.RUNNER_TEMP!, "warm-after") : driverRoot;
  const production = git(driverRoot, [
    "diff",
    "--name-only",
    baseline,
    head,
    "--",
    "crates",
    "davinci",
    "vendor",
    "npm",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    ".config",
    "rust-toolchain.toml",
  ])
    .split("\n")
    .filter(Boolean);
  const allowed = new Set([
    // Art lint delivery retains the original 400-provider protocol corpus.
    "crates/vize_maestro/src/ide/diagnostics/art_lint_tests.rs",
    "crates/vize_maestro/src/ide/diagnostics/art_lint.rs",
    "crates/vize_maestro/src/ide/diagnostics/service.rs",
    "crates/vize_maestro/src/ide/rename/corsa_session_tests/harness/protocol/readiness.rs",
    "crates/vize_canon/src/corsa_bridge.rs",
    "crates/vize_canon/src/corsa_bridge/preparation_trace.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/prepare.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    "crates/vize_canon/src/lsp_client/diagnostics_lsp.rs",
    "crates/vize_canon/src/lsp_client/diagnostics_lsp/readiness_workers.rs",
    "crates/vize_canon/src/lsp_client/diagnostics_lsp/readiness_workers/tests.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/readiness.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/synchronize.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/build.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/catalog.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/fingerprint.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document.rs",
    "crates/vize_canon/src/corsa_bridge/script_document.rs",
    "crates/vize_canon/src/corsa_bridge/bridge/documents.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/build.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/types.rs",
    "crates/vize_canon/src/batch/import_rewriter.rs",
    "crates/vize_canon/src/batch/import_rewriter_dependency_tests.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependency_specifiers.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependency_specifiers/original.rs",
    "crates/vize_maestro/src/ide/corsa_support/canonical/open.rs",
    "crates/vize_maestro/src/ide/diagnostics.rs",
    "crates/vize_maestro/src/ide/diagnostics/assembly_parity_tests.rs",
    "crates/vize_maestro/src/ide/diagnostics/assembly_parity_custody.rs",
    "crates/vize_maestro/src/ide/diagnostics/corsa/collect_virtual.rs",
    "crates/vize_maestro/src/ide/diagnostics/native.rs",
    "crates/vize_maestro/src/ide/diagnostics/native/timeout.rs",
    "crates/vize_maestro/src/ide/diagnostics/native/timeout/tests.rs",
    "crates/vize_maestro/src/ide/diagnostics/corsa/collect.rs",
    "crates/vize_maestro/src/ide/diagnostics/editor_reference_options_tests.rs",
    "crates/vize_maestro/src/server/state/global_components.rs",
    "crates/vize_maestro/src/server/state/global_components/editor_options.rs",
    "crates/vize_maestro/src/server/state/global_components/editor_options/custody.rs",
    "crates/vize_maestro/src/server/state/global_components/editor_options/tests.rs",
    // Explicit global-component configuration retains the full original provider gate.
    "crates/vize/src/commands/lint/entry_rules.rs",
    "crates/vize/tests/lint_global_components_cli.rs",
    "crates/vize_maestro/src/ide/diagnostics/configured_global_components_tests.rs",
    "crates/vize_maestro/src/ide/diagnostics/linter_options.rs",
    "crates/vize_patina/src/linter/restricted_rules.rs",
    "crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs",
    "crates/vize_patina/tests/fixtures/global-component-registration/Card.art.vue",
    "crates/vize_patina/tests/fixtures/global-component-registration/vize.config.json",
    "crates/vize_patina/tests/global_component_registration.rs",
    "davinci/vize_l0/src/config/model/linter_rule_options.rs",
    "davinci/vize_l0/src/config/model/linter_rule_options/component_registration.rs",
    "npm/cli/pkl/LinterConfig.pkl",
    "npm/cli/pkl/jsonschema/LintRuleOptionsSchemaDefinitions.pkl",
    "npm/cli/pkl/vize.pkl",
    "npm/cli/schemas/vize.config.schema.json",
    "npm/cli/src/types/generated.ts",
  ]);
  const harnessOnly = !cut && production.length === 0;
  if (harnessOnly) {
    const changed = git(driverRoot, ["diff", "--name-only", baseline, head])
      .split("\n")
      .filter(Boolean);
    assert.ok(changed.length > 0);
    const harness = new Set([
      ".github/workflows/davinci-canon-scaling.yml",
      "tests/tooling/support/lsp/session.ts",
      "tests/tooling/support/lsp/session-process.ts",
      "tests/tooling/support/lsp/session-capture.ts",
      "tests/tooling/support/lsp/published-launch.ts",
      "tests/performance/support/warm-type-backed-release-bridge.py",
      "tests/performance/support/warm-type-backed-release-bridge.json",
      "docs/davinci/decisions/2026-09-27-level-restructure.md",
      "docs/davinci/decisions/2026-10-05-lsp-warm-query-surfaces.md",
    ]);
    for (const file of changed)
      assert.ok(
        harness.has(file) ||
          /^tests\/performance\/support\/warm-type-backed-[a-z-]+\.ts$/u.test(file),
        `unqualified harness delta: ${file}`,
      );
    for (const [file, digest] of Object.entries(inputAuthority())) {
      const original = spawnSync(
        "git",
        ["show", `${baseline}:tests/_fixtures/differential/lsp/warm-type-backed-requests/${file}`],
        { cwd: driverRoot },
      );
      assert.equal(original.status, 0, original.stderr.toString());
      assert.equal(
        sha256(original.stdout),
        digest,
        "harness qualification preserves every original input",
      );
    }
  } else assert.ok(production.length > 0);
  if (!cut) {
    for (const file of production)
      assert.ok(allowed.has(file), `unqualified production delta: ${file}`);
  }
  const result = spawnSync("git", ["worktree", "add", "--detach", before, baseline], {
    cwd: driverRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  if (cut) {
    assert.ok(!fs.existsSync(afterRoot), "the literal cut needs its own fresh checkout");
    const after = spawnSync("git", ["worktree", "add", "--detach", afterRoot, head], {
      cwd: driverRoot,
      encoding: "utf8",
    });
    assert.equal(after.status, 0, after.stderr);
  }
  for (const [side, root] of [
    ["before", before],
    ["after", afterRoot],
    ["driver", driverRoot],
  ]) {
    const directory = path.join(output, "source-locks", side);
    fs.mkdirSync(directory, { recursive: true });
    for (const file of [
      "Cargo.toml",
      "Cargo.lock",
      "package.json",
      "pnpm-lock.yaml",
      "rust-toolchain.toml",
    ])
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
  }
  fs.writeFileSync(
    path.join(output, "workflow-source.json"),
    JSON.stringify(
      {
        head,
        base,
        baseline,
        production,
        afterRoot,
        driverSource: driver,
        authority: cut
          ? "root-frozen-release-cut"
          : harnessOnly
            ? "harness-only-qualification"
            : "owned-source-pr",
        finiteCut: cut,
        originals: inputAuthority(),
        runner: {
          run: process.env.GITHUB_RUN_ID,
          attempt: process.env.GITHUB_RUN_ATTEMPT,
          workflow: process.env.GITHUB_WORKFLOW_SHA,
          node: process.version,
          platform: process.platform,
          arch: process.arch,
        },
        driver: Object.fromEntries(
          fs
            .readdirSync(path.dirname(new URL(import.meta.url).pathname))
            .filter((name) => name.startsWith("warm-type-backed-"))
            .toSorted()
            .map((name) => [name, sha256(fs.readFileSync(new URL(name, import.meta.url)))]),
        ),
        protocol: Object.fromEntries(
          [
            "session.ts",
            "session-process.ts",
            "launch.ts",
            "session-capture.ts",
            "published-launch.ts",
          ].map((name) => [
            name,
            sha256(fs.readFileSync(path.join(driverRoot, "tests/tooling/support/lsp", name))),
          ]),
        ),
        scope: cut
          ? "Literal published v0.433 and root-frozen release cut; complete changed Git-entry manifest, independent driver, identical fresh ci builds, original400 and one recorded runtime"
          : harnessOnly
            ? "Same production source; closed reviewed harness-only delta and unchanged original inputs/locks. Qualification only, no performance or source-effect gain."
            : "Actual common ancestor and current source, only owned prepared-surface, readiness-ack, shared editor reference-options or owned document-batch transfer delta; one worker, identical release recipe, original400 inputs and current locked runtime",
      },
      null,
      2,
    ) + "\n",
  );
  if (!cut) assert.deepEqual(sourceIdentity(before).locks, sourceIdentity(afterRoot).locks);
  else fs.writeFileSync(path.join(output, "changed-tree-manifest.json"), cut.manifestBytes);
} else {
  throw new Error("usage: warm-type-backed-workflow.ts prepare");
}
