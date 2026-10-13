import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { authenticateCssReleaseDriver } from "../../tools/support/dependencies/css-engine-release-driver-custody.ts";
import { repoRoot } from "./_helpers/moonbit.ts";

const filename = "tools/commands/release/pr.rs";
const originalHash = "db53a5ec6abf597c01dfb77b27477cfb10d69bdeba7018ab5920278d0a590d2d";
const current = fs.readFileSync(path.join(repoRoot, filename));
const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

test("the exact composed cache driver reverses to the retained whole producer", () => {
  authenticateCssReleaseDriver(filename, current, originalHash);
  const stable = current
    .toString()
    .replace("release-bot-envelope-variants-v16", "decorated-integration-body-custody-v15");
  authenticateCssReleaseDriver(filename, Buffer.from(stable), originalHash);
  const original = fs.readFileSync(
    path.join(
      repoRoot,
      "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/stable-driver/pr-original.rs.txt",
    ),
  );
  authenticateCssReleaseDriver(filename, original, originalHash);
  assert.equal(
    checksum(
      Buffer.from(
        stable.replace(
          '        ["start-stable", policy, cut] => pr_pin::start_stable(policy, cut, &root),\n',
          "",
        ),
      ),
    ),
    originalHash,
  );
});

test("cache driver custody rejects unreviewed bytes, reference pins and foreign paths", () => {
  for (const bytes of [
    Buffer.concat([current, Buffer.from("\n")]),
    Buffer.from(current.toString().replace("variants-v16", "variants-v17")),
    Buffer.from(current.toString().replace('["start-stable"', '["unreviewed"')),
  ]) {
    assert.throws(() => authenticateCssReleaseDriver(filename, bytes, originalHash));
  }
  assert.throws(() => authenticateCssReleaseDriver(filename, current, checksum(current)));
  assert.throws(() => authenticateCssReleaseDriver("unknown.rs", current, originalHash));
  assert.throws(() => authenticateCssReleaseDriver("__proto__", current, originalHash));
});
