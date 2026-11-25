import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { collectVueLikeFilesByTarget, standaloneHtmlTargets } from "./files.ts";
import { prepareHtmlContexts } from "./html-operation.ts";
import { joinHtmlTargets } from "./html-project-output.ts";

const ownedPlugin = fileURLToPath(new URL("../../dist/index.mjs", import.meta.url));

function project(): { root: string; entrypoint: string; candidates: Set<string> } {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-html-targets-")));
  const files = {
    "src/index.html": '<div id="app"></div>\n',
    "src/App.vue": "<template><div /></template>\n",
    "src/nested/Nested.html": "<p></p>\n",
    "scripts/build.ts": "export const ready = true;\n",
    "pages/legacy.htm": "<p></p>\n",
    "node_modules/oxlint/bin/oxlint": "",
    "node_modules/oxlint/package.json": '{"name":"oxlint","version":"1.86.0"}\n',
    ".oxlintrc.json": `${JSON.stringify({ jsPlugins: [ownedPlugin] })}\n`,
  };
  for (const [name, source] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    fs.writeFileSync(path.join(root, name), source);
  }
  return {
    root,
    entrypoint: path.join(root, "node_modules/oxlint/bin/oxlint"),
    candidates: new Set(),
  };
}

void test("only targets that hold standalone HTML get an HTML operation", () => {
  const { root, candidates } = project();
  try {
    const targets = ["src", "scripts", "pages/legacy.htm", "src/App.vue"];
    const filesByTarget = collectVueLikeFilesByTarget(root, targets, (file) =>
      candidates.add(file),
    );
    assert.deepEqual(standaloneHtmlTargets(targets, filesByTarget), ["src", "pages/legacy.htm"]);
    assert.deepEqual(standaloneHtmlTargets(["scripts"], [filesByTarget[1]]), []);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("several HTML targets share one host setup and run one operation each", () => {
  const { root, entrypoint, candidates } = project();
  try {
    const targets = ["src/nested", "pages", "src"];
    // A missing target beside them selects nothing, as in Oxlint, and is no failure.
    collectVueLikeFilesByTarget(root, [...targets, "missing"], (file) => candidates.add(file));
    const args = ["-c", ".oxlintrc.json", "-f", "unix", "--deny-warnings", ...targets, "missing"];
    const contexts = prepareHtmlContexts(root, args, entrypoint, "1.86.0", candidates, targets);

    assert.deepEqual(
      contexts.map((context) => context.options?.literalTarget),
      ["pages", "src", "src/nested"],
      "enclosing targets run before the targets nested inside them",
    );
    for (const context of contexts) {
      assert.equal(context.failure, undefined);
      assert.equal(context.skipReason, undefined);
      assert.equal(context.custody, contexts[0].custody);
      assert.equal(context.provider, contexts[0].provider);
      assert.deepEqual(
        { ...context.options, literalTarget: undefined },
        { ...contexts[0].options, literalTarget: undefined },
      );
    }
    const [single] = prepareHtmlContexts(root, args, entrypoint, "1.86.0", candidates, ["src"]);
    assert.deepEqual(single.options, contexts[1].options);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("a host-wide HTML failure is reported once for every target", () => {
  const original = { stdout: "host report\n", stderr: "", status: 0 };
  const failed = joinHtmlTargets(
    original,
    [{ environment: {}, failure: "scoped preparation failed" }],
    [],
  );
  assert.equal(failed.stdout, original.stdout);
  assert.equal(failed.status, 1);
  assert.equal(failed.stderr, "\nOriginal HTML unavailable: scoped preparation failed\n");

  const skipped = joinHtmlTargets(
    original,
    [{ environment: {}, skipReason: "unqualified host" }],
    [],
  );
  assert.equal(skipped.status, 0);
  assert.equal(
    skipped.stderr,
    "\n[oxlint-plugin-vize] Skipped standalone HTML lint: unqualified host\n",
  );
});
