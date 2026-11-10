import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

type Step = {
  env?: Record<string, string>;
  id?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type Job = {
  env?: Record<string, string>;
  outputs?: Record<string, string>;
  steps?: Step[];
};

const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};
const verify = parse(readRepoFile(".github", "workflows", "release-verify.yml")) as {
  jobs: Record<string, Job>;
};

test("release derives prerelease metadata from the stamped version", () => {
  const version = release.jobs.version;
  assert.equal(version.outputs?.npm_tag, "${{ steps.channel.outputs.npm_tag }}");
  assert.equal(version.outputs?.prerelease, "${{ steps.channel.outputs.prerelease }}");
  assert.equal(version.outputs?.make_latest, "${{ steps.channel.outputs.make_latest }}");

  const channel = version.steps?.find((step) => step.id === "channel");
  assert.equal(
    channel?.env?.RELEASE_VERSION,
    "${{ steps.resume.outputs.version || steps.cut.outputs.version }}",
  );
  assert.match(String(channel?.run), /tools\/commands\/release\/npm\/tag\.ts/u);
  assert.match(String(channel?.run), /npm_tag=\$npm_tag/u);
  assert.match(String(channel?.run), /prerelease=(?:true|false)/u);
  assert.match(String(channel?.run), /make_latest=(?:true|false)/u);
});

test("release verification accepts stable and prerelease versions", () => {
  const steps = (verify.jobs["verify-npm"].steps ?? []).map((step) => step.run ?? "").join("\n");
  const versionPattern = /\[\[ "\$RELEASE_VERSION" =~ (\^.*\$) \]\]/u.exec(steps)?.[1];
  assert.ok(versionPattern);

  const releaseVersion = new RegExp(versionPattern);
  for (const version of ["1.0.0", "1.0.0-alpha", "1.0.0-alpha.1", "2.3.4-rc.12"])
    assert.match(version, releaseVersion);
  for (const version of ["v1.0.0", "1.0", "1.0.0-", "1.0.0-alpha..1"])
    assert.doesNotMatch(version, releaseVersion);
});

test("the prerelease channel reaches editors and GitHub Release", () => {
  for (const name of [
    "publish-vscode",
    "publish-open-vsx",
    "publish-vscode-platforms",
    "publish-open-vsx-platforms",
  ]) {
    assert.equal(release.jobs[name].env?.NPM_TAG, "${{ needs.version.outputs.npm_tag }}", name);
  }

  const create = release.jobs["github-release"].steps?.find((step) =>
    step.uses?.startsWith("softprops/action-gh-release@"),
  );
  assert.equal(create?.with?.prerelease, "${{ needs.version.outputs.prerelease }}");
  assert.equal(create?.with?.make_latest, "${{ needs.version.outputs.make_latest }}");
});
