import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { openVueApp } from "../_helpers/ui-library-apps.ts";
import { OPENVUE_SOURCE } from "../_helpers/openvue-source.ts";
import { planAppE2eRows } from "../../tools/support/compat/github/app-e2e-plan.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixturePath = "tests/_fixtures/_git/openvue";

test("OpenVue source, license, globs, and App E2E selectors stay pinned together", () => {
  assert.deepEqual(OPENVUE_SOURCE, {
    schema: "vize.realWorldFixtureSource",
    version: 1,
    id: "openvue",
    repository: "https://github.com/openvi-foundation/openvue",
    release: "1.0.0",
    revision: "116be30dbadb1b03a26ce080aa1aae2ba70e8972",
    discussion: "https://github.com/ubugeeei-prod/vize/discussions/8552",
    license: {
      spdx: "MIT",
      path: "LICENSE.md",
      gitBlob: "4c4480ef4e5e7ad1a8810a23059d1537211772a2",
    },
    verifiedVueFiles: {
      "packages/primevue/src/**/*.vue": 281,
      "packages/icons/src/**/*.vue": 48,
      "packages/forms/src/**/*.vue": 4,
      "apps/showcase/**/*.vue": 1712,
      "apps/volt/**/*.vue": 628,
    },
  });
  assert.equal(
    execFileSync("git", ["config", "-f", ".gitmodules", "--get", `submodule.${fixturePath}.url`], {
      cwd: root,
      encoding: "utf8",
    }).trim(),
    OPENVUE_SOURCE.repository,
  );
  assert.equal(
    execFileSync("git", ["ls-files", "--stage", "--", fixturePath], {
      cwd: root,
      encoding: "utf8",
    }).trim(),
    `160000 ${OPENVUE_SOURCE.revision} 0\t${fixturePath}`,
  );
  assert.deepEqual(openVueApp.check?.patterns, Object.keys(OPENVUE_SOURCE.verifiedVueFiles));
  assert.deepEqual(openVueApp.lint?.patterns, Object.keys(OPENVUE_SOURCE.verifiedVueFiles));
  assert.equal(openVueApp.check?.tsconfig, "packages/primevue/tsconfig.json");
  assert.equal(
    Object.values(OPENVUE_SOURCE.verifiedVueFiles).reduce((sum, count) => sum + count),
    2673,
  );

  for (const suite of ["check", "lint"]) {
    assert(
      planAppE2eRows("full", suite).every((row) => row.fixtures.includes(fixturePath)),
      `${suite} must hydrate OpenVue`,
    );
  }
  const rustPlan = fs.readFileSync(
    path.join(root, "tools/commands/ci/github/app-e2e-plan.rs"),
    "utf8",
  );
  assert.match(
    rustPlan,
    /const CHECK_FIXTURES:[\s\S]*?"nuxt-ui",\s*"openvue",\s*"primevue",/,
    "the Rust planner used by Actions must hydrate OpenVue",
  );
  const scripts = JSON.parse(fs.readFileSync(path.join(root, "tests/package.json"), "utf8"))
    .scripts as Record<string, string>;
  assert.match(scripts["test:check"]!, /snapshots\/check\/openvue\.ts/);
  assert.match(scripts["test:lint"]!, /snapshots\/lint\/openvue\.ts/);
  assert.match(
    fs.readFileSync(path.join(root, ".github/actions/hydrate-app-fixtures/action.yml"), "utf8"),
    /tests\/_fixtures\/_git\/openvue/,
  );
});
