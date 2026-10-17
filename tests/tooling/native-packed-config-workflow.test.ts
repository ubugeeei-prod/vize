import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync, execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import {
  withoutPackedConfigExtension,
  verifyHistoricalProducerFiles,
  readPackedProducerRetirement,
  verifyPackedProducerPreflight,
} from "./support/native-packed-config-extension.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, ".github/workflows/native-smoke.yml"), "utf8");
const workflow = parse(source);
const steps = workflow.jobs["packed-config-acceptance"].steps;

void test("the actual manager version audit uses a manifest-free cwd and preserves failed probes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-manager-probe-law-"));
  try {
    const run = steps.find(
      (step: { name?: string }) =>
        step.name === "Verify packed configuration driver laws and command versions",
    ).run as string;
    const match = /<<'JS' >>[^\n]+\n([\s\S]*?)\nJS/u.exec(run);
    assert.ok(match);
    const script = match[1];
    const env = { ...process.env };
    for (const manager of ["npm", "pnpm", "yarn", "bun", "vp"]) {
      // A process control for context isolation, never installed-manager proof.
      writeFakeCommand(
        directory,
        manager,
        `
        const fs = require('node:fs'), path = require('node:path');
        let parent = process.cwd();
        for (;;) {
          const manifest = path.join(parent, 'package.json');
          if (fs.existsSync(manifest) && JSON.parse(fs.readFileSync(manifest)).packageManager) {
            process.stderr.write('project context inherited: ' + manifest + '\\n');
            process.exit(23);
          }
          const next = path.dirname(parent);
          if (next === parent) break;
          parent = next;
        }
        process.stdout.write('fixture-version\\n');
      `,
      );
      env[`${manager.toUpperCase()}_BIN`] = path.join(directory, manager);
    }
    const execute = (body: string) =>
      spawnSync(process.execPath, ["--input-type=module", "-e", body], {
        cwd: root,
        env,
        encoding: "utf8",
        timeout: 10_000,
      });
    const success = execute(script);
    assert.equal(success.status, 0, success.stderr);
    assert.equal(success.signal, null);
    const records = success.stdout
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.deepEqual(
      records.map((record) => record.manager),
      ["npm", "pnpm", "yarn", "bun", "vp"],
    );
    for (const record of records) {
      assert.notEqual(record.cwd, root);
      assert.equal(fs.existsSync(record.cwd), false, "temporary probe cwd is removed");
      assert.deepEqual(record.args, ["--version"]);
      assert.equal(record.status, 0);
      assert.equal(record.stdout, "fixture-version\n");
      assert.equal(record.stderr, "");
    }
    const inherited = script.replace("cwd: managerProbeRoot, ", "");
    assert.notEqual(inherited, script);
    const failure = execute(inherited);
    assert.notEqual(failure.status, 0);
    const failed = JSON.parse(failure.stdout.trim());
    assert.equal(failed.status, 23);
    assert.equal(failed.stderr, `project context inherited: ${path.join(root, "package.json")}\n`);
    assert.match(failure.stderr, /AssertionError/u);
    assert.equal(fs.existsSync(failed.cwd), false, "failed probes also remove their temporary cwd");
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("the historical composed producer authenticates every whole source and moved test module", () => {
  const producer = "354c1b4a054c0a727d839d08dc134e72a21819c4";
  const witnessPath =
    "tests/_fixtures/differential/vite-native-image-identity/packed-config-extension.json";
  const witness = JSON.parse(fs.readFileSync(path.join(root, witnessPath), "utf8"));
  assert.equal(workflow.jobs["packed-config-acceptance"].env.PACKED_CONFIG_PRODUCER, producer);
  assert.equal(witness.producer.head, producer);
  assert.equal(witness.producer.composedBase, "178bbe12ea5cb939c2b43f4cc0656e45952c6d3b");
  assert.equal(witness.producer.priorProducer, "820bf6b5d688c77f8edea6ba18ff6f86f76bffae");
  assert.equal(witness.producer.previousFrozenProducer, "c8855589af88f3804686959377d29e85e3d7a4f4");
  assert.equal(witness.producer.earlierFrozenProducer, "9654d136f504765f0dfbdb1e17cf806e0b967878");
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root });
  const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
  const files = Object.keys(witness.producer.files);
  assert.equal(files.length, 348);
  assert.equal(new Set(files).size, files.length);
  const changedRust = git(
    "diff",
    "--name-only",
    "--no-renames",
    witness.producer.composedBase,
    producer,
  )
    .toString()
    .trim()
    .split("\n")
    .filter((file) => file.endsWith(".rs"));
  assert.equal(changedRust.length, 353);
  const roleBytes = fs.readFileSync(path.join(root, witness.producer.sourceOnlyRust.path));
  assert.equal(digest(roleBytes), witness.producer.sourceOnlyRust.sha256);
  const roles = JSON.parse(roleBytes.toString());
  assert.equal(roles.producer, producer);
  assert.equal(roles.priorSource, "e0bc8fb50bdc28915167a3fbdceea86f00b65030");
  assert.equal(roles.main, "a09d0e96214610bee58a5b31429eae51d07fb0df");
  const sourceOnly = Object.keys(roles.files);
  assert.deepEqual(sourceOnly, [
    "tools/commands/ci/github/release-preflight.rs",
    "tools/commands/ci/source-file-lengths.rs",
    "tools/commands/fixtures/lint-divergence-report.rs",
    "tools/commands/release/pr.rs",
    "tools/support/lint_divergence_surface.rs",
    "tools/support/release/pr_ci.rs",
    "tools/support/release/pr_pin.rs",
    "tools/support/release/pr_pin_metadata.rs",
    "tools/support/release/pr_pin_retire.rs",
    "tools/support/release/pr_pin_retire_archive.rs",
    "tools/support/release/pr_pin_retire_evidence.rs",
    "tools/support/release/pr_pin_retire_guard.rs",
    "tools/support/release/pr_pin_retire_ledger.rs",
    "tools/support/release/pr_pin_retire_lifecycle_tests.rs",
    "tools/support/release/pr_pin_retire_prequalification_archive_tests.rs",
    "tools/support/release/pr_pin_retire_source.rs",
    "tools/support/release/pr_pin_runs.rs",
    "tools/support/release/pr_pin_target_tests.rs",
    "tools/support/release/pr_pin_tests.rs",
    "tools/support/release/pr_watch.rs",
    "tools/support/release/preflight_pinned_evidence_tests.rs",
  ]);
  for (const row of Object.values(roles.files) as { compilerBound: boolean }[])
    assert.equal(row.compilerBound, false);
  const historicalAbsences = witness.producer.historicalRenames.map(
    (rename: { from: string }) => rename.from,
  );
  assert.deepEqual(
    [
      ...files.filter(
        (file) =>
          file.endsWith(".rs") && !historicalAbsences.includes(file) && !sourceOnly.includes(file),
      ),
      ...sourceOnly,
    ].sort(),
    changedRust.sort(),
    "complete Rust composition, including additions and deletion, is authenticated",
  );
  assert.deepEqual(
    files.filter((file) => !file.endsWith(".rs")),
    [
      "Cargo.lock",
      "Cargo.toml",
      "crates/vize_carton/src/config/loader/module-runtime.d.mts",
      "crates/vize_carton/src/config/loader/module-runtime.mjs",
    ],
  );
  assert.deepEqual(witness.producer.cargoInputs, ["Cargo.lock", "Cargo.toml"]);
  git("merge-base", "--is-ancestor", producer, "HEAD");
  assert.deepEqual(witness.producer.absentFiles, [
    "crates/vize/src/commands/check/runner/input_scope/tests.rs",
    "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/retained_assignments.rs",
    "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/retained_scopes.rs",
  ]);
  const [deletion] = witness.producer.netRustDeletions;
  assert.equal(witness.producer.netRustDeletions.length, 1);
  assert.equal(deletion.from, "crates/vize/src/commands/check/runner/input_scope/tests.rs");
  assert.equal(deletion.to, "crates/vize/src/commands/check/runner/input_scope_tests.rs");
  assert.equal(deletion.fromSource, witness.producer.composedBase);
  assert.equal(deletion.toSource, "a9dec43c406c7afd1d2ce8067fc563678e28ac64");
  assert.equal(digest(git("show", deletion.fromSource + ":" + deletion.from)), deletion.fromSha256);
  assert.equal(digest(git("show", deletion.toSource + ":" + deletion.to)), deletion.toSha256);
  const removedRust = git(
    "diff",
    "--name-only",
    "--no-renames",
    "--diff-filter=D",
    witness.producer.composedBase,
    producer,
  )
    .toString()
    .trim()
    .split("\n")
    .filter((file) => file.endsWith(".rs"));
  const retirement = readPackedProducerRetirement(root, witness.producer);
  assert.equal(retirement.source, "7136efec7025c009299b583b36c40b2dc9b7424e");
  assert.equal(retirement.retiredAt, "c022cf601c6e49b14dbcc57093948c56f132cc55");
  assert.equal(Object.keys(retirement.files).length, 98);
  assert.deepEqual(
    removedRust.sort((left, right) => left.localeCompare(right)),
    [
      deletion.from,
      ...Object.keys(retirement.files),
      ...sourceOnly.filter((file) => roles.files[file].current === null),
    ].sort((left: string, right: string) => left.localeCompare(right)),
  );
  assert.equal(Object.values(witness.producer.files).filter((value) => value === null).length, 101);
  const qualification = witness.producer.previousQualification;
  assert.equal(qualification.source, retirement.source);
  assert.equal(qualification.producer, "c45e0901fc05cfa51c50d167ac97a34ec466dd18");
  const previousQualification = git("show", qualification.source + ":" + witnessPath);
  assert.equal(digest(previousQualification), qualification.witnessSha256);
  assert.equal(
    digest(git("show", qualification.source + ":.github/workflows/native-smoke.yml")),
    qualification.workflowSha256,
  );
  const previousQualificationWitness = JSON.parse(previousQualification.toString());
  assert.equal(Object.keys(previousQualificationWitness.producer.files).length, 212);
  for (const file of Object.keys(previousQualificationWitness.producer.files))
    assert.ok(files.includes(file), file);
  const diagnostic = witness.producer.previousDiagnostic;
  assert.equal(diagnostic.source, "eb01ef3ace9ed68513b7cf65ca426d73ea139f46");
  assert.equal(diagnostic.producer, "47284d8e618b660658bdd28445306e18cfddca90");
  const previousDiagnosticBytes = git("show", diagnostic.source + ":" + witnessPath);
  assert.equal(digest(previousDiagnosticBytes), diagnostic.witnessSha256);
  assert.equal(
    digest(git("show", diagnostic.source + ":.github/workflows/native-smoke.yml")),
    diagnostic.workflowSha256,
  );
  const previousDiagnostic = JSON.parse(previousDiagnosticBytes.toString());
  assert.equal(Object.keys(previousDiagnostic.producer.files).length, 343);
  assert.deepEqual(
    files.filter(
      (file) =>
        ![
          "crates/vize/tests/check_flow_depth_isolation_cli.rs",
          "crates/vize_canon/src/virtual_ts/scope/vif_guard.rs",
          "crates/vize_canon/src/virtual_ts/scope/vif_guard_chain.rs",
          "crates/vize_patina/src/linter/tests/no_mutating_props.rs",
          "crates/vize_patina/src/rules/vue/v_slot_style/tests.rs",
        ].includes(file),
    ),
    Object.keys(previousDiagnostic.producer.files),
  );
  const changedDiagnosticInputs = files.filter(
    (file) => previousDiagnostic.producer.files[file] !== witness.producer.files[file],
  );
  assert.deepEqual(changedDiagnosticInputs, [
    "crates/vize/tests/check_flow_depth_isolation_cli.rs",
    "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_inputs.rs",
    "crates/vize_canon/src/corsa_bridge/preparation_trace.rs",
    "crates/vize_canon/src/package_route/stamp.rs",
    "crates/vize_canon/src/virtual_ts/scope/vif_guard.rs",
    "crates/vize_canon/src/virtual_ts/scope/vif_guard_chain.rs",
    "crates/vize_maestro/src/server/state/project_contexts/freshness.rs",
    "crates/vize_maestro/src/server/state/project_contexts/freshness_tests.rs",
    "crates/vize_patina/src/linter/tests/no_mutating_props.rs",
    "crates/vize_patina/src/rules/vue/v_slot_style/tests.rs",
    "tools/commands/release/pr.rs",
    "tools/support/release/pr_pin.rs",
  ]);
  const latest = roles.latestDiagnostic;
  assert.equal(latest.source, "3bfb3885f3328133968731873d2414773d00ee4a");
  assert.equal(latest.producer, roles.priorSource);
  const latestBytes = git("show", latest.source + ":" + witnessPath);
  assert.equal(digest(latestBytes), latest.witnessSha256);
  assert.equal(
    digest(git("show", latest.source + ":.github/workflows/native-smoke.yml")),
    latest.workflowSha256,
  );
  const latestWitness = JSON.parse(latestBytes.toString());
  assert.equal(Object.keys(latestWitness.producer.files).length, 343);
  for (const file of Object.keys(latestWitness.producer.files))
    assert.ok(files.includes(file), file);
  for (const field of [
    "previousFreeze",
    "priorAdmission",
    "retiredRust",
    "previousQualification",
    "previousDiagnostic",
  ])
    assert.deepEqual(witness.producer[field], latestWitness.producer[field]);
  verifyHistoricalProducerFiles(root, witness.producer);
  const original = witness.producer.previousFreeze;
  assert.equal(original.source, "f7ee25f5db27bd210db07f514837b9f4c1b65054");
  assert.equal(original.head, "5089dd1ad6dae7fbf6fc9efb33f202f8824f6f1e");
  const previous = JSON.parse(git("show", original.source + ":" + witnessPath).toString());
  assert.deepEqual(original.files, previous.producer.files);
  assert.equal(Object.keys(original.files).length, 26);
  for (const file of Object.keys(original.files)) assert.ok(files.includes(file), file);
  assert.equal(
    digest(git("show", original.source + ":.github/workflows/native-smoke.yml")),
    original.workflowSha256,
  );
  const prior = witness.producer.priorAdmission;
  assert.equal(prior.source, "35d5b71d364a59accfa7f64dcfc23fa3667be417");
  assert.equal(prior.producer, "a9dec43c406c7afd1d2ce8067fc563678e28ac64");
  assert.equal(prior.witnessPath, witnessPath);
  const priorBytes = git("show", prior.source + ":" + witnessPath);
  assert.equal(digest(priorBytes), prior.witnessSha256);
  const priorWitness = JSON.parse(priorBytes.toString());
  assert.equal(
    digest(git("show", prior.source + ":.github/workflows/native-smoke.yml")),
    prior.workflowSha256,
  );
  assert.equal(Object.keys(priorWitness.producer.files).length, 197);
  for (const file of Object.keys(priorWitness.producer.files))
    assert.ok(files.includes(file), file);
  for (const field of ["previousFreeze", "absentFiles", "historicalRenames", "netRustDeletions"])
    assert.deepEqual(witness.producer[field], priorWitness.producer[field], field);
  const historicalBase = "85b7c8d4b12226b67eaf23dd7b84fb14b616c832";
  const prefix = "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/";
  const renames = git(
    "diff",
    "--name-status",
    "--find-renames",
    historicalBase,
    "a9dec43c406c7afd1d2ce8067fc563678e28ac64",
  ).toString();
  for (const [index, name] of ["assignments", "scopes"].entries()) {
    const from = prefix + "retained_" + name + ".rs",
      to = prefix + "retained/" + name + ".rs";
    const record = witness.producer.historicalRenames[index];
    assert.equal(record.status, "R100");
    assert.equal(record.fromSource, historicalBase);
    assert.equal(record.toSource, "a9dec43c406c7afd1d2ce8067fc563678e28ac64");
    assert.equal(record.from, from);
    assert.equal(record.to, to);
    assert.ok(renames.split("\n").includes(["R100", from, to].join("\t")));
    assert.equal(
      git("ls-tree", "--name-only", witness.producer.composedBase, "--", from).length,
      0,
    );
    assert.equal(record.absentAtBase, true);
    assert.equal(digest(git("show", historicalBase + ":" + from)), record.fromSha256);
    assert.equal(digest(git("show", record.toSource + ":" + to)), record.toSha256);
    assert.equal(record.fromSha256, record.toSha256, "the whole renamed source stays exact");
  }
});

