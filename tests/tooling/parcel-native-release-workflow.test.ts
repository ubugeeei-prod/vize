import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";
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
      schedule: { cron: string }[];
      workflow_dispatch: {
        inputs: Record<string, { type: string; default: string | boolean }>;
      };
    };
    jobs: Record<
      string,
      {
        permissions: Record<string, string>;
        outputs?: Record<string, string>;
        steps: Array<{
          name?: string;
          uses?: string;
          run?: string;
          with?: Record<string, unknown>;
        }>;
        needs?: string | string[];
        if?: string;
      }
    >;
  };
  const matrix = parsed.jobs["native-plugin-matrix"]!;
  const publish = parsed.jobs["native-plugin-publish"]!;

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
  // Only the publish job can mint the npm trusted-publishing token.
  for (const [id, job] of Object.entries(parsed.jobs)) {
    assert.equal(
      job.permissions?.["id-token"],
      id === "native-plugin-publish" ? "write" : undefined,
    );
  }
  assert.deepEqual(publish.needs, ["native-plugin-matrix", "native-plugin-pack"]);
  assert.equal(
    publish.if,
    "${{ !cancelled() && needs.native-plugin-pack.result == 'success' && needs.native-plugin-matrix.outputs.publish == 'true' }}",
  );
  assert.equal(
    matrix.outputs?.publish,
    "${{ github.repository == 'ubugeeei-prod/vize' && ((github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')) || (github.event_name == 'workflow_dispatch' && inputs.publish == true && (github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')))) }}",
  );
  // Packaging runs Parcel's scripts from the commit the reusable workflow was pinned to.
  const scripts = matrix.steps.find((step) => step.name === "Check out packaging scripts");
  assert.equal(scripts?.with?.repository, "parcel-bundler/parcel");
  assert.equal(scripts?.with?.ref, "2bcfb43e725b2688b787e497e0b0b4105c353222");
  assert.match(
    publish.steps.find((step) => step.name === "Update npm")?.run ?? "",
    /^npm install -g npm@\d+\.\d+\.\d+$/u,
  );
});

test("every Parcel native job runs only commit-pinned actions", () => {
  const parsed = parse(workflow) as {
    jobs: Record<string, { uses?: string; steps?: Array<{ uses?: string }> }>;
  };
  for (const [id, job] of Object.entries(parsed.jobs)) {
    assert.equal(job.uses, undefined, `${id} calls a reusable workflow`);
    for (const step of job.steps ?? []) {
      if (step.uses === undefined || step.uses.startsWith("./")) continue;
      assert.match(step.uses, /^[\w.-]+\/[\w./-]+@[0-9a-f]{40}$/u, `${id}: ${step.uses}`);
    }
  }
});

test("native Parcel packages publish only from this repository's v* tags or a dispatch on main", () => {
  const parsed = parse(workflow) as { jobs: Record<string, { outputs: Record<string, string> }> };
  const condition = parsed.jobs["native-plugin-matrix"].outputs.publish.slice(3, -2).trim();
  const startsWith = (value: string, prefix: string) => value.startsWith(prefix);
  const publishes = (
    event: string,
    ref: string,
    publish: boolean | null,
    repository = "ubugeeei-prod/vize",
  ) =>
    new Script(condition).runInNewContext({
      github: { repository, event_name: event, ref },
      inputs: { publish },
      startsWith,
    });
  const tag = "refs/tags/v0.800.0";
  // release.yml dispatches `--ref "$RELEASE_TAG" -f publish=true` once the tag exists.
  assert.equal(publishes("workflow_dispatch", tag, true), true);
  assert.equal(publishes("workflow_dispatch", "refs/heads/main", true), true);
  assert.equal(publishes("push", tag, null), true);
  // Builds without a publish request, and requests from any other ref, publish nothing.
  assert.equal(publishes("workflow_dispatch", tag, false), false);
  assert.equal(publishes("workflow_dispatch", "refs/heads/main", false), false);
  assert.equal(publishes("workflow_dispatch", "refs/heads/feature", true), false);
  assert.equal(publishes("workflow_dispatch", "refs/heads/v0.800.0", true), false);
  assert.equal(publishes("workflow_dispatch", "refs/tags/nightly", true), false);
  assert.equal(publishes("schedule", "refs/heads/main", null), false);
  assert.equal(publishes("pull_request", "refs/pull/1/merge", null), false);
  assert.equal(publishes("push", "refs/tags/nightly", null), false);
  // Another repository never publishes, whatever its event.
  assert.equal(publishes("workflow_dispatch", tag, true, "someone/vize"), false);
  assert.equal(publishes("workflow_dispatch", "refs/heads/main", true, "someone/vize"), false);
  assert.equal(publishes("push", tag, null, "someone/vize"), false);
});

test("pull requests build per-platform packages only for native plugin changes", () => {
  const parsed = parse(workflow) as {
    on: { schedule: { cron: string }[] };
    jobs: Record<
      string,
      {
        if?: string;
        needs?: string;
        "runs-on"?: string;
        outputs?: Record<string, string>;
        steps?: Array<{ id?: string; uses?: string; with?: { filters?: string } }>;
      }
    >;
  };
  // Hosted macOS arm64 runners are capacity-limited; main still builds nightly.
  assert.deepEqual(parsed.on.schedule, [{ cron: "37 2 * * *" }]);
  const plan = parsed.jobs["packaging-changes"]!;
  assert.equal(plan.if, "${{ github.event_name == 'pull_request' }}");
  assert.equal(plan["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.deepEqual(plan.outputs, { packages: "${{ steps.changes.outputs.packages }}" });
  const [detect] = plan.steps ?? [];
  assert.equal(detect?.uses, "dorny/paths-filter@7b450fff21473bca461d4b92ce414b9d0420d706");
  assert.equal(
    detect?.with?.filters,
    "packages:\n  - 'npm/builder/parcel-native/**'\n  - 'crates/vize_parcel_transformer/**'\n  - '.github/workflows/parcel-transformer.yml'\n",
  );
  const packages = parsed.jobs["native-plugin-matrix"]!;
  assert.equal(packages.needs, "packaging-changes");
  assert.equal(
    packages.if,
    "${{ !cancelled() && (github.event_name != 'pull_request' || needs.packaging-changes.outputs.packages == 'true') }}",
  );
  // The Linux consumer still builds the native transformer on every change.
  assert.equal(parsed.jobs["native-consumer"]?.if, undefined);
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
