import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { readRouterEditorFixture } from "../../editors/vscode/test/router-host-workspace.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixturePath = "tests/fixtures/editor/experimental-router-5.4";

test("installed editor fixture pins complete published scanner declarations and both authored page languages", () => {
  const { authority, files } = readRouterEditorFixture(root);
  assert.equal(authority.issue, 10578);
  assert.deepEqual(authority.provider, {
    name: "vue-router",
    version: "5.4.0",
    archiveSha256: "0c3a6d93f5236da11d0d2496fa4244e56a2518d8a2c5cb93a81e6956e117dbab",
  });
  assert.deepEqual(
    [...files.keys()],
    [
      "src/pages/products/[id=int].vue",
      "src/pages/art-products/[id=int].art.vue",
      "src/pages/art-variants/[id=int].art.vue",
      "typed-router.d.ts",
    ],
  );
  const declaration = files.get("typed-router.d.ts")!.toString("utf8");
  for (const name of [...files.keys()].filter((file) => file.endsWith(".vue"))) {
    assert.ok(declaration.includes(`'${name}': {`));
    assert.match(files.get(name)!.toString("utf8"), /\/\/ 雪😀\n/u);
  }
});

test("editor authority rejects changed complete declaration bytes and changed parent provider", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-editor-custody-"));
  try {
    const fixture = path.join(directory, fixturePath);
    fs.cpSync(path.join(root, fixturePath), fixture, { recursive: true });
    const parent = path.join(directory, "tests/fixtures/typechecker/experimental-router-5.4");
    fs.mkdirSync(parent, { recursive: true });
    const parentFile = path.join(parent, "provider.json");
    fs.copyFileSync(
      path.join(root, "tests/fixtures/typechecker/experimental-router-5.4/provider.json"),
      parentFile,
    );
    readRouterEditorFixture(directory);
    fs.appendFileSync(path.join(fixture, "typed-router.d.ts.txt"), "\n");
    assert.throws(() => readRouterEditorFixture(directory));
    fs.copyFileSync(
      path.join(root, fixturePath, "typed-router.d.ts.txt"),
      path.join(fixture, "typed-router.d.ts.txt"),
    );
    fs.appendFileSync(parentFile, "\n");
    assert.throws(() => readRouterEditorFixture(directory));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("artifact workflow executes the installed VSIX with same-source receipt custody and always retains failures", () => {
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/router-editor-artifact.yml"),
    "utf8",
  );
  assert.match(workflow, /uses: \.\/\.github\/workflows\/full-source-cli\.yml/u);
  assert.match(workflow, /uses: \.\/\.github\/actions\/restore-source-cli/u);
  const restoreAt = workflow.indexOf("uses: ./.github/actions/restore-source-cli");
  for (const prerequisite of [
    "uses: dtolnay/rust-toolchain@",
    "uses: wild-linker/action@",
    "git rev-parse 'HEAD^{tree}'",
  ]) {
    assert.ok(
      workflow.indexOf(prerequisite) >= 0 && workflow.indexOf(prerequisite) < restoreAt,
      `${prerequisite} must precede receipt authentication`,
    );
  }
  const packageAt = workflow.indexOf("test:vscode-extension:vsix");
  const runAt = workflow.indexOf("xvfb-run -a node editors/vscode/test/run-router-host-real.mjs");
  assert.ok(packageAt >= 0 && runAt > packageAt);
  assert.match(workflow.slice(runAt), /if: \$\{\{ always\(\) \}\}/u);
  assert.match(workflow.slice(runAt), /if-no-files-found: error/u);
  const runner = fs.readFileSync(
    path.join(root, "editors/vscode/test/run-router-host-real.mjs"),
    "utf8",
  );
  assert.match(runner, /runPackagedExtensionHost\(runVSCodeCommand/u);
  assert.match(runner, /consumed\.bundleSha256, producer\.packagedBundleSha256/u);
  assert.match(runner, /processes\.spawns\[0\]\.binarySha256, producer\.native\.sha256/u);
  assert.match(runner, /finally[\s\S]*host-logs[\s\S]*fs\.rmSync\(directory/u);
});

test("local completion ownership uses the installed client after the real watcher and keeps the original Art campaign", () => {
  const suite = fs.readFileSync(
    path.join(root, "editors/vscode/test/suite/router-config-real.cjs"),
    "utf8",
  );
  const local = fs.readFileSync(
    path.join(root, "editors/vscode/test/suite/router-local-resolve.cjs"),
    "utf8",
  );
  assert.ok(
    suite.indexOf("await localResolution.afterWatch(on, after)") >
      suite.indexOf("filesystem-only tsconfig watcher delivery"),
  );
  assert.match(local, /vscode\.commands\.executeCommand\(command, \.\.\.args\)/u);
  assert.match(local, /"vscode\.executeCompletionItemProvider"/u);
  assert.match(local, /"vize\.test\.executeLspRequest"/u);
  assert.equal(local.includes("workspace/didChangeWatchedFiles"), false);
  assert.equal(local.includes("LspSession"), false);
  assert.match(local, /stale completion does not reach the native compiler/u);
  assert.match(suite, /assert\.deepEqual\(failures, \[\]\)/u);
});