void test("the bounded mode has one Linux Node24 job and reuses every fresh-install step", () => {
  assert.equal(steps, workflow.jobs["fresh-install-smoke"].steps);
  const job = workflow.jobs["packed-config-acceptance"];
  assert.equal(job["runs-on"], "ubuntu-22.04");
  assert.deepEqual(job.strategy.matrix, {
    "node-version": ["24"],
    platform: [{ runner: "ubuntu-22.04", target: "linux-x64-gnu", msvc_arch: "amd64" }],
  });
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const target of [undefined, "all", "darwin-arm64", "packed-config"])
      for (const hostOnly of [false, true]) {
        const context = { github: { event_name: event }, inputs: { target, host_only: hostOnly } };
        const selected = Object.entries(workflow.jobs)
          .filter(([, entry]) => runInNewContext((entry as { if: string }).if, context))
          .map(([name]) => name);
        if (event === "workflow_dispatch" && target === "packed-config")
          assert.deepEqual(selected, ["packed-config-acceptance"]);
        else assert.ok(!selected.includes("packed-config-acceptance"));
      }
});

void test("the complete old and new recipes reject unknown, missing, or altered commands", () => {
  withoutPackedConfigExtension(source);
  for (const command of [
    "cargo build --profile ci -p vize",
    "--runtime-checks --keep-temp",
    "['diff', '--exit-code', 'HEAD']",
    "node tools/support/compat/npm/smoke-release-config-evidence.ts collect",
  ]) {
    assert.ok(source.includes(command), command);
    assert.throws(() => withoutPackedConfigExtension(source.replace(command, "removed-command")));
  }
  assert.throws(() => withoutPackedConfigExtension(source + "\n# unreviewed extension\n"));
});

