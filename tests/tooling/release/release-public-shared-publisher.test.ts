import assert from "node:assert/strict";
import { test } from "node:test";
import {
  derivePublicationPlan,
  readRawBlob,
} from "../../../tools/support/release/public_acceptance/plan.ts";
import { readRepoFile } from "../support/github-workflows.ts";
import { sourceFixture } from "../support/release-public-acceptance-fixtures.ts";

const actionPath = ".github/actions/publish-release-package/action.yml";
const invocation = `      - uses: ./.github/actions/publish-release-package
        with:
          artifact: release-package-musea-mcp-server
          package-path: npm/mcp-musea`;
const oldCommand =
  "      - run: node tools/support/release/publish_npm_package/main.ts npm/mcp-musea --provenance";

test("shared publication retains all source targets and binds the immutable action blob", (t) => {
  const source = sourceFixture(t);
  source.git("reset", "--soft", source.cut);
  source.write(
    ".github/workflows/release.yml",
    readRawBlob(source.root, source.head, ".github/workflows/release.yml")
      .toString()
      .replace(oldCommand, invocation),
  );
  source.write(actionPath, readRepoFile(actionPath));
  source.git("add", ".");
  source.git("commit", "-qm", "shared source publisher H");
  const head = source.git("rev-parse", "HEAD");
  const plan = derivePublicationPlan(source.root, head);
  assert.deepEqual(plan.npm, source.plan.npm);
  const action = plan.authority.blobs.find(({ path }) => path === actionPath);
  assert.equal(action?.oid, source.git("rev-parse", `${head}:${actionPath}`));
  source.write(actionPath, "working tree is not source authority");
  assert.deepEqual(derivePublicationPlan(source.root, head), plan);
});

test("shared publication refuses altered commands, dynamic paths and extra inputs", (t) => {
  const source = sourceFixture(t);
  const workflow = readRawBlob(
    source.root,
    source.head,
    ".github/workflows/release.yml",
  ).toString();
  const recipe = readRepoFile(actionPath);
  const hostile: Array<[string, string]> = [
    [invocation.replace("npm/mcp-musea", "npm/../mcp-musea"), recipe],
    [invocation.replace("npm/mcp-musea", "${{ env.OTHER_PACKAGE }}"), recipe],
    [invocation + "\n          ignore-failure: true", recipe],
    [invocation + "\n        continue-on-error: true", recipe],
    [invocation + "\n\n          ignore-failure: true", recipe],
    [invocation, recipe.replace("--provenance", "--provenance && true")],
    [invocation, recipe.replace('"$RELEASE_PACKAGE_PATH"', "npm/other")],
    [invocation, recipe + "    - run: echo ignored\n      shell: bash\n"],
    [invocation, recipe.replace("inputs.package-path", "inputs.artifact")],
  ];
  for (const [call, action] of hostile) {
    source.git("reset", "--soft", source.cut);
    source.write(".github/workflows/release.yml", workflow.replace(oldCommand, call));
    source.write(actionPath, action);
    source.git("add", ".");
    source.git("commit", "-qm", "unreviewed shared publisher H");
    assert.throws(
      () => derivePublicationPlan(source.root, source.git("rev-parse", "HEAD")),
      /unsupported shared npm publish/,
    );
  }
});
