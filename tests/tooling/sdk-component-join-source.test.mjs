import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  admittedSdkHistoricalView,
  authenticateSdkComponentJoin,
} from "./support/sdk-component-join.mjs";
import { runSdkJoinedHistoricalReaderLaws } from "./support/sdk-component-join-replay.mjs";
import { joinGit, joinSHA } from "./support/no-mutating-component-join-physical.mjs";
import { retainedAsciiCallerFrame } from "./support/sdk-ascii-symbol-original-callers.ts";
import {
  authenticateNoMutatingComponentJoinPhysical,
  componentJoinDirectory,
} from "./support/no-mutating-component-join.mjs";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const data = JSON.parse(
  fs.readFileSync(new URL("./support/sdk-component-join.json", import.meta.url)),
);
void test("actual SDK112 and NoMutating323 retain all source authorities before historical five-law replay", () => {
  const current = authenticateSdkComponentJoin(root);
  assert.equal(current.currentSdkBodies, 112);
  assert.equal(current.currentSdkSuccessors, 10);
  assert.equal(current.currentNoMutatingBodies, 323);
  assert.equal(current.currentGraphBodies, 237);
  assert.equal(current.actualSdkAsciiBodies, 10);
  assert.equal(current.currentOld111Receipt, null);
  const loan = admittedSdkHistoricalView(root, current);
  const expected = Buffer.from(loan.oldBodies.get(data.originalDriver));
  loan.oldBodies.get(data.originalDriver)[0] ^= 1;
  loan.oldBodies.delete(data.originalDriver);
  loan.oldRows[0].sha256 = "f".repeat(64);
  loan.model.rows[0].final.sha256 = "f".repeat(64);
  const again = admittedSdkHistoricalView(root, current);
  assert.deepEqual(again.oldBodies.get(data.originalDriver), expected);
  assert.equal(joinSHA(again.oldBodies.get(data.originalDriver)), data.originalDriverSHA256);
  assert.notEqual(again.oldRows[0].sha256, "f".repeat(64));
  assert.notEqual(again.model.rows[0].final.sha256, "f".repeat(64));
  const historical = runSdkJoinedHistoricalReaderLaws(root);
  assert.equal(historical.currentSource, current.head);
  assert.equal(historical.historicalSource, data.sdkSource);
  assert.equal(historical.historicalDriverRows, 233);
  assert.equal(historical.historicalSourceBodies, 187);
  assert.equal(historical.passed, 5);
  assert.equal(historical.skipped, 0);
  assert.equal(historical.historicalProjection, true);
  assert.equal(historical.nativeCredit, false);
});
function privateCurrent(operation) {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sdk-current-join-refusal-"));
  try {
    // Source controls mutate only an owned copy; the repository and source objects stay untouched.
    joinGit(fixture, ["init", "--quiet"]);
    const common = joinGit(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(fixture, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
    joinGit(fixture, ["read-tree", head]);
    // Copy only the finite authenticated closure, never a whole checkout or its artifacts.
    const joined = authenticateNoMutatingComponentJoinPhysical(root);
    const privateFiles = JSON.parse(
      fs.readFileSync(new URL("./support/sdk-component-join-files.json", import.meta.url)),
    );
    const files = new Set([
      ...privateFiles,
      ...retainedAsciiCallerFrame("owned-sdk-reader").map(({ file }) => file),
      ...ownedCorsaSource.rows.map(({ file }) => file),
      ...joined.manifest.current.map(({ path }) => path),
      ...joined.manifest.literalFrames.map(({ path }) => path),
      joined.manifest.externalMaintenanceLaw.path,
      componentJoinDirectory + "/successor.json",
      "tests/tooling/no-mutating-component-join-source.test.mjs",
      "tests/tooling/support/sdk-ascii-symbol-original-callers.ts",
      "tests/tooling/support/sdk-component-join.mjs",
      "tests/tooling/support/sdk-component-join-replay.mjs",
      "tests/tooling/support/sdk-component-join.json",
      "tests/tooling/support/sdk-component-join-files.json",
      "tests/tooling/sdk-ascii-symbol-original-callers.test.ts",
      "tests/tooling/sdk-component-join-source.test.mjs",
      ...[
        "no-mutating-component-join.mjs",
        "no-mutating-component-join-physical.mjs",
        "no-mutating-component-join-frame.mjs",
        "no-mutating-component-join-runner.mjs",
        "no-mutating-props-current-caller-frame.mjs",
      ].map((name) => "tests/tooling/support/" + name),
      ...[
        "owned-corsa-source-custody.mjs",
        "owned-corsa-clippy-source.mjs",
        "owned-corsa-clippy-source.json",
        "owned-corsa-source-manifest.json",
        ...Array.from(
          { length: 20 },
          (_, i) => `owned-corsa-source-part-${String(i).padStart(2, "0")}.json`,
        ),
      ].map((name) => "tools/support/compat/github/" + name),
    ]);
    for (const name of files) {
      const target = path.join(root, name);
      const stat = fs.lstatSync(target);
      assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644, name);
      const destination = path.join(fixture, name);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, fs.readFileSync(target), {
        mode: stat.mode & 0o777,
        flag: "wx",
      });
    }
    joinGit(fixture, ["update-ref", "refs/heads/current-source", head]);
    joinGit(fixture, ["symbolic-ref", "HEAD", "refs/heads/current-source"]);
    assert.equal(authenticateSdkComponentJoin(fixture).head, head);
    return operation(fixture, head);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}
void test("every ten actual successor refuses historical, missing, changed and foreign-mode physical source", () => {
  privateCurrent((fixture) => {
    for (const row of data.changes) {
      const target = path.join(fixture, row.file);
      const body = fs.readFileSync(target);
      const old = joinGit(root, ["cat-file", "blob", row.before.blob]);
      for (const changed of [old, Buffer.concat([body, Buffer.from("\n// unknown\n")])]) {
        fs.writeFileSync(target, changed);
        assert.throws(() => authenticateSdkComponentJoin(fixture));
        fs.writeFileSync(target, body);
      }
      fs.unlinkSync(target);
      assert.throws(() => authenticateSdkComponentJoin(fixture));
      fs.writeFileSync(target, body, { mode: 0o644 });
      fs.chmodSync(target, 0o755);
      assert.throws(() => authenticateSdkComponentJoin(fixture));
      fs.chmodSync(target, 0o644);
      const same = path.join(fixture, "source-copy.txt");
      fs.writeFileSync(same, body);
      fs.unlinkSync(target);
      fs.symlinkSync(same, target);
      assert.throws(() => authenticateSdkComponentJoin(fixture));
      fs.unlinkSync(target);
      fs.unlinkSync(same);
      fs.writeFileSync(target, body, { mode: 0o644 });
    }
    assert.equal(authenticateSdkComponentJoin(fixture).currentSdkSuccessors, 10);
  });
});
void test("real raw changed body/mode and replacement source refuse without lending a historical tuple", () => {
  privateCurrent((fixture, head) => {
    for (const row of data.changes) {
      const original = fs.readFileSync(path.join(fixture, row.file));
      const changed = Buffer.concat([original, Buffer.from("\n// unknown raw source\n")]);
      const blob = joinGit(fixture, ["hash-object", "-w", "--stdin"], changed).toString().trim();
      for (const mode of ["100644", "100755", "120000"]) {
        joinGit(fixture, ["read-tree", head]);
        joinGit(fixture, ["update-index", "--cacheinfo", mode + "," + blob + "," + row.file]);
        const tree = joinGit(fixture, ["write-tree"]).toString().trim();
        const source = joinGit(
          fixture,
          [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit-tree",
            tree,
          ],
          "foreign source\n",
        )
          .toString()
          .trim();
        joinGit(fixture, ["update-ref", "refs/heads/current-source", source]);
        assert.throws(() => authenticateSdkComponentJoin(fixture));
        joinGit(fixture, ["update-ref", "refs/heads/current-source", head]);
      }
    }
    for (const file of [
      "tests/tooling/support/sdk-component-join.mjs",
      "tests/tooling/support/sdk-component-join-replay.mjs",
      "tests/tooling/support/sdk-component-join.json",
      "tests/tooling/support/sdk-component-join-files.json",
      "tests/tooling/support/sdk-ascii-symbol-original-callers.ts",
      "tests/tooling/sdk-ascii-symbol-original-callers.test.ts",
      "tests/tooling/sdk-component-join-source.test.mjs",
    ]) {
      const target = path.join(fixture, file);
      const original = fs.readFileSync(target);
      const changed = Buffer.concat([original, Buffer.from("\n ")]);
      fs.writeFileSync(target, changed);
      joinGit(fixture, ["read-tree", head]);
      joinGit(fixture, ["add", "--", file]);
      const tree = joinGit(fixture, ["write-tree"]).toString().trim();
      const source = joinGit(
        fixture,
        [
          "-c",
          "user.name=Fixture",
          "-c",
          "user.email=fixture@example.invalid",
          "commit-tree",
          tree,
        ],
        "unknown enrolled-source body\n",
      )
        .toString()
        .trim();
      joinGit(fixture, ["update-ref", "refs/heads/current-source", source]);
      assert.throws(() => authenticateSdkComponentJoin(fixture));
      joinGit(fixture, ["update-ref", "refs/heads/current-source", head]);
      fs.writeFileSync(target, original);
    }
    joinGit(fixture, ["replace", head, data.sdkSource]);
    assert.throws(() => authenticateSdkComponentJoin(fixture));
    joinGit(fixture, ["replace", "-d", head]);
    assert.equal(authenticateSdkComponentJoin(fixture).head, head);
    assert.equal(
      joinSHA(fs.readFileSync(path.join(fixture, data.originalDriver))),
      data.changes.find(({ file }) => file === data.originalDriver).after.sha256,
    );
  });
});
