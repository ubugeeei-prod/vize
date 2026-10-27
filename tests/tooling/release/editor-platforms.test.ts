import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { targets } from "../../../tools/support/editors/typescript-provider/artifacts.ts";
import { nativeReleasePlatforms } from "../../../tools/support/github/release-platforms.ts";
import {
  editorArtifactsForResume,
  editorPlatformAssets,
  editorPlatformMatrix,
} from "../../../tools/support/release/editor-platforms.ts";
import { installVscodeExtensionDependencies } from "../../../tools/config/vite-plus/task-commands.ts";
import { buildTasks } from "../../../tools/config/vite-plus/tasks/build.ts";
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
  for (const row of matrix) assert.equal(row.sourcePath, row.archive);
});

test("resume signs only a complete platform cohort and refuses foreign artifacts", () => {
  const workflow = "jobs:\n  build-editor-platforms:\n    needs: [version, build-native]\n";
  const names = editorPlatformMatrix().map(({ artifact }) => artifact);
  assert.equal(editorArtifactsForResume("jobs:\n", ["editor-extensions"]), false);
  assert.equal(editorArtifactsForResume(workflow, ["editor-extensions", ...names]), true);
  // A failed platform build must not stop the npm resume.
  assert.equal(editorArtifactsForResume(workflow, []), false);
  assert.equal(editorArtifactsForResume(workflow, names.slice(1)), false);
  for (const invalid of [
    [...names, names[0]],
    [...names, "editor-extension-platform-foreign"],
  ])
    assert.throws(() => editorArtifactsForResume(workflow, invalid), /inventory/);
  assert.throws(() => editorArtifactsForResume("jobs:\n", names.slice(0, 1)), /inventory/);
});

test("platform providers publish after their packages without gating the release", () => {
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
  for (const key of ["tag", "publish-npm", "publish-vscode", "publish-open-vsx"])
    assert.ok(!release.jobs[key].needs.includes("build-editor-platforms"), key);
  for (const key of ["publish-vscode", "publish-open-vsx"])
    assert.match(release.jobs[key].steps.at(-1).run, /editors\/vscode\/dist\/vize\.vsix$/);
  for (const key of ["publish-vscode-platforms", "publish-open-vsx-platforms"]) {
    const publisher = release.jobs[key];
    assert.deepEqual(publisher.needs, ["version", "tag", "build-editor-platforms"]);
    assert.equal(publisher.strategy["fail-fast"], false);
    assert.equal(
      publisher.strategy.matrix.settings,
      "${{ fromJSON(needs.version.outputs.editor_matrix) }}",
    );
    assert.equal(publisher.env.VIZE_EXTENSION_TARGET, "${{ matrix.settings.target }}");
    assert.equal(publisher.env.VSIX_SOURCE_PATH, "${{ matrix.settings.sourcePath }}");
    assert.match(publisher.steps.at(-1).run, /"editor-extensions\/\$VSIX_SOURCE_PATH"/);
  }
  const githubRelease = release.jobs["github-release"];
  assert.ok(githubRelease.needs.includes("build-editor-platforms"));
  assert.match(githubRelease.if, /always\(\)/);
  assert.doesNotMatch(githubRelease.if, /build-editor-platforms/);
  assert.match(
    githubRelease.env.EDITOR_PLATFORMS,
    /needs\.build-editor-platforms\.result == 'success'/,
  );
  const github = JSON.stringify(githubRelease);
  assert.match(github, /editor-extension-platform-\*/);
  assert.match(github, /platform\/\*\.receipt\.json/);
  assert.match(github, /VIZE_EDITOR_PLATFORM_ARTIFACTS/);
});

test("platform providers install the extension's packager like the universal VSIX", () => {
  const release = parse(readRepoFile(".github", "workflows", "release.yml"));
  type Step = { name?: string; uses?: string; run?: string };
  const steps = (job: string): Step[] => release.jobs[job].steps;
  const index = (job: string, match: (step: Step) => boolean) => steps(job).findIndex(match);
  const install = (step: Step) => step.run === "vp run --workspace-root package:vscode-extension";
  const rustScript = (step: Step) => step.uses === "./.github/actions/setup-rust-script";
  const packaging = (step: Step) => step.name === "Package the exact target provider";
  // vsce and the prepublish `vp pack` resolve from editors/vscode/node_modules.
  assert.ok(index("build-editor-platforms", rustScript) < index("build-editor-platforms", install));
  assert.ok(index("build-editor-platforms", install) < index("build-editor-platforms", packaging));
  assert.match(
    steps("build-editor-extensions").find((step) => step.name === "Package editor extensions")
      ?.run ?? "",
    /^vp run --workspace-root package:vscode-extension$/m,
  );
  // Both jobs share the setup before their extension step.
  const setup = (job: string, until: (step: Step) => boolean) =>
    steps(job)
      .slice(0, index(job, until))
      .map((step) => step.uses ?? step.run);
  assert.deepEqual(
    setup("build-editor-platforms", install),
    setup("build-editor-extensions", (step) => step.name === "Package editor extensions"),
  );
  const command = (task: unknown) => {
    assert.ok(task !== null && typeof task === "object" && "command" in task);
    assert.equal(typeof task.command, "string");
    return String(task.command);
  };
  assert.ok(
    command(buildTasks["package:vscode-extension"]).startsWith(
      `${installVscodeExtensionDependencies} && `,
    ),
  );
});

test("editor publishers outwait registry validation and Open VSX maintenance", () => {
  const release = parse(readRepoFile(".github", "workflows", "release.yml"));
  const waitSeconds = (job: { env: Record<string, string> }) =>
    Number(job.env.PUBLISH_RESOLUTION_RETRY_LIMIT) * 10;
  // The Marketplace listed the 0.584.0 platform uploads 6.5-9 min after vsce exited.
  const vscode = release.jobs["publish-vscode-platforms"];
  assert.equal(waitSeconds(vscode), 20 * 60);
  assert.ok(vscode["timeout-minutes"] * 60 >= waitSeconds(vscode) + 5 * 60);
  // Read-only retries wait 30 + 60 + 120 + 240 s before the scan wait starts.
  for (const key of ["publish-open-vsx", "publish-open-vsx-platforms"]) {
    const job = release.jobs[key];
    // Release 0.598.0 still had missing targets over twelve minutes after upload.
    assert.equal(waitSeconds(job), 20 * 60, key);
    assert.ok(job["timeout-minutes"] * 60 >= 450 + waitSeconds(job) + 5 * 60, key);
  }
});
