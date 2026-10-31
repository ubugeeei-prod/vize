import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import {
  lstat,
  mkdtemp,
  mkdir,
  readFile,
  readdir,
  readlink,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";

import { importConfigModuleWithDependencies } from "../../crates/vize_carton/src/config/loader/module-runtime.mjs";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { nativeLoaderScript as nativeScript } from "./support/native-config-script.ts";

const require = createRequire(join(process.cwd(), "package.json"));
const vitePlus = dirname(require.resolve("vite-plus/package.json"));
const execute = promisify(execFile);
const reproduction = JSON.parse(
  await readFile(
    new URL(
      "../_fixtures/differential/typechecker/editor-vite-stub-11145/project.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { files: Record<string, string> };
const stubManifest = reproduction.files["node_modules/vite/package.json"];
const stubTypes = reproduction.files["node_modules/vite/client.d.ts"];

async function inventory(directory: string): Promise<unknown[]> {
  const result: unknown[] = [];
  for (const name of (await readdir(directory)).sort()) {
    const path = join(directory, name);
    const metadata = await lstat(path);
    result.push(
      metadata.isSymbolicLink()
        ? [name, "link", await readlink(path)]
        : metadata.isDirectory()
          ? [name, "directory", await inventory(path)]
          : [name, "file", await readFile(path, "utf8")],
    );
  }
  return result;
}

async function project(run: (root: string, stub: string) => Promise<void>, withVitePlus = true) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-types-only-11145-")));
  try {
    const stub = join(root, "node_modules/vite");
    await mkdir(stub, { recursive: true });
    await writeFile(join(root, "package.json"), '{"private":true,"type":"module"}');
    await writeFile(join(stub, "package.json"), stubManifest);
    await writeFile(join(stub, "client.d.ts"), stubTypes);
    if (withVitePlus) await symlink(vitePlus, join(root, "node_modules/vite-plus"), "junction");
    await run(root, stub);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("declaration-only Vite preserves the installation and uses real project Vite+ in both loaders", async () => {
  await project(async (root, stub) => {
    const selected = join(root, "vite.config.ts");
    const counter = join(root, "evaluations.txt");
    const helper = "export const width: number = 93;\n";
    const authored = `import {width} from './settings';
      import {appendFileSync} from 'node:fs';
      appendFileSync(${JSON.stringify(counter)}, 'module\\n');
      export default () => {
        appendFileSync(${JSON.stringify(counter)}, 'source\\n');
        return {vue:{formatter:{printWidth:width}}};
      };`;
    await writeFile(join(root, "settings.ts"), helper);
    await writeFile(selected, authored);
    const original = await inventory(join(root, "node_modules"));
    const phases: string[][] = [];
    const loaded = await importConfigModuleWithDependencies(
      selected,
      undefined,
      undefined,
      (namespace) => resolveViteConfigExport(namespace.default, undefined, root),
      undefined,
      async (paths) => {
        phases.push([...paths]);
      },
    );
    assert.deepEqual(loaded.value, {
      typeChecker: { jsxTypecheck: true },
      formatter: { printWidth: 93 },
      __vizeProjectRoot: root,
    });
    assert.equal(await readFile(counter, "utf8"), "module\nsource\n");
    assert.ok(phases.some((paths) => paths.includes(join(stub, "package.json"))));
    assert.ok(phases.some((paths) => paths.includes(join(stub, "index.js"))));
    const output = await execute(
      process.execPath,
      ["--input-type=module", "-e", await nativeScript(), selected],
      { cwd: root, timeout: 10_000 },
    );
    assert.equal(output.stderr, "");
    const native = JSON.parse(output.stdout);
    assert.deepEqual(native.config, loaded.value);
    assert.equal(native.watchDependenciesComplete, true);
    for (const path of [
      stub,
      join(stub, "package.json"),
      join(stub, "index.js"),
      join(stub, "index.json"),
      join(stub, "index.node"),
    ])
      assert.ok(native.watchDependencies.includes(path), path);
    assert.equal(await readFile(counter, "utf8"), "module\nsource\nmodule\nsource\n");
    assert.deepEqual(await inventory(join(root, "node_modules")), original);
    assert.equal(await readFile(selected, "utf8"), authored);
    assert.equal(await readFile(join(root, "settings.ts"), "utf8"), helper);
  });
});

test("a declaration-only package without either runtime retains direct native module loading", async () => {
  await project(async (root) => {
    const selected = join(root, "vite.config.mjs");
    await writeFile(selected, "export default {vue:{formatter:{printWidth:94}}};");
    const output = await execute(
      process.execPath,
      ["--input-type=module", "-e", await nativeScript(), selected],
      { cwd: root, timeout: 10_000 },
    );
    assert.equal(output.stderr, "");
    assert.deepEqual(JSON.parse(output.stdout).config, {
      typeChecker: { jsxTypecheck: true },
      formatter: { printWidth: 94 },
      __vizeProjectRoot: root,
    });
  }, false);
});

test("runtime entry declarations and implicit index files still own loading", async () => {
  for (const form of ["main", "exports", "implicit", "broken-main"]) {
    await project(async (root, stub) => {
      const selected = join(root, "vite.config.ts");
      await writeFile(selected, "throw new Error('authored config must not run');");
      if (form === "main" || form === "exports")
        await writeFile(
          join(stub, "package.json"),
          JSON.stringify({ name: "vite", types: "client.d.ts", [form]: "./index.js" }),
        );
      if (form === "broken-main")
        await writeFile(
          join(stub, "package.json"),
          '{"name":"vite","types":"client.d.ts","main":"./missing.js"}',
        );
      else
        await writeFile(
          join(stub, "index.js"),
          "exports.loadConfigFromFile = () => { throw new Error('selected runtime refuses'); };\n",
        );
      await assert.rejects(
        importConfigModuleWithDependencies(selected),
        form === "broken-main"
          ? /Cannot find module|invalid.*main|escapes selected/
          : /selected runtime refuses/,
      );
    });
  }
});
