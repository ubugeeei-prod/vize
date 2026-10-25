import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { targets } from "../../../tools/support/editors/typescript-provider/artifacts.ts";
import { nativeReleasePlatforms } from "../../../tools/support/github/release-platforms.ts";
import {
  editorArtifactsForResume,
  editorPlatformAssets,
  editorPlatformMatrix,
  editorPublishMatrix,
} from "../../../tools/support/release/editor-platforms.ts";
import { readRepoFile } from "../support/github-workflows.ts";

test("all eight release providers match the packager and have distinct immutable artifacts", () => {
  const matrix = editorPlatformMatrix();
  assert.equal(matrix.length, 8);
  assert.deepEqual(
    matrix.map(({ rustTarget }) => rustTarget).sort(),
    nativeReleasePlatforms.map(({ target }) => target).sort(),
  );
  assert.deepEqual(
    Object.fromEntries(matrix.map(({ rustTarget, target }) => [rustTarget, target])),
    Object.fromEntries(Object.entries(targets).map(([rust, value]) => [rust, value.platform])),
  );
  for (const key of ["target", "archive", "artifact"] as const)
    assert.equal(new Set(matrix.map((row) => row[key])).size, 8);
  assert.equal(editorPlatformAssets().length, 16);
  const publish = editorPublishMatrix();
  assert.equal(publish.length, 9);
  assert.equal(publish[0].target, "universal");
  assert.equal(publish[0].sourcePath, "editors/vscode/dist/vize.vsix");
  assert.deepEqual(publish.slice(1), matrix);
});

test("tag authority keeps legacy resume separate and refuses missing or foreign target cohorts", () => {
  const workflow = "jobs:\n  build-editor-platforms:\n    needs: [version, build-native]\n";
  const names = editorPlatformMatrix().map(({ artifact }) => artifact);
  assert.equal(editorArtifactsForResume("jobs:\n", ["editor-extensions"]), false);
  assert.equal(editorArtifactsForResume(workflow, ["editor-extensions", ...names]), true);
  for (const invalid of [
    [],
    names.slice(1),
    [...names, names[0]],
    [...names, "editor-extension-platform-foreign"],
  ])
    assert.throws(() => editorArtifactsForResume(workflow, invalid), /inventory/);
  assert.throws(() => editorArtifactsForResume("jobs:\n", names), /inventory/);
});

test("the tag waits for all provider packages and every publisher selects its own target", () => {
  const release = parse(readRepoFile(".github", "workflows", "release.yml"));
  const job = release.jobs["build-editor-platforms"];
  assert.deepEqual(job.needs, ["version", "build-native"]);
  assert.equal(job.strategy["fail-fast"], false);
  assert.equal(
    job.strategy.matrix.settings,
    "${{ fromJSON(needs.version.outputs.editor_matrix) }}",
  );
  const download = job.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(download.with.name, "bindings-${{ matrix.settings.rustTarget }}");
  const packaging = job.steps.find(
    (step: { name?: string }) => step.name === "Package the exact target provider",
  );
  assert.match(packaging.run, /package-typescript-provider\.rs/);
  assert.match(packaging.run, /GITHUB_WORKSPACE\/\.artifacts\/vscode/);
  assert.doesNotMatch(packaging.run, /cargo|build-napi/);
  const upload = job.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.equal(upload.with.path, ".artifacts/vscode/*");
  assert.ok(release.jobs.tag.needs.includes("build-editor-platforms"));
  for (const key of ["publish-vscode", "publish-open-vsx"]) {
    const publisher = release.jobs[key];
    assert.equal(publisher.strategy["fail-fast"], false);
    assert.equal(
      publisher.strategy.matrix.settings,
      "${{ fromJSON(needs.version.outputs.editor_publish_matrix) }}",
    );
    assert.equal(publisher.env.VIZE_EXTENSION_TARGET, "${{ matrix.settings.target }}");
    assert.equal(publisher.env.VSIX_SOURCE_PATH, "${{ matrix.settings.sourcePath }}");
    assert.match(publisher.steps.at(-1).run, /"editor-extensions\/\$VSIX_SOURCE_PATH"/);
  }
  const github = JSON.stringify(release.jobs["github-release"]);
  assert.match(github, /editor-extension-platform-\*/);
  assert.match(github, /platform\/\*\.receipt\.json/);
  assert.match(github, /VIZE_EDITOR_PLATFORM_ARTIFACTS/);
});
