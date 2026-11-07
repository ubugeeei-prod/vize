import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

type Step = {
  id?: string;
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};
type Job = {
  needs?: string[];
  if?: string;
  "timeout-minutes"?: number;
  steps?: Step[];
};

const workflow = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};
const editorHostAction = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

test("release editor hosts consume the packed candidate before the tag exists", () => {
  const job = workflow.jobs["smoke-editor-artifacts"]!;
  assert.deepEqual(job.needs, [
    "version",
    "build-cli",
    "build-packages",
    "build-editor-extensions",
  ]);
  assert.equal(job.if, "needs.version.outputs.ref != ''");
  assert.ok((job["timeout-minutes"] ?? 0) >= 45);
  const setupNode =
    job.steps?.findIndex((step) => step.uses?.startsWith("actions/setup-node@")) ?? -1;
  const installArtifacts = job.steps?.findIndex((step) => step.id === "artifacts") ?? -1;
  assert.ok(setupNode >= 0 && setupNode < installArtifacts);
  assert.deepEqual(job.steps?.[setupNode]?.with, {
    "node-version-file": "package.json",
    "package-manager-cache": false,
  });
  const serialized = JSON.stringify(job.steps);
  assert.match(serialized, /"name":"cli-x86_64-unknown-linux-gnu"/);
  assert.match(serialized, /"name":"editor-extensions"/);
  assert.match(serialized, /"name":"npm-packages"/);
  assert.match(serialized, /vize-x86_64-unknown-linux-gnu\.tar\.gz/);
  assert.match(serialized, /vize \$RELEASE_VERSION/);
  assert.match(serialized, /for editor in zed nvim vim helix emacs/);
  assert.match(serialized, /target\/source-editors\/\$editor/);
  assert.match(serialized, /\$\{editor\}-vize-extension\.tar\.gz/);
  const install = job.steps?.find((step) => step.id === "artifacts")?.run ?? "";
  assert.match(install, /node "\$packaged_cli" init [\s\S]*--typecheck --editor --no-install/);
  assert.match(serialized, /VIZE_TS45_EXISTING_WORKSPACE/);
  for (const client of ["neovim", "helix", "vscode"]) {
    assert.match(serialized, new RegExp(`tests/editor-conformance/run\\.ts ${client}`));
  }
  assert.match(serialized, /project-links\.txt/);
  assert.match(serialized, /workspace-links\.txt/);
  assert.match(serialized, /vue\/require-v-for-key/);
  const host = job.steps?.find((step) => step.uses === "./.github/actions/vscode-host-smoke");
  assert.equal(host?.with?.["prebuilt-server"], "${{ steps.artifacts.outputs.server }}");
  assert.equal(host?.with?.["prebuilt-vsix"], "${{ steps.artifacts.outputs.vsix }}");
  assert.equal(host?.with?.["expected-version"], "${{ needs.version.outputs.version }}");
  assert.equal(host?.with?.["release-workspace"], "${{ steps.artifacts.outputs.fresh }}");
  assert.equal(host?.with?.["release-evidence"], "${{ steps.artifacts.outputs.evidence }}");
});

test("release editor hosts install the supplied CLI and VSIX without rebuilding them", () => {
  const action = parse(editorHostAction) as {
    inputs: Record<string, unknown>;
    runs: { steps: Step[] };
  };
  assert.ok(action.inputs["prebuilt-server"]);
  assert.ok(action.inputs["prebuilt-vsix"]);
  const build = action.runs.steps.find((step) =>
    step.run?.includes("cargo build --profile ci -p vize"),
  );
  assert.equal(build?.if, "inputs.prebuilt-server == ''");
  assert.match(editorHostAction, /install -m 0755 "\$PREBUILT_SERVER" target\/ci\/vize/);
  assert.match(editorHostAction, /target\/ci\/vize --version[\s\S]*vize \$EXPECTED_VERSION/);
  assert.match(
    editorHostAction,
    /install -m 0644 "\$PREBUILT_VSIX" editors\/vscode\/dist\/vize\.vsix/,
  );
  assert.match(editorHostAction, /vp node editors\/vscode\/test\/run-extension-host-real\.mjs/);
  assert.match(editorHostAction, /zed-linux-x86_64\.tar\.gz/);
  assert.match(editorHostAction, /node tests\/editor-conformance\/release-zed\.ts/);
  const zed = action.runs.steps.find(
    (step) => step.name === "Run actual Zed with the packaged release extension",
  );
  assert.equal(zed?.if, "inputs.release-workspace != ''");
});
