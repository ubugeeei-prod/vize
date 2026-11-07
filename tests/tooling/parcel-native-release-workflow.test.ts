import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const workflow = fs.readFileSync(
  new URL("../../.github/workflows/parcel-transformer.yml", import.meta.url),
  "utf8",
);

test("tagged releases publish every native Parcel package through OIDC", () => {
  const parsed = parse(workflow) as {
    on: { push: { tags: string[] }; pull_request: unknown };
    jobs: Record<
      string,
      {
        permissions: Record<string, string>;
        uses: string;
        with: Record<string, string>;
      }
    >;
  };
  const job = parsed.jobs["native-plugin-packages"];

  assert.deepEqual(parsed.on.push.tags, ["v*"]);
  assert.ok(parsed.on.pull_request);
  assert.equal(job.permissions["id-token"], "write");
  assert.equal(
    job.uses,
    "parcel-bundler/parcel/.github/workflows/native-plugin.yml@2bcfb43e725b2688b787e497e0b0b4105c353222",
  );
  assert.equal(job.with.paths, "npm/builder/parcel-native");
  assert.equal(
    job.with.publish,
    "${{ github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v') }}",
  );
});
