/** Reverse only the named, complete packed-config extension before old custody laws. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { runInNewContext } from "node:vm";

interface Extension {
  schema: string;
  beforeSha256: string;
  afterSha256: string;
  hunks: { before: string; after: string }[];
  producer: { head: string; files: Record<string, string | null>; absentFiles: string[] };
}
const witness = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/vite-native-image-identity/packed-config-extension.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as Extension;
const digest = (source: string) => createHash("sha256").update(source).digest("hex");

export function withoutPackedConfigExtension(source: string): string {
  assert.equal(witness.schema, "vize.native-smoke-packed-config-extension.v1");
  assert.equal(
    digest(source),
    witness.afterSha256,
    "complete current workflow matches the named extension",
  );
  for (const hunk of witness.hunks) {
    assert.equal(
      source.split(hunk.after).length,
      2,
      "each whole extension hunk occurs exactly once",
    );
    source = source.replace(hunk.after, hunk.before);
  }
  assert.equal(
    digest(source),
    witness.beforeSha256,
    "the complete pre-extension workflow stays exact",
  );
  return source;
}

/** Execute the unchanged real guard in synthetic Git history, never a native claim. */
export function verifyPackedProducerPreflight(script: string): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-producer-"));
  try {
    const repository = path.join(directory, "repo");
    fs.mkdirSync(repository);
    const match = /const files = (\[[\s\S]*?\]);/u.exec(script);
    assert.ok(match);
    const sourceFiles = runInNewContext(match[1]) as string[];
    const producerFiles = Object.keys(witness.producer.files);
    assert.equal(producerFiles.length, 197);
    const fixtureBytes = "reviewed synthetic fixture bytes\n";
    for (const name of [...sourceFiles, ...producerFiles]) {
      if (witness.producer.absentFiles.includes(name)) continue;
      const file = path.join(repository, name);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, fixtureBytes);
    }
    fs.writeFileSync(
      path.join(repository, "rust-toolchain.toml"),
      '[toolchain]\nchannel = "1.99.1"\n',
    );
    const git = (...args: string[]) =>
      execFileSync(
        "git",
        [
          "-c",
          "core.hooksPath=/dev/null",
          "-c",
          "commit.gpgsign=false",
          "-c",
          "user.name=Config fixture",
          "-c",
          "user.email=config-fixture@example.invalid",
          ...args,
        ],
        { cwd: repository, encoding: "utf8" },
      ).trim();
    const commit = () => {
      git("add", "--all");
      git("commit", "--quiet", "-m", "test: retain exact guard control inputs");
    };
    git("init", "--quiet", "--initial-branch=main");
    commit();
    const producer = git("rev-parse", "HEAD");
    const fixtureWitness = structuredClone(witness);
    fixtureWitness.producer.head = producer;
    fixtureWitness.producer.files = Object.fromEntries(
      producerFiles.map((file) => [
        file,
        witness.producer.files[file] === null ? null : digest(fixtureBytes),
      ]),
    );
    const witnessPath = path.join(
      repository,
      "tests/_fixtures/differential/vite-native-image-identity/packed-config-extension.json",
    );
    const saveWitness = () => {
      fs.writeFileSync(witnessPath, JSON.stringify(fixtureWitness));
      commit();
    };
    // The witness is a descendant of its producer; no self-referential fake SHA.
    saveWitness();
    const execute = (selectedProducer = producer) => {
      const current = git("rev-parse", "HEAD");
      const child = spawnSync("bash", ["-e", "-o", "pipefail", "-c", script], {
        cwd: repository,
        encoding: "utf8",
        timeout: 10_000,
        env: {
          ...process.env,
          PACKED_CONFIG_OUTPUT: path.join(directory, "evidence"),
          PACKED_CONFIG_EXPECTED_SOURCE: current,
          PACKED_CONFIG_HOST_ONLY: "false",
          PACKED_CONFIG_PRODUCER: selectedProducer,
          GITHUB_SHA: current,
          GITHUB_WORKFLOW_SHA: current,
          GITHUB_OUTPUT: path.join(directory, "outputs"),
        },
      });
      assert.equal(child.error, undefined);
      assert.equal(child.signal, null);
      const receipt = JSON.parse(
        fs.readFileSync(path.join(directory, "evidence/source.json"), "utf8"),
      );
      return { ...child, receipt };
    };
    const admitted = execute();
    assert.equal(admitted.status, 0, admitted.stderr);
    assert.deepEqual(admitted.receipt.producerAbsentFiles, witness.producer.absentFiles);
    assert.deepEqual(admitted.receipt.producerFiles, fixtureWitness.producer.files);
    assert.notEqual(execute("b".repeat(40)).status, 0, "exact producer identity is required");
    const unrelated = git(
      "commit-tree",
      git("rev-parse", `${producer}^{tree}`),
      "-m",
      "test: unrelated identical producer tree",
    );
    fixtureWitness.producer.head = unrelated;
    saveWitness();
    const nonancestor = execute(unrelated);
    assert.notEqual(nonancestor.status, 0, "identical bytes cannot replace source ancestry");
    assert.match(nonancestor.stderr, /merge-base --is-ancestor/u);
    fixtureWitness.producer.head = producer;
    saveWitness();
    const moved =
      "crates/vize_maestro/src/server/state/project_contexts/config_reload_recovery_tests.rs";
    const materialize = "crates/vize_canon/src/batch/virtual_project/materialize.rs";
    for (const file of [
      moved,
      materialize,
      "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_snapshot.rs",
      "crates/vize/src/commands/check/runner/direct.rs",
      "crates/vize_carton/src/config/loader/module-runtime.mjs",
    ]) {
      assert.ok(producerFiles.includes(file));
      const target = path.join(repository, file);
      fs.writeFileSync(target, "changed whole source packet\n");
      assert.notEqual(execute().status, 0, "dirty source cannot pass");
      commit();
      const changed = execute();
      assert.notEqual(changed.status, 0, file);
      assert.notEqual(changed.receipt.files[file], changed.receipt.producerFiles[file]);
      fs.writeFileSync(target, fixtureBytes);
      commit();
      assert.equal(execute().status, 0, "whole source restoration recovers admission");
    }
    for (const file of [
      materialize,
      "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_inputs.rs",
    ]) {
      fs.unlinkSync(path.join(repository, file));
      commit();
      const missing = execute();
      assert.notEqual(missing.status, 0);
      assert.equal(missing.receipt.files[file], null, "missing sources retain failure evidence");
      assert.equal(missing.receipt.reviewedProducerFiles[file], digest(fixtureBytes));
      fs.writeFileSync(path.join(repository, file), fixtureBytes);
      commit();
      assert.equal(execute().status, 0);
    }
    for (const file of witness.producer.absentFiles) {
      const target = path.join(repository, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, "resurrected old source path\n");
      commit();
      const resurrected = execute();
      assert.notEqual(resurrected.status, 0, file);
      assert.equal(typeof resurrected.receipt.files[file], "string");
      fs.unlinkSync(target);
      commit();
      assert.equal(execute().status, 0);
    }
    fixtureWitness.producer.absentFiles.push(materialize);
    fixtureWitness.producer.files[materialize] = null;
    saveWitness();
    assert.notEqual(execute().status, 0, "a real producer file cannot be declared absent");
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
