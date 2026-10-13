import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { authenticateCssReleaseDriver } from "../../tools/support/dependencies/css-engine-release-driver-custody.ts";

const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const fixture =
  "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/stable-driver";
const drivers = [
  [
    "tools/commands/release/pr.rs",
    "pr-original.rs.txt",
    "db53a5ec6abf597c01dfb77b27477cfb10d69bdeba7018ab5920278d0a590d2d",
  ],
  [
    "tools/support/release/pr_pin.rs",
    "pr-pin-original.rs.txt",
    "273b5f7263feb8ddf5bf132ccf329a388f467bf5e1ab586d866ba2ce8eca904d",
  ],
] as const;

test("whole old and exact reviewed current drivers preserve the original references and reject forged bodies/paths/references", () => {
  for (const [file, originalFile, originalHash] of drivers) {
    const old = fs.readFileSync(`${fixture}/${originalFile}`);
    const current = fs.readFileSync(file);
    assert.equal(hash(old), originalHash);
    assert.notEqual(hash(current), originalHash);
    authenticateCssReleaseDriver(file, old, originalHash);
    authenticateCssReleaseDriver(file, current, originalHash);
    for (const source of [old, current]) {
      const bad = Buffer.from(source);
      bad[0] ^= 1;
      assert.throws(() => authenticateCssReleaseDriver(file, bad, originalHash));
    }
    assert.throws(() => authenticateCssReleaseDriver(file, current, hash(current)));
    for (const wrong of [
      "__proto__",
      "constructor",
      "tools/commands/release/unreviewed.rs",
      drivers.find((row) => row[0] !== file)![0],
    ]) {
      assert.throws(() => authenticateCssReleaseDriver(wrong, current, originalHash));
    }
  }
  const unrelated = Buffer.from("whole unchanged unrelated source\n");
  authenticateCssReleaseDriver("other/source.rs", unrelated, hash(unrelated));
  assert.throws(() =>
    authenticateCssReleaseDriver(
      "other/source.rs",
      Buffer.concat([unrelated, Buffer.from("extra")]),
      hash(unrelated),
    ),
  );
});

test("the complete existing vendor guard retains all 130 upstream and 20 original consumer authorities", () => {
  const result = spawnSync(
    process.execPath,
    ["tools/support/dependencies/css-engine-vendor-custody.ts"],
    { encoding: "utf8" },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  const receipt = JSON.parse(result.stdout);
  assert.equal(receipt.upstreamFiles, 130);
  assert.equal(receipt.originalConsumerFiles, 20);
});
