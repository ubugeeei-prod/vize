import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUILD_RECIPE,
  validateBuildReceipt,
  type BuildIdentity,
} from "../../../differential/build-receipt.ts";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { object, sha256 } from "../project-uri-membership-10622.ts";
import { verifyIndependentObservation } from "./project-uri-source-control-custody-10622.ts";

const OBSERVER = "df4b7d5e406bece8325af403f1668d6573c04184";
const QUALIFIER = "6573b92e74c93883303e34e9f12869868f567207";
const OBSERVER_FILE = "tests/tooling/support/lsp/project-uri-source-control-10622.ts";
const SOURCES = { control: "225b884276cdc066a692bf2cd1b51a15e952e97b", candidate: QUALIFIER };
const text = (value: unknown): string => {
  assert.equal(typeof value, "string");
  assert.ok(typeof value === "string");
  return value;
};
const gitBlob = (root: string, revision: string, file: string) =>
  execFileSync("git", ["show", `${revision}:${file}`], { cwd: root });

/** Historical file identity is not a newly captured call-site witness. */
export function validateRecordedObserver(root: string, identities: unknown) {
  const row = object(identities);
  assert.equal(row.experimentRevision, OBSERVER);
  assert.equal(row.qualifierRevision, QUALIFIER);
  const observer = object(row.observer);
  assert.equal(observer.file, `/home/runner/_work/vize/vize/${OBSERVER_FILE}`);
  assert.equal(observer.sha256, sha256(gitBlob(root, OBSERVER, OBSERVER_FILE)));
  const qualifier = object(row.qualifierFiles);
  assert.deepEqual(
    Object.keys(qualifier).sort(),
    [
      "tests/differential/build-receipt.ts",
      "tests/tooling/support/lsp/session.ts",
      "tests/tooling/support/lsp/session-process.ts",
      "tests/tooling/support/lsp/launch.ts",
      "tests/tooling/support/lsp/published-launch.ts",
      "tests/tooling/support/lsp/verify-session-captures.ts",
      "tests/tooling/support/project-uri-membership-10622.ts",
      "tests/tooling/support/project-uri-membership-position-10622.ts",
      "tests/tooling/support/project-uri-membership-provider-10622.ts",
      "editors/vscode/test/fixtures/passive-vize-server.cjs",
    ].sort(),
  );
  for (const [file, digest] of Object.entries(qualifier)) {
    assert.ok(!file.split("/").includes("..") && !path.isAbsolute(file));
    assert.equal(digest, sha256(gitBlob(root, QUALIFIER, file)));
  }
  return {
    authority: "recorded-independent-observer",
    revision: OBSERVER,
    file: OBSERVER_FILE,
    sha256: observer.sha256,
    qualifierRevision: QUALIFIER,
    qualifierFiles: qualifier,
    callSiteWitnessCaptured: false,
  };
}

function recordedBuild(value: unknown): BuildIdentity {
  const row = object(value);
  return {
    sourceRevision: text(row.sourceRevision),
    binaryPath: text(row.binaryPath),
    binarySha256: text(row.binarySha256),
    cliVersion: text(row.cliVersion),
  };
}

/** No binary is launched, reconstructed, or rehashed by this archived-record authority. */
export function verifyRecordedTrial(
  root: string,
  producerValue: unknown,
  identitiesValue: unknown,
  directory: string,
) {
  const producer = object(producerValue),
    identities = object(identitiesValue);
  const role = identities.role;
  assert.ok(role === "control" || role === "candidate");
  assert.ok(identities.trial === "1" || identities.trial === "2");
  assert.deepEqual(identities.producer, producer);
  assert.equal(producer.state, "built");
  assert.equal(producer.role, role);
  assert.equal(producer.sourceRevision, SOURCES[role]);
  const sourceRoot = `/home/runner/_work/_temp/uri-source-control-producers/${role}`;
  assert.equal(producer.sourceRoot, sourceRoot);
  assert.equal(producer.targetRoot, `${sourceRoot}/target`);
  assert.equal(producer.targetInitiallyAbsent, true);
  const build = object(producer.build);
  assert.deepEqual(build.argv, ["cargo", "build", "--profile", "ci", "-p", "vize"]);
  assert.equal(build.recipe, BUILD_RECIPE);
  assert.equal(build.cwd, sourceRoot);
  assert.equal(build.targetRoot, producer.targetRoot);
  assert.deepEqual(build.environment, {
    CARGO_TARGET_DIR: `${sourceRoot}/target`,
    CARGO_INCREMENTAL: "0",
    RUSTC_WRAPPER: "",
    RUSTC_WORKSPACE_WRAPPER: "",
    SCCACHE_DISABLE: "1",
  });
  const locks = object(producer.locks);
  assert.deepEqual(Object.keys(locks).sort(), [
    "Cargo.lock",
    "Cargo.toml",
    "package.json",
    "pnpm-lock.yaml",
  ]);
  for (const [file, digest] of Object.entries(locks))
    assert.equal(digest, sha256(gitBlob(root, SOURCES[role], file)));
  const expected = recordedBuild(producer.receipt);
  assert.equal(expected.sourceRevision, SOURCES[role]);
  assert.equal(expected.binaryPath, "target/ci/vize");
  validateBuildReceipt(producer.receipt, expected, BUILD_RECIPE);
  const observer = validateRecordedObserver(root, identities);
  assert.notEqual(
    sourceRoot,
    path.posix.dirname(text(object(identities.observer).file)).split("/tests/")[0],
  );
  const observation = verifyIndependentObservation(directory, expected);
  const client = decodeFrames(fs.readFileSync(path.join(directory, "client.bin"))).messages;
  const initialize = client.filter((packet) => packet.method === "initialize");
  assert.equal(initialize.length, 1);
  const params = object(initialize[0].params);
  assert.equal(fileURLToPath(text(params.rootUri)), identities.workspace);
  assert.equal(
    identities.workspace,
    `${sourceRoot}/target/uri-source-control-workspaces/${role}-${identities.trial}`,
  );
  return {
    authority: "archived-dual-root-record",
    producer,
    observer,
    observation,
    producerBinaryRehashed: false,
    genericSourceWitnessAdmission: false,
    semanticAcceptance: false,
  };
}
