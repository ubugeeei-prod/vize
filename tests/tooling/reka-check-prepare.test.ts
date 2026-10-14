import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import { createRequire } from "node:module";
import * as os from "node:os";
import * as path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

import { inspectPrepareNode } from "../_helpers/nuxt-check-runtime.ts";
import { git } from "../_helpers/nuxt-check-source.ts";
import { prepareRekaCheckFixture } from "../_helpers/reka-check-prepare.ts";
import { inspectRekaNpx, inspectRekaProviders } from "../_helpers/reka-check-providers.ts";

const require = createRequire(import.meta.url);
const configPackage = fileURLToPath(
  new URL("../_fixtures/reka-tsconfig-provider", import.meta.url),
);
const typeScriptPackage = path.dirname(require.resolve("typescript/package.json"));
const vuePackage = path.dirname(require.resolve("vue/package.json"));
const vueVersion = JSON.parse(
  fs.readFileSync(path.join(vuePackage, "package.json"), "utf8"),
).version;

// Real complete published config bytes and the runner's actual TS/Vue packages.
// This tests provider custody/config parsing, never substitutes a native verdict.
function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reka-provider-law-"));
  const cwd = path.join(root, "tests/_fixtures/_git/reka-ui");
  const core = path.join(cwd, "packages/core");
  fs.mkdirSync(path.join(core, "src"), { recursive: true });
  fs.writeFileSync(path.join(cwd, ".gitignore"), "node_modules/\n.vize-check-prepare-lock\n");
  fs.writeFileSync(
    path.join(cwd, "package.json"),
    JSON.stringify({
      packageManager: "pnpm@10.13.1",
      scripts: { prepare: "pnpm simple-git-hooks" },
      devDependencies: { typescript: ts.version },
    }),
  );
  fs.writeFileSync(
    path.join(core, "package.json"),
    JSON.stringify({
      devDependencies: { "@vue/tsconfig": "^0.7.0", vue: vueVersion },
    }),
  );
  fs.writeFileSync(
    path.join(cwd, "pnpm-lock.yaml"),
    `lockfileVersion: '9.0'
importers:
  .:
    devDependencies:
      typescript: {specifier: '${ts.version}', version: '${ts.version}'}
  packages/core:
    devDependencies:
      '@vue/tsconfig': {specifier: '^0.7.0', version: '0.7.0(typescript@${ts.version})'}
      vue: {specifier: '${vueVersion}', version: '${vueVersion}(typescript@${ts.version})'}
`,
  );
  fs.writeFileSync(
    path.join(core, "tsconfig.app.json"),
    JSON.stringify({
      extends: "@vue/tsconfig/tsconfig.dom.json",
      include: ["src/**/*.ts"],
    }),
  );
  fs.writeFileSync(
    path.join(core, "src/control.ts"),
    "export const control: string = 'authored';\n",
  );
  git(cwd, ["init", "-q"]);
  git(cwd, ["add", "."]);
  git(cwd, [
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.com",
    "commit",
    "-qm",
    "fixture",
  ]);
  const pin = git(cwd, ["rev-parse", "HEAD"]).trim();
  git(root, ["init", "-q"]);
  git(root, ["update-index", "--add", "--cacheinfo", `160000,${pin},tests/_fixtures/_git/reka-ui`]);
  git(root, [
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.com",
    "commit",
    "-qm",
    "fixture authority",
  ]);
  fs.mkdirSync(path.join(cwd, "node_modules/.pnpm"), { recursive: true });
  fs.writeFileSync(path.join(cwd, "node_modules/.modules.yaml"), "unit custody test\n");
  fs.copyFileSync(path.join(cwd, "pnpm-lock.yaml"), path.join(cwd, "node_modules/.pnpm/lock.yaml"));
  fs.symlinkSync(typeScriptPackage, path.join(cwd, "node_modules/typescript"), "junction");
  fs.mkdirSync(path.join(core, "node_modules/@vue"), { recursive: true });
  fs.cpSync(configPackage, path.join(core, "node_modules/@vue/tsconfig"), { recursive: true });
  fs.symlinkSync(vuePackage, path.join(core, "node_modules/vue"), "junction");
  return { root, cwd, core, pin, cleanup: () => fs.rmSync(root, { recursive: true, force: true }) };
}

test("real complete Vue config inheritance and locked nearest providers remain required", () => {
  const f = fixture();
  try {
    const files: string[] = [];
    const result = inspectRekaProviders(f.cwd, (file) => files.push(file));
    assert.equal(result.parserVersion, ts.version);
    assert.equal(result.options.strict, true);
    assert.equal(result.options.noImplicitThis, true);
    assert.equal(result.options.moduleResolution, ts.ModuleResolutionKind.Bundler);
    assert.deepEqual(
      result.providers.map((provider) => provider.name),
      ["typescript", "@vue/tsconfig", "vue"],
    );
    assert.ok(files.includes(path.join(f.core, "node_modules/@vue/tsconfig/tsconfig.dom.json")));
    // An available ancestor cannot mask the missing authored core provider.
    fs.mkdirSync(path.join(f.cwd, "node_modules/@vue"), { recursive: true });
    fs.renameSync(
      path.join(f.core, "node_modules/@vue/tsconfig"),
      path.join(f.cwd, "node_modules/@vue/tsconfig"),
    );
    assert.throws(() => inspectRekaProviders(f.cwd, () => {}), /missing owned @vue\/tsconfig/);
  } finally {
    f.cleanup();
  }
});

