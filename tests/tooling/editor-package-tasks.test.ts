import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile } from "./support/github-workflows.ts";

test("CI packages editor extension artifacts through Rust Script wrappers", () => {
  const packageAction = readRepoFile(
    ".github",
    "actions",
    "package-editor-extensions",
    "action.yml",
  );
  const buildTasks = readRepoFile("config", "vite-plus", "tasks", "build.ts");
  const taskCommands = readRepoFile("config", "vite-plus", "task-commands.ts");
  const testTasks = readRepoFile("config", "vite-plus", "tasks", "test-benchmark.ts");
  const vscodePackage =
    /const assertVscodePackage = rustToolFromVscodeExtension\(\s*"editors\/vscode\/assert-vsix-package"/;

  assert.match(packageAction, /package:editor-extensions/);
  assert.match(
    taskCommands,
    /if \$\{vscodeExtensionPackageBin\("vite-plus", "vp"\)\} --version >\/dev\/null 2>&1[\s\S]*corepack pnpm install --ignore-workspace --lockfile-dir \. --no-lockfile --prefer-offline --ignore-scripts/,
  );
  for (const contents of [buildTasks, testTasks]) {
    assert.match(contents, vscodePackage);
  }
  assert.match(buildTasks, /package:vscode-extension[\s\S]*assertVscodePackage/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*assertVscodePackage/);
  assert.match(testTasks, /test:vscode-extension:vsix[\s\S]*assertVscodePackage/);
  assert.match(testTasks, /test:vscode-extension:host[\s\S]*pnpm run test:host/);
  assert.match(testTasks, /test:vscode-extension:host-real[\s\S]*run-extension-host-real\.mjs/);

  for (const [task, wrapper] of [
    ["zed", "editors/zed/assert-zed-package"],
    ["nvim", "editors/neovim/assert-nvim-package"],
    ["vim", "editors/vim/assert-vim-package"],
    ["helix", "editors/helix/assert-helix-package"],
    ["emacs", "editors/emacs/assert-emacs-package"],
  ] as const) {
    const escapedWrapper = wrapper.replaceAll("/", "\\/");
    assert.match(
      buildTasks,
      new RegExp(`package:${task}-extension[\\s\\S]*rustTool\\("${escapedWrapper}`),
    );
    assert.match(
      buildTasks,
      new RegExp(`package:editor-extensions[\\s\\S]*package:${task}-extension`),
    );
    assert.match(
      testTasks,
      new RegExp(`test:${task}-extension:package[\\s\\S]*package:${task}-extension`),
    );
  }

  assert.match(testTasks, /test:zed-extension:unit[\s\S]*cargo test/);
  assert.match(testTasks, /test:nvim-extension:headless[\s\S]*nvim --headless/);
  assert.match(testTasks, /test:vim-extension:headless[\s\S]*vim -Nu NONE/);
});
