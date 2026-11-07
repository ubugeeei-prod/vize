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
    on: {
      push: { tags: string[] };
      pull_request: unknown;
      workflow_dispatch: {
        inputs: Record<string, { type: string; default: string | boolean }>;
      };
    };
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
  assert.deepEqual(parsed.on.workflow_dispatch.inputs.publish, {
    description: "Publish every native package through npm trusted publishing",
    type: "boolean",
    default: false,
  });
  assert.deepEqual(parsed.on.workflow_dispatch.inputs.version, {
    description: "Exact package version override for a manual release recovery",
    type: "string",
    default: "",
  });
  assert.equal(job.permissions["id-token"], "write");
  assert.equal(
    job.uses,
    "parcel-bundler/parcel/.github/workflows/native-plugin.yml@2bcfb43e725b2688b787e497e0b0b4105c353222",
  );
  assert.equal(job.with.paths, "npm/builder/parcel-native");
  assert.equal(
    job.with.publish,
    "${{ (github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')) || inputs.publish == true }}",
  );
  assert.equal(job.with.version, "${{ inputs.version }}");
});

test("the guarded release dispatches Parcel publication from the immutable tag", () => {
  const release = parse(
    fs.readFileSync(new URL("../../.github/workflows/release.yml", import.meta.url), "utf8"),
  ) as {
    jobs: Record<
      string,
      {
        permissions: Record<string, string>;
        steps: Array<{ name?: string; env?: Record<string, string>; run?: string }>;
      }
    >;
  };
  const tag = release.jobs.tag;
  const dispatch = tag.steps.find(
    (step) => step.name === "Dispatch Parcel native publication from the immutable tag",
  );

  assert.deepEqual(tag.permissions, { actions: "write", contents: "write" });
  assert.equal(dispatch?.env?.GH_TOKEN, "${{ github.token }}");
  assert.equal(
    dispatch?.run,
    'gh workflow run parcel-transformer.yml --ref "$RELEASE_TAG" -f publish=true',
  );
});
