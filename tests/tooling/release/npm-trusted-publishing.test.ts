import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const releaseSource = fs.readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");

interface Step {
  id?: string;
  run?: string;
}

interface Workflow {
  jobs: Record<
    string,
    {
      environment?: string;
      permissions?: Record<string, string>;
      "runs-on"?: string;
      steps?: Step[];
    }
  >;
}

test("initial and resumed npm publication use only the trusted OIDC identity", () => {
  const workflow = YAML.parse(releaseSource) as Workflow;
  const publishJob = workflow.jobs["publish-npm"];
  const publish = publishJob?.steps?.find((step) => step.id === "publish");

  assert.equal(publishJob?.["runs-on"], "ubuntu-24.04");
  assert.equal(publishJob?.environment, "npm");
  assert.deepEqual(publishJob?.permissions, {
    actions: "read",
    contents: "read",
    "id-token": "write",
  });
  assert.match(publish?.run ?? "", /--provenance/u);
  assert.doesNotMatch(
    JSON.stringify(publishJob),
    /secrets\.|NPM_TOKEN|NODE_AUTH_TOKEN|_authToken/u,
  );
  assert.doesNotMatch(releaseSource, /npm-bootstrap-auth|NPM_BOOTSTRAP_TOKEN/u);
  assert.equal(
    fs.existsSync(path.join(root, ".github/actions/npm-bootstrap-auth/action.yml")),
    false,
  );
});
