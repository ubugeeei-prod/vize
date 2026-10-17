import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import vm from "node:vm";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { completeConfig, qualifyReference } from "../differential/formatter-vite-observation.mjs";
import {
  observePublic,
  publicIdentity,
  expectedPublic,
  publicCoreReference,
  publicStrings,
  assertMetadata,
} from "../differential/formatter-vite-public.mjs";
import { originalViteArchives } from "../differential/formatter-vite-source-family.mjs";
import {
  originalViteInputs,
  viteConfigurationPlans,
  migrateViteConfigSource,
  expectedViteNativeConfig,
} from "../differential/formatter-vite-source.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
void test("actual multi-document lock keeps one workspace owner and refuses duplicate or malformed documents", () => {
  const require = createRequire(path.join(root, "package.json"));
  const version = require("oxfmt/package.json").version;
  const lock = qualifyReference(root, require, version);
  assert.equal(
    lock.importers["npm/builder/vite"].devDependencies["vite-plus"].version.split("(")[0],
    "0.2.9",
  );
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-lock-control-"));
  const raw = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8");
  try {
    fs.copyFileSync(
      path.join(root, "pnpm-workspace.yaml"),
      path.join(directory, "pnpm-workspace.yaml"),
    );
    fs.writeFileSync(path.join(directory, "pnpm-lock.yaml"), `${raw}\n${raw}`);
    assert.throws(
      () => qualifyReference(directory, require, version),
      /ambiguous or missing workspace lock importer owner/,
    );
    fs.writeFileSync(path.join(directory, "pnpm-lock.yaml"), `${raw}\n---\ninvalid: [\n`);
    assert.throws(() => qualifyReference(directory, require, version), /YAMLParseError/);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
void test("Vite history pins all original vectors while adding nine distinct whole input plans", () => {
  const original = originalViteInputs(root);
  const plans = viteConfigurationPlans(root);
  assert.equal(plans.length, 9);
  assert.equal(new Set(plans.map(({ id }) => id)).size, 9);
  for (const [index, [script, setting]] of original.controls.entries()) {
    assert.equal(plans[index].input, `<script setup lang="ts">\n${script}</script>\n`);
    assert.deepEqual(plans[index].source, { fmt: { sortImports: setting } });
  }
  const inherited = plans.at(-1);
  assert.deepEqual(inherited.public.server, { port: 4321, host: true });
  const server = expectedPublic(inherited).properties.find(({ key }) => key === "server").value;
  assert.deepEqual(
    server.properties.map(({ key }) => key),
    ["port", "ws", "hmr", "host"],
  );
  const hmr = server.properties.find(({ key }) => key === "hmr").value;
  assert.deepEqual(
    hmr.properties.map(({ key }) => key),
    publicCoreReference.keys,
  );
  for (const descriptor of hmr.properties) {
    assert.equal(descriptor.kind, "accessor");
    assert.equal(descriptor.get.source, publicCoreReference.getter);
    assert.equal(descriptor.set.source, publicCoreReference.setter);
  }
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-source-control-"));
  try {
    const archives = Object.values(originalViteArchives);
    for (const file of archives) {
      fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
    }
    assert.deepEqual(originalViteInputs(directory), original);
    for (const file of archives) {
      fs.appendFileSync(path.join(directory, file), "\n");
      assert.throws(() => originalViteInputs(directory), /historical Vite vector changed/);
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("complete config custody keeps undefined, symbols and noninvoked accessor identity; unproved values refuse", () => {
  const config = async (env) => ({ formatter: { singleQuote: env.command === "fmt" } });
  const symbol = Symbol("tasks");
  const observed = completeConfig({ absent: undefined, [symbol]: { config } });
  assert.equal(observed.properties.length, 2);
  assert.deepEqual(observed.properties[0].value, { type: "undefined" });
  assert.deepEqual(observed.properties[1].key, { symbol: "tasks", global: null });
  assert.equal(observed.properties[1].value.properties[0].value.source, config.toString());
  assert.notDeepEqual(completeConfig({}), completeConfig({ absent: undefined }));
  assert.notDeepEqual(completeConfig({ tasks: 1 }), completeConfig({ [symbol]: 1 }));
  let accessorCalls = 0;
  const guarded = {
    get value() {
      accessorCalls++;
      throw new Error("must not run");
    },
  };
  const before = observePublic(guarded);
  assert.equal(before.snapshot.properties[0].kind, "accessor");
  assert.deepEqual(before.snapshot.properties[0].set, { type: "undefined" });
  const descriptor = Object.getOwnPropertyDescriptor(guarded, "value");
  assert.equal(before.functions[0].value, Object.getOwnPropertyDescriptor(descriptor, "get").value);
  assert.equal(publicIdentity(guarded, before).functions[0].sameFunction, true);
  assert.equal(accessorCalls, 0);
  assert.throws(() => completeConfig({ pattern: /vue/ }), /unsupported public config value/);
  config.extra = true;
  assert.throws(() => completeConfig(config), /unsupported callable/);
});

void test("declared CLI loader observation forwards real return and identical thrown Error, even if custody writes fail", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-loader-control-"));
  const filename = path.join(directory, "synthetic-negative-addon.input");
  const destination = path.join(directory, "capture.json");
  fs.writeFileSync(filename, "synthetic control bytes; no addon or native credit");
  const source = fs.readFileSync(
    path.join(root, "tests/differential/formatter-vite-loader.cjs"),
    "utf8",
  );
  const actualError = new Error("original loader error");
  const marker = {};
  const thisValue = {};
  const module = { exports: { runCli() {} } };
  try {
    for (const failure of [false, true]) {
      const process = {
        env: {
          VIZE_VITE_CLI_LOAD_CAPTURE: failure
            ? path.join(directory, "missing", "capture.json")
            : destination,
        },
        dlopen(...args) {
          assert.equal(this, thisValue);
          assert.deepEqual(args, [module, filename, 4]);
          if (failure) throw actualError;
          return marker;
        },
      };
      vm.runInNewContext(source, {
        process,
        require: (name) => {
          if (name === "node:fs") return fs;
          if (name === "node:crypto") return requireCrypto;
          throw new Error(name);
        },
      });
      const invoke = () =>
        Reflect.apply(Object.getOwnPropertyDescriptor(process, "dlopen").value, thisValue, [
          module,
          filename,
          4,
        ]);
      if (failure) assert.throws(invoke, (error) => error === actualError);
      else {
        assert.equal(invoke(), marker);
        const { loads } = JSON.parse(fs.readFileSync(destination));
        assert.equal(loads.length, 1);
        assert.equal(loads[0].completed, true);
        assert.equal(loads[0].beforeSha256, loads[0].afterSha256);
        assert.deepEqual(loads[0].observationErrors, []);
        assert.equal(loads[0].exportDescriptors[0].valueType, "function");
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

import * as requireCrypto from "node:crypto";

void test("active history inputs migrate namespaces while archived inheritance remains evidence", () => {
  const original = originalViteInputs(root);
  const base = migrateViteConfigSource(original.inherited.base);
  const own = migrateViteConfigSource(original.inherited.own);
  assert.equal(base.lint.vue.preset, "essential");
  assert.equal(own.lint.vue.typecheck, false);
  assert.equal(own.fmt.vue.tabWidth, 4);
  assert.ok(Object.hasOwn(original.inherited.own, "extends"));
  assert.ok(!Object.hasOwn(own, "extends"));
  assert.ok(!Object.hasOwn(base, "vize"));
  const inherited = viteConfigurationPlans(root).at(-1);
  const expected = expectedViteNativeConfig(inherited, "/authored/project");
  assert.equal(expected.entries.length, 1);
  assert.equal(expected.__vizeProjectRoot, "/authored/project");
  assert.equal(expected.entries[0].__vizeProjectRoot, "/authored/project");
  assert.deepEqual(expected.typeChecker, { jsxTypecheck: true, strict: true });
  assert.deepEqual(expected.formatter, { singleQuote: true, tabWidth: 4 });
});

void test("all migrated public snapshots keep the independently authored Vite and task oracles", async () => {
  const { defineConfig } = await import("../../npm/builder/vite/src/vite-plus.ts");
  const { taskConfigKey, sourceConfigKey } =
    await import("../../npm/builder/vite/src/vite-plus/types.ts");
  const require = createRequire(path.join(root, "npm/builder/vite/package.json"));
  const { mergeConfig } = require("vite-plus");
  for (const fixture of viteConfigurationPlans(root)) {
    const source = fixture.base
      ? mergeConfig(
          mergeConfig(
            migrateViteConfigSource(fixture.base),
            migrateViteConfigSource(fixture.promisedBase),
          ),
          migrateViteConfigSource(fixture.source),
        )
      : migrateViteConfigSource(fixture.source);
    const result = await defineConfig(
      source,
      fixture.integration,
    )({ command: "build", mode: "production" });
    assert.deepEqual(
      publicStrings(observePublic(result).snapshot),
      expectedPublic(fixture),
      fixture.id,
    );
    assertMetadata(result[taskConfigKey], fixture, {});
    assert.equal(result[sourceConfigKey], source);
  }
});
