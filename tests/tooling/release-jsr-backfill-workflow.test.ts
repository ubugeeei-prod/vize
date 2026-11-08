import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { parse } from "yaml";

interface Step {
  readonly name?: string;
  readonly if?: string;
  readonly run?: string;
  readonly env?: Readonly<Record<string, string>>;
}

interface Workflow {
  readonly on?: {
    readonly workflow_dispatch?: { readonly inputs?: Readonly<Record<string, unknown>> };
  };
  readonly jobs: Readonly<Record<string, { readonly steps?: readonly Step[] }>>;
}

const jsr = parse(
  readFileSync(new URL("../../.github/workflows/release-jsr.yml", import.meta.url), "utf8"),
) as Workflow;

await test("manual JSR backfills verify npm before either publication path", () => {
  assert.deepEqual(jsr.on?.workflow_dispatch?.inputs?.backfill_existing_npm_release, {
    description: "Publish facades for an older exact npm release after registry verification",
    required: false,
    default: false,
    type: "boolean",
  });
  for (const jobName of ["publish", "publish-packages"]) {
    const steps = jsr.jobs[jobName]?.steps ?? [];
    const normal = steps.find(
      (step) => step.name === "Generate the package for the published npm release",
    );
    const backfill = steps.find(
      (step) => step.name === "Generate a registry-verified npm release backfill",
    );
    assert.equal(normal?.if, "inputs.backfill_existing_npm_release != true");
    assert.equal(backfill?.if, "inputs.backfill_existing_npm_release == true");
    if (jobName === "publish") {
      assert.match(
        backfill?.run ?? "",
        /package\.ts prepare-backfill "\$RUNNER_TEMP\/jsr-package" "\$JSR_VERSION"$/,
      );
    } else {
      assert.match(
        backfill?.run ?? "",
        /package\.ts prepare-backfill-one "\$RUNNER_TEMP\/jsr-package" "\$JSR_PACKAGE" "\$JSR_VERSION"$/,
      );
      assert.equal(backfill?.env?.JSR_PACKAGE, "${{ matrix.jsrName }}");
    }
    assert.equal(backfill?.env?.JSR_VERSION, "${{ inputs.version }}");
  }
});
