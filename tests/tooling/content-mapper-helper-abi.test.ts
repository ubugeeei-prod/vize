import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import path from "node:path";
import fs from "node:fs";
import os from "node:os";
import ts from "typescript";
import { contentMapperHelperAbi } from "../../npm/cli/scripts/content-mapper-helper-abi.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = (file: string) => readFileSync(path.join(root, file), "utf8");

void test("the native ABI snapshot qualifies all conditional entries and private dependencies", () => {
  const [file, snapshot] = contentMapperHelperAbi(root, new Map());
  assert.equal(source(file), snapshot, "regenerate the native ABI with the SDK declarations");
});

for (const extension of ["ts", "mts", "cts"]) {
  void test(`a missing consumed export invalidates the .d.${extension} entry`, () => {
    const file = `npm/cli/virtual-ts.d.${extension}`;
    const changed = source(file).replace(
      "type $ComponentRawProps<",
      "type $MissingComponentRawProps<",
    );
    assert.notEqual(changed, source(file));
    assert.throws(
      () => contentMapperHelperAbi(root, new Map([[file, changed]])),
      /must export \$ComponentRawProps|invalid SDK declaration/u,
    );
  });
}

void test("private bindings participate in the same dependency contract", () => {
  const file = "npm/cli/virtual-ts-emits.d.ts";
  const changed = source(file).replace(
    "type $LooseEmitPropsFactory<",
    "type $MissingLooseEmitPropsFactory<",
  );
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /invalid SDK declaration/u,
  );
});

void test("core imports must be public through every helper frontend", () => {
  const file = "npm/cli/generated/content-mapper-types.d.ts";
  const changed = source(file).replace("  $ComponentInput,\n", "");
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /must export \$ComponentInput/u,
  );
});

void test("every relative import must have an installed declaration target", () => {
  const file = "npm/cli/virtual-ts.d.mts";
  const changed = source(file).replace("./virtual-ts-emits.js", "./absent-emit-dependency.js");
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /missing SDK dependency/u,
  );
});

void test("ambient implicit exports are qualified by TypeScript rather than modifier text", () => {
  const pending = new Map(
    ["ts", "mts", "cts"].map((extension) => {
      const file = `npm/cli/virtual-ts.d.${extension}`;
      const changed = source(file).replace(
        "export type $ComponentRawProps<",
        "type $ComponentRawProps<",
      );
      assert.notEqual(changed, source(file));
      return [file, changed];
    }),
  );
  assert.doesNotThrow(() => contentMapperHelperAbi(root, pending));
});

for (const [name, samePeer, preserveSymlinks] of [
  ["different physical peer", false, false],
  ["same physical peer", true, false],
  ["preserved logical peer", false, true],
] as const) {
  void test(`pnpm SDK resolution uses the ${name} Vue authority`, () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-abi-peer-"));
    try {
      const project = path.join(directory, "app");
      const store = path.join(directory, "store/vize/node_modules");
      const sdk = path.join(store, "vize");
      fs.mkdirSync(path.join(project, "node_modules"), { recursive: true });
      fs.mkdirSync(sdk, { recursive: true });
      const sdkSource = path.join(root, "npm/cli");
      const manifest = JSON.parse(source("npm/cli/package.json")) as { files: string[] };
      fs.copyFileSync(path.join(sdkSource, "package.json"), path.join(sdk, "package.json"));
      for (const file of manifest.files.filter((file) => /\.d\.(?:ts|mts|cts)$/u.test(file))) {
        fs.mkdirSync(path.dirname(path.join(sdk, file)), { recursive: true });
        fs.copyFileSync(path.join(sdkSource, file), path.join(sdk, file));
      }
      const link = (target: string, file: string) =>
        fs.symlinkSync(target, file, process.platform === "win32" ? "junction" : "dir");
      link(sdk, path.join(project, "node_modules/vize"));
      const projectVue = path.join(project, "node_modules/vue");
      const installVue = (directory: string) => {
        fs.mkdirSync(directory);
        fs.writeFileSync(
          path.join(directory, "package.json"),
          JSON.stringify({
            name: "vue",
            version: "3.5.44",
            types: "index.d.ts",
          }),
        );
        fs.writeFileSync(path.join(directory, "index.d.ts"), "export {};\n");
      };
      installVue(projectVue);
      if (samePeer) link(projectVue, path.join(store, "vue"));
      else installVue(path.join(store, "vue"));
      const options: ts.CompilerOptions = {
        module: ts.ModuleKind.Preserve,
        moduleResolution: ts.ModuleResolutionKind.Bundler,
        preserveSymlinks,
      };
      const resolve = (name: string, from: string) => {
        const resolved = ts.resolveModuleName(
          name,
          from,
          options,
          ts.sys,
          undefined,
          undefined,
          ts.ModuleKind.ESNext,
        ).resolvedModule;
        assert.ok(resolved, name);
        return resolved.resolvedFileName;
      };
      const helper = resolve("vize/virtual-ts", path.join(project, "input.ts"));
      const actual = fs.realpathSync(resolve("vue", helper));
      const authored = fs.realpathSync(resolve("vue", path.join(project, "input.ts")));
      assert.equal(actual === authored, samePeer || preserveSymlinks);
      assert.equal(
        actual,
        fs.realpathSync(
          path.join(
            samePeer || preserveSymlinks ? projectVue : path.join(store, "vue"),
            "index.d.ts",
          ),
        ),
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