void test("the actual source preflight refuses a wrong immutable SHA and retains its receipt", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-source-"));
  try {
    const current = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
    }).trim();
    const script = steps.find(
      (step: { name?: string }) =>
        step.name === "Authenticate selected packed configuration source",
    ).run;
    const child = spawnSync("bash", ["-e", "-o", "pipefail", "-c", script], {
      cwd: root,
      encoding: "utf8",
      timeout: 10_000,
      env: {
        ...process.env,
        PACKED_CONFIG_OUTPUT: directory,
        PACKED_CONFIG_EXPECTED_SOURCE: "b".repeat(40),
        PACKED_CONFIG_HOST_ONLY: "false",
        PACKED_CONFIG_PRODUCER:
          workflow.jobs["packed-config-acceptance"].env.PACKED_CONFIG_PRODUCER,
        GITHUB_SHA: current,
        GITHUB_WORKFLOW_SHA: current,
      },
    });
    assert.equal(child.error, undefined);
    assert.equal(child.signal, null);
    assert.notEqual(child.status, 0);
    assert.match(child.stderr, /AssertionError/u);
    assert.match(child.stderr, /'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'/u);
    const receipt = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
    assert.equal(receipt.expected, "b".repeat(40));
    assert.equal(receipt.source, current);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("the actual packed runtime shell retains the full packet without hiding exit17", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-shell-"));
  try {
    const binary = path.join(directory, "rust-script");
    fs.writeFileSync(
      binary,
      '#!/bin/sh\nprintf "required-whole-packet\\n"\nprintf "%s\\n" "$@" > "$PACKED_CONFIG_OUTPUT/argv.txt"\nexit 17\n',
      { mode: 0o755 },
    );
    const script = steps.find(
      (step: { name?: string }) =>
        step.name === "Fresh install and runtime smoke published tarballs",
    ).run;
    const child = spawnSync("bash", ["-e", "-o", "pipefail", "-c", script], {
      cwd: root,
      encoding: "utf8",
      timeout: 10_000,
      env: {
        ...process.env,
        PACKED_CONFIG_OUTPUT: directory,
        PATH: `${directory}${path.delimiter}${process.env.PATH}`,
      },
    });
    assert.equal(child.error, undefined);
    assert.equal(child.signal, null);
    assert.equal(child.status, 17);
    assert.equal(
      fs.readFileSync(path.join(directory, "runtime.log"), "utf8"),
      "required-whole-packet\n",
    );
    const argv = fs.readFileSync(path.join(directory, "argv.txt"), "utf8").trim().split("\n");
    for (const expected of [
      "--force",
      "--prepare-manifests",
      "--runtime-checks",
      "--keep-temp",
      "npm/native",
      "npm/cli",
      "npm/builder/vite",
    ])
      assert.ok(argv.includes(expected), expected);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

// Execute the unchanged real preflight in synthetic history, without builds/installs.
void test("all reviewed producer files and deletion tombstones are bound", () => {
  const script = steps.find(
    (step: { name?: string }) => step.name === "Authenticate selected packed configuration source",
  ).run as string;
  verifyPackedProducerPreflight(script);
});
