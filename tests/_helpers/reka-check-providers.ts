import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import { createRequire } from "node:module";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

import { nearestPackage, walkFiles } from "./nuxt-check-source.ts";

type Retain = (file: string, role: string) => void;

/** Refuse ambient fallbacks: these providers belong to the authored workspace. */
export function inspectRekaProviders(cwd: string, retain: Retain) {
  const root = JSON.parse(fs.readFileSync(path.join(cwd, "package.json"), "utf8"));
  const core = path.join(cwd, "packages/core");
  const manifest = JSON.parse(fs.readFileSync(path.join(core, "package.json"), "utf8"));
  const lock = parse(fs.readFileSync(path.join(cwd, "pnpm-lock.yaml"), "utf8"));
  const providers: Array<Record<string, unknown>> = [];
  for (const [owner, importer, name, request] of [
    [cwd, ".", "typescript", root.devDependencies.typescript],
    [core, "packages/core", "@vue/tsconfig", manifest.devDependencies["@vue/tsconfig"]],
    [core, "packages/core", "vue", manifest.devDependencies.vue],
  ]) {
    const declared = lock.importers[importer].devDependencies[name];
    assert.equal(declared.specifier, request, `authored ${name} request differs from its lock`);
    const expected = declared.version.split("(")[0];
    const logical = path.join(owner, "node_modules", name);
    assert.ok(fs.existsSync(path.join(logical, "package.json")), `missing owned ${name} provider`);
    const physical = fs.realpathSync(logical);
    assert.equal(nearestPackage(owner, name), physical, `shadowed ${name} provider`);
    const actual = JSON.parse(fs.readFileSync(path.join(physical, "package.json"), "utf8"));
    assert.equal(actual.name, name, `wrong ${name} provider identity`);
    assert.equal(actual.version, expected, `wrong locked ${name} provider version`);
    retain(path.join(physical, "package.json"), "installed-provider-manifest");
    providers.push({ owner, name, request, expected, logical, physical });
  }
  const configProvider = path.join(core, "node_modules/@vue/tsconfig");
  const publishedConfig = fileURLToPath(
    new URL("../_fixtures/reka-tsconfig-provider/", import.meta.url),
  );
  for (const name of [
    "package.json",
    "tsconfig.json",
    "tsconfig.dom.json",
    "tsconfig.lib.json",
    "LICENSE",
    "README.md",
  ]) {
    assert.deepEqual(
      fs.readFileSync(path.join(configProvider, name)),
      fs.readFileSync(path.join(publishedConfig, name)),
      `installed @vue/tsconfig differs from the complete pinned published provider: ${name}`,
    );
    retain(path.join(configProvider, name), "installed-config-provider");
  }
  for (const provider of [
    "vue",
    "@vue/runtime-dom",
    "@vue/runtime-core",
    "@vue/reactivity",
    "@vue/shared",
  ]) {
    const directory = nearestPackage(
      fs.realpathSync(path.join(core, "node_modules/vue")),
      provider,
    );
    assert.ok(directory, `missing Vue declaration dependency: ${provider}`);
    retain(path.join(directory, "package.json"), "installed-vue-declaration-manifest");
    walkFiles(
      directory,
      (file) => {
        if (/\.d\.(?:ts|mts|cts)$/.test(file)) retain(file, "installed-vue-declaration-candidate");
      },
      new Set(["node_modules", ".git"]),
    );
  }
  const require = createRequire(path.join(core, "package.json"));
  const parserEntry = require.resolve("typescript");
  const ownedParser = fs.realpathSync(path.join(cwd, "node_modules/typescript"));
  assert.ok(
    fs.realpathSync(parserEntry).startsWith(`${ownedParser}${path.sep}`),
    "config parser was shadowed by an ambient TypeScript provider",
  );
  const ts: typeof import("typescript") = require("typescript");
  assert.equal(ts.version, providers[0].expected, "config parser version differs from its lock");
  retain(parserEntry, "executed-config-parser");
  const config = path.join(core, "tsconfig.app.json");
  const read = ts.readConfigFile(config, (file) => ts.sys.readFile(file));
  assert.equal(read.error, undefined, "authored core config cannot be read");
  const parsed = ts.parseJsonConfigFileContent(read.config, ts.sys, core, undefined, config);
  assert.deepEqual(
    parsed.errors.map((error) => ({
      code: error.code,
      text: ts.flattenDiagnosticMessageText(error.messageText, "\n"),
    })),
    [],
    "authored core config has unresolved or invalid providers",
  );
  assert.equal(parsed.options.strict, true, "the genuine inherited strict config is required");
  return { providers, parserEntry, parserVersion: ts.version, config, options: parsed.options };
}

/** These are the exact original scopes linked by the native PinInput replay. */
export function rekaProviderScopes(cwd: string, authorityRoot: string) {
  const directories = [path.join(cwd, "packages/core"), path.join(cwd, ".histoire")];
  for (let directory = cwd; ; directory = path.dirname(directory)) {
    directories.push(directory);
    if (directory === authorityRoot) break;
    assert.notEqual(directory, path.dirname(directory), "ReKa provider scopes escaped authority");
  }
  return directories.map((owner) => {
    const logical = path.join(owner, "node_modules");
    return {
      owner,
      logical,
      present: fs.existsSync(logical),
      physical: fs.existsSync(logical) ? fs.realpathSync(logical) : null,
    };
  });
}

/** Authenticate npm separately when Actions' selected Node distribution is bare. */
export function inspectRekaNpx(env: NodeJS.ProcessEnv) {
  const pathKey = Object.keys(env).find((key) => key.toLowerCase() === "path") ?? "PATH";
  const candidates = (env[pathKey] ?? "").split(path.delimiter).filter(Boolean);
  const entry = candidates
    .map((directory) => path.join(directory, process.platform === "win32" ? "npx.cmd" : "npx"))
    .find((file) => fs.existsSync(file));
  assert.ok(entry, "ReKa preparation requires an actual npm CLI");
  const real = fs.realpathSync(entry);
  const cli =
    process.platform === "win32"
      ? path.join(path.dirname(entry), "node_modules/npm/bin/npx-cli.js")
      : real;
  assert.ok(
    fs.existsSync(cli) && fs.statSync(cli).isFile(),
    "ReKa preparation refuses an npx shim",
  );
  assert.equal(path.basename(cli), "npx-cli.js", "ReKa preparation refuses an npx shim");
  const manifest = path.resolve(path.dirname(cli), "../package.json");
  const npm = JSON.parse(fs.readFileSync(manifest, "utf8"));
  assert.equal(npm.name, "npm", "ReKa preparation requires npm's genuine CLI");
  assert.match(npm.version, /^\d+\.\d+\.\d+$/);
  return {
    entry,
    cli,
    version: npm.version,
    manifest,
    sha256: createHash("sha256").update(fs.readFileSync(cli)).digest("hex"),
  };
}
