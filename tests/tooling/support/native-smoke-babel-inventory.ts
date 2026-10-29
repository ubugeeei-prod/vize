import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const directory = new URL(
  "../../_fixtures/differential/workflows/native-smoke-babel-inventory-10702/",
  import.meta.url,
);
const registration = {
  schemaVersion: 1,
  scope:
    "Only append the unpublished Babel package to the otherwise identical native-smoke inventory; no runtime or publication credit",
  workflow: ".github/workflows/native-smoke.yml",
  issue: 10702,
  beforeSource: "280a6b22ad9d525eac109cf4c6cebe079def2174",
  afterSource: "556f65f4bd7034305f4930693053be0e790da1fa",
  qualification: {
    runId: 40603306405,
    attempt: 1,
    artifactId: 12779736895,
    artifactDigest: "sha256:ca52ff0891c0e851a1ae140b2e2a300279474e85a4c5c81444c2f05f7b7b9d3f",
    scope:
      "Separate Babel source host/runtime/style qualification, not an eighteen-row native-smoke execution",
  },
  originalProvenanceSha256: "89a0cdf0739297cbe3f5ed596fc1c81d2d8b1ada3eb2647e36b74427117a1ad1",
  rawFiles: {
    "before.yml": {
      gitBlob: "5f2eab031746c94f45eb2ed37a65426bd4b8ca48",
      sha256: "259c6c57c8e9c4609d5d7b3df292f6aaaf570d0f88b67f84e22d29a3f7bce935",
    },
    "after.yml": {
      gitBlob: "a5246a5eeafe52f6dfc3ccba68f207d332559596",
      sha256: "cb090990a287cd13949c5e38ed22d3a2619b3b64c7415b6593f171c02c02574a",
    },
  },
  edit: {
    before:
      "  RELEASE_JS_PACKAGES: npm/cli npm/builder/vite npm/oxlint npm/builder/unplugin npm/fresco npm/mcp-musea npm/builder/vite-musea npm/builder/rspack npm/framework/musea-nuxt npm/framework/nuxt-lint-config npm/framework/nuxt npm/marquette npm/compose/core npm/ui npm/plugin-sdk\n",
    after:
      "  RELEASE_JS_PACKAGES: npm/cli npm/builder/vite npm/oxlint npm/builder/unplugin npm/fresco npm/mcp-musea npm/builder/vite-musea npm/builder/rspack npm/framework/musea-nuxt npm/framework/nuxt-lint-config npm/framework/nuxt npm/marquette npm/compose/core npm/ui npm/plugin-sdk npm/babel\n",
    occurrences: 1,
    oldPackages: 15,
    newPackages: 16,
    defaultRuntimeRows: 18,
  },
} as const;

function gitBlob(source: string): string {
  const bytes = Buffer.from(source);
  return createHash("sha1")
    .update(Buffer.concat([Buffer.from("blob " + bytes.length + "\0"), bytes]))
    .digest("hex");
}

/** Authenticate both complete documents and exactly one additive inventory edit. */
export function verifyBabelSmokeInventoryPair(
  before: string,
  after: string,
  admission: unknown,
): void {
  assert.deepEqual(admission, registration, "the complete source registration remains frozen");
  for (const [name, source] of [
    ["before.yml", before],
    ["after.yml", after],
  ] as const) {
    assert.equal(gitBlob(source), registration.rawFiles[name].gitBlob, name);
    assert.equal(
      createHash("sha256").update(source).digest("hex"),
      registration.rawFiles[name].sha256,
      name,
    );
  }
  assert.equal(before.split(registration.edit.before).length, 2, "one exact inventory line");
  assert.equal(after, before.replace(registration.edit.before, registration.edit.after));
}

/** Restore only the registered complete successor for the original provenance guard. */
export function admitBabelSmokeInventory(source: string): string {
  const before = fs.readFileSync(new URL("before.yml", directory), "utf8");
  const after = fs.readFileSync(new URL("after.yml", directory), "utf8");
  const admission: unknown = JSON.parse(
    fs.readFileSync(new URL("admission.json", directory), "utf8"),
  );
  verifyBabelSmokeInventoryPair(before, after, admission);
  assert.ok(source === before || source === after, "only the two complete authenticated documents");
  return before;
}
