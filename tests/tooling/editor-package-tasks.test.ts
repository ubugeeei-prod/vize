import assert from "node:assert/strict";
import { test } from "node:test";
import { taskCatalog } from "../../tools/config/vite-plus/task-groups.ts";
import { workspaceTaskSteps } from "../../tools/config/vite-plus/task-plan.ts";
import { readRepoFile } from "./support/github-workflows.ts";

test("CI packages each editor archive and runs its package guard", () => {
  assert.match(
    readRepoFile(".github", "actions", "package-editor-extensions", "action.yml"),
    /package:editor-extensions/,
  );
  const all = workspaceTaskSteps(taskCatalog["package:editor-extensions"]);
  for (const [editor, guard] of [
    ["zed", "zed"],
    ["nvim", "neovim"],
    ["vim", "vim"],
    ["helix", "helix"],
    ["emacs", "emacs"],
  ] as const) {
    const name = `package:${editor}-extension` as const;
    const archive = editor + "-vize-extension.tar.gz";
    const [tar, verify] = workspaceTaskSteps(taskCatalog[name]);
    assert.deepEqual(tar, {
      program: "tar",
      arguments: [
        ...(editor === "zed" ? ["--exclude", "zed/target"] : []),
        "-czf",
        archive,
        "-C",
        "editors",
        editor,
      ],
      environment: { COPYFILE_DISABLE: "1", LC_ALL: "C", LANG: "C" },
    });
    assert.deepEqual(verify, {
      program: "node",
      arguments: [`tools/commands/editors/${guard}/assert-${editor}-package.ts`, archive],
    });
    const expected = { program: "vp", arguments: ["run", "--workspace-root", name] };
    assert.ok(all.some((step) => step.program === "vp" && step.arguments?.at(-1) === name));
    assert.deepEqual(workspaceTaskSteps(taskCatalog[`test:${editor}-extension:package`]), [
      expected,
    ]);
  }
});

test("VS Code package and host checks use the extension's local tools", () => {
  for (const name of [
    "package:vscode-extension",
    "package:editor-extensions",
    "test:vscode-extension:vsix",
    "test:vscode-extension:host-real",
  ] as const) {
    const commands = workspaceTaskSteps(taskCatalog[name]);
    assert.deepEqual(commands[0], {
      program: "node",
      arguments: ["tools/commands/editors/vscode/ensure-dependencies.ts"],
    });
    assert.ok(
      commands.some(
        (step) =>
          step.directory === "editors/vscode" &&
          step.program === "rust-script" &&
          step.arguments?.includes("../../tools/commands/editors/vscode/assert-vsix-package.rs"),
      ),
    );
  }
  assert.deepEqual(workspaceTaskSteps(taskCatalog["test:vscode-extension:host"]).at(-1), {
    program: "pnpm",
    arguments: ["run", "test:host"],
    directory: "editors/vscode",
  });
  assert.equal(workspaceTaskSteps(taskCatalog["test:zed-extension:unit"])[0]?.program, "cargo");
  assert.equal(workspaceTaskSteps(taskCatalog["test:nvim-extension:headless"])[0]?.program, "nvim");
  assert.equal(workspaceTaskSteps(taskCatalog["test:vim-extension:headless"])[0]?.program, "vim");
});
