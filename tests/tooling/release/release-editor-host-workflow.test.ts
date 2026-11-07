import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

type Step = { uses?: string; with?: Record<string, unknown> };
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
  const serialized = JSON.stringify(job.steps);
  assert.match(serialized, /"name":"cli-x86_64-unknown-linux-gnu"/);
  assert.match(serialized, /"name":"editor-extensions"/);
  assert.match(serialized, /"name":"npm-packages","path":"npm"/);
  assert.match(serialized, /vize-x86_64-unknown-linux-gnu\.tar\.gz/);
  assert.match(serialized, /vize \$RELEASE_VERSION/);
  assert.match(serialized, /for editor in zed nvim vim helix emacs/);
  assert.match(serialized, /target\/source-editors\/\$editor/);
  assert.match(serialized, /\$\{editor\}-vize-extension\.tar\.gz/);
  assert.match(serialized, /node \\"\$init_cli\\" init .*--typecheck --editor --no-install/);
  assert.match(serialized, /npm\/cli\/dist\/cli\.mjs/);
  assert.match(serialized, /project-links\.txt/);
  assert.match(serialized, /workspace-links\.txt/);
  assert.match(serialized, /vue\/require-v-for-key/);
  const host = job.steps?.find((step) => step.uses === "./.github/actions/vscode-host-smoke");
  assert.equal(host?.with?.["prebuilt-server"], "${{ steps.artifacts.outputs.server }}");
  assert.equal(host?.with?.["prebuilt-vsix"], "${{ steps.artifacts.outputs.vsix }}");
  assert.equal(host?.with?.["expected-version"], "${{ needs.version.outputs.version }}");
});

test("release editor hosts install the supplied CLI and VSIX without rebuilding them", () => {
  assert.match(editorHostAction, /prebuilt-server:/);
  assert.match(editorHostAction, /prebuilt-vsix:/);
  assert.match(
    editorHostAction,
    /if: inputs\.prebuilt-server == ''[\s\S]*cargo build --profile ci -p vize/,
  );
  assert.match(editorHostAction, /install -m 0755 "\$PREBUILT_SERVER" target\/ci\/vize/);
  assert.match(editorHostAction, /target\/ci\/vize --version[\s\S]*vize \$EXPECTED_VERSION/);
  assert.match(
    editorHostAction,
    /install -m 0644 "\$PREBUILT_VSIX" editors\/vscode\/dist\/vize\.vsix/,
  );
  assert.match(editorHostAction, /vp node editors\/vscode\/test\/run-extension-host-real\.mjs/);
});
