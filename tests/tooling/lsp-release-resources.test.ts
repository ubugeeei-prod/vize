import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

test("the shipping-profile supplement preserves the scheduled ci-opt campaign", () => {
  const main = parse(readRepoFile(".github", "workflows", "davinci-resource-budgets.yml"));
  const release = parse(readRepoFile(".github", "workflows", "lsp-release-resources.yml"));
  assert.equal(main.on.workflow_dispatch.inputs.include_release.default, false);
  assert.equal(
    main.jobs["release-resource"].if,
    "github.event_name == 'workflow_dispatch' && inputs.include_release",
  );
  assert.equal(main.jobs["release-resource"].uses, "./.github/workflows/lsp-release-resources.yml");
  const steps = release.jobs.release.steps as Array<{ name?: string; run?: string; uses?: string }>;
  const command = (name: string) => steps.find((step) => step.name === name)?.run ?? "";
  assert.match(
    command("Build the shipped Linux CLI recipe"),
    /cargo build --release -p vize --target x86_64-unknown-linux-gnu/u,
  );
  assert.equal(
    steps.some((step) => step.uses?.startsWith("wild-linker/")),
    false,
  );
  const campaign = command("Measure the complete TS-44 campaign with the shipped profile");
  assert.match(campaign, /--runs 3 --keystrokes 20 --idle-seconds 60/u);
  assert.match(campaign, /--server target\/x86_64-unknown-linux-gnu\/release\/vize/u);
  assert.match(
    command("Apply every unchanged TS-44 ceiling to this additional build"),
    /--check --measurement/u,
  );
  const bulk = command("Measure 10000 open buffers, 16 production providers, and Corsa");
  assert.match(bulk, /--files 10000 --warm-providers 16 --runs 1 --idle-seconds 10/u);
  assert.match(bulk, /--build "release: fat LTO, one codegen unit"/u);
});
