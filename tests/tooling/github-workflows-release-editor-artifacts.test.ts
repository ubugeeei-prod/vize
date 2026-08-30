import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("release workflow builds every editor-extension artifact it uploads", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const job = workflowJobBody(workflow, "build-editor-extensions");

  // `actions/upload-artifact` with `if-no-files-found: error` only fails when
  // no path matches, so a tarball nobody packages can be silently dropped.
  const uploadPaths = job
    .slice(job.indexOf("path: |"))
    .split("\n")
    .slice(1)
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.includes(":"));

  assert.deepEqual(uploadPaths, [
    "emacs-vize-extension.tar.gz",
    "helix-vize-extension.tar.gz",
    "editors/vscode/dist/vize.vsix",
    "nvim-vize-extension.tar.gz",
    "vim-vize-extension.tar.gz",
    "zed-vize-extension.tar.gz",
  ]);

  const producingTaskFor = (uploadPath: string): string => {
    if (uploadPath === "editors/vscode/dist/vize.vsix") return "package:vscode-extension";
    const editor = uploadPath.replace(/-vize-extension\.tar\.gz$/, "");
    return `package:${editor}-extension`;
  };

  const unbuilt = uploadPaths.filter(
    (uploadPath) => !job.includes(`vp run --workspace-root ${producingTaskFor(uploadPath)}`),
  );
  assert.deepEqual(unbuilt, [], "every uploaded editor artifact needs a packaging step");
});
