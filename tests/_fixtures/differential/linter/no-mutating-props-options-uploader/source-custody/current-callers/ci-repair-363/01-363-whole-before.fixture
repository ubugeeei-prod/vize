import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { qualifyDefaultMigrationHost } from "../../performance/support/warm-type-backed-default-migration-host.ts";
import {
  qualifyGitReaderSetupHarness,
  gitReaderSetupChanges,
} from "../../performance/support/warm-type-backed-git-reader-setup.ts";
import { gitReaderNoDeepSource } from "../../performance/support/warm-type-backed-git-reader-nodeep.ts";
import { privateNoMutatingCallerGit } from "./no-mutating-props-current-caller-git.mjs";
import { authenticateNoMutatingModuleLayoutSourceFamily } from "./no-mutating-props-module-layout-source-custody.mjs";
import {
  runNoMutatingLiteralCaller,
  rawNoMutatingCallerDigest as currentCallerRawDigest,
} from "./no-mutating-props-current-caller-frame.mjs";

export const noMutatingCurrentCallerDirectory =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/current-callers";
export const noMutatingCurrentCallerManifestSHA256 =
  "fa45cf8fa852f0c222af9378a981cb77d51d33f2bf58bad346230e4a00f8f532";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const reader = (root) => (name) => fs.readFileSync(path.join(root, name));

export function composeNoMutatingCurrentCallerLaw(law, cases) {
  return Buffer.from(
    'import path from "node:path";\nimport { fileURLToPath } from "node:url";\nimport { test } from "node:test";\n' +
      'import { runNoMutatingCurrentCallerHistory } from "./support/no-mutating-props-current-caller-source-custody.mjs";\n' +
      'const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");\n' +
      'void test("current complete23 refuses old tuples before the complete historical ' +
      cases +
      '-case caller", async () => {\n  await runNoMutatingCurrentCallerHistory(\n    root,\n    ' +
      JSON.stringify(law) +
      ",\n  );\n});\n",
  );
}

function identity(body, expected) {
  assert.equal(body.length, expected.bytes, expected.path);
  assert.equal(sha(body), expected.sha256, expected.path);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    expected.gitBlob,
    expected.path,
  );
}

/** Current whole23 is separate from every immutable historical measurement tuple. */
export function authenticateNoMutatingCurrentCallerSource(root, read = reader(root)) {
  const check = (name, expected) => {
    assert(name && !path.isAbsolute(name) && path.normalize(name) === name);
    assert(!name.split("/").includes(".."), name);
    const target = path.join(root, name);
    const status = fs.lstatSync(target);
    assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111), name);
    assert.equal(fs.realpathSync(target), path.join(fs.realpathSync(root), name), name);
    const body = read(name);
    identity(body, expected);
    assert.deepEqual(fs.readFileSync(target), body, name);
    return body;
  };
  const manifestPath = noMutatingCurrentCallerDirectory + "/successor.json";
  const bytes = read(manifestPath);
  assert.equal(sha(bytes), noMutatingCurrentCallerManifestSHA256);
  const manifest = JSON.parse(bytes);
  check(manifestPath, {
    path: manifestPath,
    bytes: bytes.length,
    sha256: sha(bytes),
    gitBlob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
  });
  assert.equal(manifest.schema, "vize.no-mutating.current-callers.source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.source, "2489128d5f8fab1b7442a1e5c09b4e1701cbf185");
  assert.deepEqual(manifest.familyCounts, [12, 11]);
  assert.equal(
    manifest.moduleLayoutManifestSHA256,
    "e53d557df855bf8a365e56cdf3cd64772ea33b8032690df34eaed0c6ed444c5e",
  );
  for (const module of manifest.closedCurrentModules) check(module.path, module);
  const current = authenticateNoMutatingModuleLayoutSourceFamily(root, read);
  assert.equal(current.paths.length, 23);
  assert.equal(current.manifest.pairs.length, 23);
  assert.equal(manifest.rule.path, "crates/vize_patina/src/rule.rs");
  const before = check(manifest.rule.before.archive, manifest.rule.before);
  const after = check(manifest.rule.after.archive, manifest.rule.after);
  check(manifest.rule.path, manifest.rule.after);
  const insertion =
    "\n    /// Borrow this actual configured instance's ordinary-SFC script callback.\n" +
    "    /// Names alone confer no capability. The existing shared parse supplies\n" +
    "    /// the Program; unprovided instances keep their original dispatch.\n" +
    "    fn as_sfc_script_rule(&self) -> Option<&dyn crate::rules::script::ScriptRule> {\n" +
    "        None\n    }\n";
  assert.equal(after.toString().split(insertion).length, 2);
  assert.deepEqual(Buffer.from(after.toString().replace(insertion, "")), before);
  assert.deepEqual(
    manifest.laws.map(({ path: name, cases }) => [name, cases]),
    [
      ["tests/tooling/reserved-suppression-default-routes.test.mjs", 1],
      ["tests/tooling/warm-type-backed-default-migration-host.test.ts", 7],
      ["tests/tooling/warm-type-backed-git-reader-nodeep.test.ts", 5],
    ],
  );
  for (const law of manifest.laws) {
    check(law.archive, law);
    assert.deepEqual(
      check(law.path, law.after),
      composeNoMutatingCurrentCallerLaw(law.path, law.cases),
    );
  }
  check(manifest.historicalFrame.path, manifest.historicalFrame);
  const migration = JSON.parse(
    read("tests/performance/support/warm-type-backed-default-migration-manifest.json"),
  );
  const expected = new Map(
    migration.files.map(([file, old, now]) => [file, { before: old, after: now }]),
  );
  const preserved = new Map(migration.preservedBodies);
  const physical = (side, file) =>
    side === "before"
      ? (expected.get(file)?.before ?? preserved.get(file) ?? null)
      : sha(fs.readFileSync(path.join(root, file)));
  assert.equal(
    qualifyDefaultMigrationHost([...expected.keys()], new Set(), physical),
    null,
    "actual current source must not receive the old measured migration tuple",
  );
  // A successful historical whole87 proof is not a current-source setup receipt.
  const beforeRoot = privateNoMutatingCallerGit(root, gitReaderNoDeepSource);
  const actualSetup = (side, file) => {
    if (side === "after") {
      try {
        return sha(fs.readFileSync(path.join(root, file)));
      } catch (error) {
        assert.equal(error.code, "ENOENT");
        return null;
      }
    }
    return currentCallerRawDigest(beforeRoot, gitReaderNoDeepSource, file);
  };
  assert.equal(qualifyGitReaderSetupHarness(gitReaderSetupChanges, new Set(), actualSetup), null);
  return {
    manifest,
    paths: current.paths,
    completeCurrentBodies: 23,
    historicalProjection: false,
    oldMeasurement: null,
    oldSetup: null,
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  };
}

/** All original law bodies execute in their exact committed2489 frame, never current87. */
export async function runNoMutatingCurrentCallerHistory(root, law) {
  const context = authenticateNoMutatingCurrentCallerSource(root);
  const declared = context.manifest.laws.find(({ path: name }) => name === law);
  assert(declared, "unknown historical caller");
  return runNoMutatingLiteralCaller(root, context, declared);
}
