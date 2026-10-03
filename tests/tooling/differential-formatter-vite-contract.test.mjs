import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import vm from "node:vm";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { completeConfig } from "../differential/formatter-vite-observation.mjs";
import {
  originalViteInputs,
  originalViteSources,
  viteConfigurationPlans,
} from "../differential/formatter-vite-source.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
void test("Vite history pins all original vectors while adding nine distinct whole input plans", () => {
  const original = originalViteInputs(root);
  const plans = viteConfigurationPlans(root);
  assert.equal(plans.length, 9);
  assert.equal(new Set(plans.map(({ id }) => id)).size, 9);
  for (const [index, [script, setting]] of original.controls.entries()) {
    assert.equal(plans[index].input, `<script setup lang="ts">\n${script}</script>\n`);
    assert.deepEqual(plans[index].source, { fmt: { sortImports: setting } });
  }
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-source-control-"));
  try {
    for (const file of Object.keys(originalViteSources)) {
      fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
    }
    for (const file of Object.keys(originalViteSources)) {
      fs.appendFileSync(path.join(directory, file), "\n");
      assert.throws(() => originalViteInputs(directory), /original Vite vector changed/);
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("complete config custody keeps undefined, symbol keys and callable source; unproved values refuse", () => {
  const config = async (env) => ({ formatter: { singleQuote: env.command === "fmt" } });
  const symbol = Symbol("tasks");
  const observed = completeConfig({ absent: undefined, [symbol]: { config } });
  assert.equal(observed.properties.length, 2);
  assert.deepEqual(observed.properties[0].value, { type: "undefined" });
  assert.deepEqual(observed.properties[1].key, { symbol: "tasks", global: null });
  assert.equal(observed.properties[1].value.properties[0].value.source, config.toString());
  assert.notDeepEqual(completeConfig({}), completeConfig({ absent: undefined }));
  assert.notDeepEqual(completeConfig({ tasks: 1 }), completeConfig({ [symbol]: 1 }));
  assert.throws(
    () =>
      completeConfig({
        get value() {
          throw new Error("must not run");
        },
      }),
    /accessor/,
  );
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
      if (failure)
        assert.throws(
          () => Reflect.apply(process.dlopen, thisValue, [module, filename, 4]),
          (error) => error === actualError,
        );
      else {
        assert.equal(Reflect.apply(process.dlopen, thisValue, [module, filename, 4]), marker);
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