test("wrong locked versions and nearer TypeScript shadows refuse config preparation", () => {
  const f = fixture();
  try {
    const manifest = path.join(f.core, "node_modules/@vue/tsconfig/package.json");
    const original = fs.readFileSync(manifest);
    const changed = JSON.parse(original.toString());
    changed.version = "0.8.0";
    fs.writeFileSync(manifest, JSON.stringify(changed));
    assert.throws(() => inspectRekaProviders(f.cwd, () => {}), /wrong locked @vue\/tsconfig/);
    fs.writeFileSync(manifest, original);
    const base = path.join(f.core, "node_modules/@vue/tsconfig/tsconfig.json");
    const baseBytes = fs.readFileSync(base);
    fs.appendFileSync(base, "\n// forged provider bytes\n");
    assert.throws(
      () => inspectRekaProviders(f.cwd, () => {}),
      /complete pinned published provider/,
    );
    fs.writeFileSync(base, baseBytes);
    fs.mkdirSync(path.join(f.core, "node_modules/typescript"));
    fs.writeFileSync(
      path.join(f.core, "node_modules/typescript/package.json"),
      JSON.stringify({
        name: "typescript",
        version: ts.version,
        main: "index.js",
      }),
    );
    fs.writeFileSync(
      path.join(f.core, "node_modules/typescript/index.js"),
      "throw new Error('must never execute');\n",
    );
    assert.throws(() => inspectRekaProviders(f.cwd, () => {}), /parser was shadowed/);
  } finally {
    f.cleanup();
  }
});

test("provider preparation retains exact inputs and restores/refuses install source mutations", () => {
  const f = fixture();
  try {
    let installs = 0;
    const options = {
      sourceDir: f.cwd,
      authorityRoot: f.root,
      expectedPin: f.pin,
      run: (_node: string, _npm: string, args: string[]) => {
        installs++;
        assert.deepEqual(args, [
          "-y",
          "pnpm@10.13.1",
          "install",
          "--frozen-lockfile",
          "--ignore-scripts",
          "--prefer-offline",
        ]);
      },
    };
    const receiptDir = prepareRekaCheckFixture(options);
    const success = JSON.parse(fs.readFileSync(path.join(receiptDir, "receipt.json"), "utf8"));
    assert.equal(success.failed, false);
    assert.equal(success.authoredPreserved, true);
    assert.equal(success.providers.options.strict, true);
    for (const file of success.files)
      assert.deepEqual(
        fs.readFileSync(path.join(receiptDir, file.retainedBytes)),
        fs.readFileSync(file.file),
      );
    const lock = path.join(f.cwd, "pnpm-lock.yaml");
    const bytes = fs.readFileSync(lock);
    assert.throws(
      () =>
        prepareRekaCheckFixture({
          ...options,
          run: () => {
            fs.appendFileSync(lock, "# unauthorized mutation\n");
          },
        }),
      /authored source changed/,
    );
    assert.deepEqual(fs.readFileSync(lock), bytes);
    const rejected = JSON.parse(fs.readFileSync(path.join(receiptDir, "receipt.json"), "utf8"));
    assert.equal(rejected.failed, true);
    assert.deepEqual(rejected.sourceMutations, ["pnpm-lock.yaml"]);
    assert.equal(installs, 1);
    assert.equal(fs.existsSync(path.join(f.cwd, ".vize-check-prepare-lock")), false);
  } finally {
    f.cleanup();
  }
});

test("forged Node identities and npx shims fail before an install is permitted", () => {
  const f = fixture();
  try {
    const node = path.join(f.root, path.basename(process.execPath));
    if (process.platform !== "win32") {
      const identity = JSON.stringify({ execPath: process.execPath, version: process.version });
      fs.writeFileSync(node, `#!/bin/sh\nprintf '%s' '${identity}'\n`, { mode: 0o755 });
      assert.throws(() => inspectPrepareNode(node, process.env), /shim changed runtime/);
      let installs = 0;
      assert.throws(
        () =>
          prepareRekaCheckFixture({
            sourceDir: f.cwd,
            authorityRoot: f.root,
            expectedPin: f.pin,
            node,
            run: () => {
              installs++;
            },
          }),
        /shim changed runtime/,
      );
      assert.equal(installs, 0);
      fs.rmSync(node);
    }
    // Actions may supply a real bare Node binary without adjacent npm.
    fs.copyFileSync(fs.realpathSync(process.execPath), node);
    const entry = path.join(f.root, process.platform === "win32" ? "npx.cmd" : "npx");
    fs.writeFileSync(entry, "filesystem-only forged npm shim\n");
    let installs = 0;
    assert.throws(
      () =>
        prepareRekaCheckFixture({
          sourceDir: f.cwd,
          authorityRoot: f.root,
          expectedPin: f.pin,
          node,
          env: {
            [Object.keys(process.env).find((key) => key.toLowerCase() === "path") ?? "PATH"]:
              f.root,
          },
          run: () => {
            installs++;
          },
        }),
      /refuses an npx shim/,
    );
    assert.equal(installs, 0);
    assert.throws(() => inspectRekaNpx({ PATH: f.root }), /refuses an npx shim/);
    assert.equal(
      execFileSync(process.execPath, ["-p", "process.version"], { encoding: "utf8" }).trim(),
      process.version,
    );
  } finally {
    f.cleanup();
  }
});
