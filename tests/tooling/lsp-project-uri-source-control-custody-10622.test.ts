import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { root as repoRoot, testOutputRoot } from "./support/lsp/paths.ts";
import { sha256 } from "./support/project-uri-membership-10622.ts";
import { validateRecordedObserver } from "./support/lsp/project-uri-source-control-recorded-10622.ts";
import {
  independentObserverWitness,
  validateObserverWitness,
  verifyIndependentObserverCaptures,
} from "./support/lsp/project-uri-source-control-custody-10622.ts";

async function fixture() {
  const base = path.join(testOutputRoot, "project-uri-source-control-10622/custody-controls");
  fs.mkdirSync(base, { recursive: true });
  const root = fs.realpathSync(fs.mkdtempSync(path.join(base, "observer-")));
  const file = path.join(root, "tests/tooling/observer.mjs");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(
    file,
    'export function stack() { return new Error("actual observer fixture").stack; }\n',
  );
  execFileSync("git", ["init", "--quiet"], { cwd: root });
  execFileSync("git", ["add", "tests"], { cwd: root });
  execFileSync(
    "git",
    [
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.test",
      "commit",
      "--quiet",
      "-m",
      "test: record independent observer fixture",
    ],
    { cwd: root },
  );
  const loaded = await import(pathToFileURL(file).href);
  const witness = independentObserverWitness(root, file, loaded.stack());
  return { root, file, witness };
}

test("independent observer custody requires its actual committed call site and full source identity", async () => {
  const { root, witness } = await fixture();
  validateObserverWitness(witness);
  for (const changed of [
    { ...witness, sha256: "0".repeat(64) },
    { ...witness, revision: "0".repeat(40) },
    { ...witness, tree: "0".repeat(40) },
    { ...witness, line: 0 },
    { ...witness, line: 1000000 },
    { ...witness, column: 1000000 },
    { ...witness, column: 0 },
    { ...witness, path: "tests/tooling/../../observer.mjs" },
    { ...witness, repoRoot: path.dirname(root) },
  ])
    assert.throws(() => validateObserverWitness(changed));
  const historical = "df4b7d5e406bece8325af403f1668d6573c04184";
  const qualifier = "6573b92e74c93883303e34e9f12869868f567207";
  const file = "tests/tooling/support/lsp/project-uri-source-control-10622.ts";
  const digest = (revision: string, name: string) =>
    sha256(execFileSync("git", ["show", `${revision}:${name}`], { cwd: repoRoot }));
  const names = [
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
  ];
  const recorded = {
    experimentRevision: historical,
    qualifierRevision: qualifier,
    observer: { file: `/home/runner/_work/vize/vize/${file}`, sha256: digest(historical, file) },
    qualifierFiles: Object.fromEntries(names.map((name) => [name, digest(qualifier, name)])),
  };
  const authority = validateRecordedObserver(repoRoot, recorded);
  assert.equal(authority.authority, "recorded-independent-observer");
  assert.equal(authority.callSiteWitnessCaptured, false);
  for (const changed of [
    { ...recorded, experimentRevision: qualifier },
    { ...recorded, qualifierRevision: historical },
    { ...recorded, observer: { ...recorded.observer, sha256: "0".repeat(64) } },
    { ...recorded, observer: { ...recorded.observer, file: path.join(root, file) } },
    { ...recorded, qualifierFiles: { [file]: "0".repeat(64) } },
  ])
    assert.throws(() => validateRecordedObserver(repoRoot, changed));
  assert.throws(
    () => independentObserverWitness(root, path.join(root, witness.path), "unrelated stack"),
    /actual call-site witness/u,
  );
});

test("observer edits after capture refuse rather than borrowing a producer witness", async () => {
  const { root, file, witness } = await fixture();
  fs.appendFileSync(file, "export const changedAfterCapture = true;\n");
  assert.throws(
    () => validateObserverWitness(witness),
    /actual independent observer remains exact/u,
  );
  fs.writeFileSync(
    file,
    'export function stack() { return new Error("actual observer fixture").stack; }\n',
  );
  validateObserverWitness(witness);
  assert.throws(
    () =>
      verifyIndependentObserverCaptures(
        root,
        {
          version: 1,
          binary: {
            sourceRevision: "0".repeat(40),
            binaryPath: "unlaunched-negative-control",
            binarySha256: "0".repeat(64),
            cliVersion: "vize 0.0.0-dev",
          },
          prior: [],
        },
        witness,
      ),
    /distinct roots/u,
  );
});
