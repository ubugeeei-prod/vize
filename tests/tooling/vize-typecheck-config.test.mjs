import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { vizeTypecheckConfig } from "../../tools/benchmarks/scripts/vize-typecheck-config.mjs";

function withProject(run) {
  const root = mkdtempSync(join(tmpdir(), "vize-adapter-options-"));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function write(root, name, value) {
  writeFileSync(join(root, name), JSON.stringify(value));
}
function translated(root, config = "tsconfig.json") {
  return JSON.parse(readFileSync(join(root, vizeTypecheckConfig(root, config)), "utf8"));
}

await test("standard strict and explicit boolean options translate without changing original JSON", () => {
  for (const [options, expected] of [
    [
      { strictTemplates: true },
      { checkUnknownProps: true, fallthroughAttributes: false, strictComponentAttrs: true },
    ],
    [{}, { checkUnknownProps: false, fallthroughAttributes: false, strictComponentAttrs: false }],
    [
      { strictTemplates: false },
      { checkUnknownProps: false, fallthroughAttributes: false, strictComponentAttrs: false },
    ],
    [
      { strictTemplates: true, checkUnknownProps: false },
      { checkUnknownProps: false, fallthroughAttributes: false, strictComponentAttrs: true },
    ],
    [
      { strictTemplates: true, fallthroughAttributes: true },
      { checkUnknownProps: true, fallthroughAttributes: true, strictComponentAttrs: true },
    ],
    [
      { strictTemplates: true, strictComponentAttrs: false },
      { checkUnknownProps: true, fallthroughAttributes: false, strictComponentAttrs: false },
    ],
    [
      { strictTemplates: false, checkUnknownProps: true, strictComponentAttrs: true },
      { checkUnknownProps: true, fallthroughAttributes: false, strictComponentAttrs: true },
    ],
  ])
    withProject((root) => {
      write(root, "tsconfig.json", {
        compilerOptions: { strict: true },
        vueCompilerOptions: options,
        files: ["App.vue"],
      });
      const before = readFileSync(join(root, "tsconfig.json"));
      assert.deepEqual(translated(root), {
        extends: "./tsconfig.json",
        vueCompilerOptions: expected,
      });
      assert.deepEqual(readFileSync(join(root, "tsconfig.json")), before);
    });
});

await test("later extends entries and then local explicit false win in a nested generated config", () =>
  withProject((root) => {
    write(root, "early.json", {
      vueCompilerOptions: {
        strictTemplates: true,
        checkUnknownProps: true,
        fallthroughAttributes: true,
      },
    });
    write(root, "later.json", {
      vueCompilerOptions: { checkUnknownProps: false, strictComponentAttrs: false },
    });
    mkdirSync(join(root, "nested"));
    write(root, "nested/tsconfig.json", {
      extends: ["../early.json", "../later.json"],
      vueCompilerOptions: { fallthroughAttributes: false },
    });
    const originals = ["early.json", "later.json", "nested/tsconfig.json"].map((name) =>
      readFileSync(join(root, name)),
    );
    assert.deepEqual(translated(root, "nested/tsconfig.json"), {
      extends: "./tsconfig.json",
      vueCompilerOptions: {
        checkUnknownProps: false,
        fallthroughAttributes: false,
        strictComponentAttrs: false,
      },
    });
    for (const [index, name] of ["early.json", "later.json", "nested/tsconfig.json"].entries())
      assert.deepEqual(readFileSync(join(root, name)), originals[index]);
  }));

await test("missing or cyclic configs fail before invoking a checker", () =>
  withProject((root) => {
    assert.throws(() => vizeTypecheckConfig(root), /ENOENT/);
    write(root, "tsconfig.json", { extends: "./parent.json" });
    write(root, "parent.json", { extends: "./tsconfig.json" });
    assert.throws(() => vizeTypecheckConfig(root), /cyclic benchmark config/);
  }));
