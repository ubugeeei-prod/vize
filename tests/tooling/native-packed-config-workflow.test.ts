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

void test("the refrozen composed producer binds every whole source and moved test module", () => {
  const producer = "a9dec43c406c7afd1d2ce8067fc563678e28ac64";
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
  assert.equal(files.length, 197);
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
  assert.equal(changedRust.length, 193);
  const historicalAbsences = witness.producer.historicalRenames.map(
    (rename: { from: string }) => rename.from,
  );
  assert.deepEqual(
    files.filter((file) => file.endsWith(".rs") && !historicalAbsences.includes(file)).sort(),
    changedRust.sort(),
    "complete Rust composition, including additions and deletion, is authenticated",
  );
  assert.deepEqual(
    files.filter((file) => !file.endsWith(".rs")),
    [
      "crates/vize_carton/src/config/loader/module-runtime.d.mts",
      "crates/vize_carton/src/config/loader/module-runtime.mjs",
    ],
  );
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
  assert.equal(deletion.toSource, producer);
  assert.equal(digest(git("show", deletion.fromSource + ":" + deletion.from)), deletion.fromSha256);
  assert.equal(digest(git("show", producer + ":" + deletion.to)), deletion.toSha256);
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
  assert.deepEqual(removedRust, [deletion.from]);
  for (const file of files) {
    if (witness.producer.absentFiles.includes(file)) {
      assert.equal(witness.producer.files[file], null);
      assert.equal(git("ls-tree", "--name-only", producer, "--", file).length, 0);
      assert.equal(fs.lstatSync(path.join(root, file), { throwIfNoEntry: false }), undefined);
    } else {
      assert.equal(digest(git("show", producer + ":" + file)), witness.producer.files[file], file);
      assert.equal(
        digest(fs.readFileSync(path.join(root, file))),
        witness.producer.files[file],
        file,
      );
    }
  }
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
  const historicalBase = "85b7c8d4b12226b67eaf23dd7b84fb14b616c832";
  const prefix = "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/";
  const renames = git(
    "diff",
    "--name-status",
    "--find-renames",
    historicalBase,
    producer,
  ).toString();
  for (const [index, name] of ["assignments", "scopes"].entries()) {
    const from = prefix + "retained_" + name + ".rs",
      to = prefix + "retained/" + name + ".rs";
    const record = witness.producer.historicalRenames[index];
    assert.equal(record.status, "R100");
    assert.equal(record.fromSource, historicalBase);
    assert.equal(record.toSource, producer);
    assert.equal(record.from, from);
    assert.equal(record.to, to);
    assert.ok(renames.split("\n").includes(["R100", from, to].join("\t")));
    assert.equal(
      git("ls-tree", "--name-only", witness.producer.composedBase, "--", from).length,
      0,
    );
    assert.equal(record.absentAtBase, true);
    assert.equal(digest(git("show", historicalBase + ":" + from)), record.fromSha256);
    assert.equal(digest(git("show", producer + ":" + to)), record.toSha256);
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
