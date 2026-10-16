import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { stringify } from "yaml";
import {
  docsWasmRecipeFiles,
  verifyDocsWasmConsumers,
} from "./support/rust-cache-docs-wasm-consumers.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const css = ".github/workflows/css-engine-vendor-candidate.yml";
const cache = "./.github/actions/setup-rust-sticky-cache";
const app = "./.github/actions/app-e2e-row";
const keys = {
  engine: "engines-${{ matrix.profile }}",
  "bundler-before": "engines-${{ matrix.profile }}",
  observations: "observations-${{ matrix.profile }}",
};

function fixture(t) {
  const directory = mkdtempSync(join(os.tmpdir(), "vize-docs-wasm-css-roles-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  cpSync(join(root, ".github"), join(directory, ".github"), { recursive: true });
  for (const file of docsWasmRecipeFiles) {
    mkdirSync(dirname(join(directory, file)), { recursive: true });
    cpSync(join(root, file), join(directory, file));
  }
  return directory;
}

function document() {
  return {
    jobs: Object.fromEntries(
      Object.entries(keys).map(([job, key]) => [
        job,
        {
          strategy: { matrix: { profile: ["dev", "release"] } },
          steps: [
            {
              uses: cache,
              with: {
                key,
                "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
              },
            },
          ],
        },
      ]),
    ),
  };
}

function write(directory, value, file = css) {
  writeFileSync(join(directory, file), stringify(value));
}

void test("the three reviewed CSS slots preserve the sole WASM and separate NAPI roles", (t) => {
  const directory = fixture(t);
  rmSync(join(directory, css), { force: true });
  const before = verifyDocsWasmConsumers(directory);
  write(directory, document());
  const result = verifyDocsWasmConsumers(directory);
  const roles = ({ wasm, napi, app }) => ({ wasm, napi, app });
  assert.deepEqual(roles(result), roles(before));
  assert.equal(result.cacheCalls, before.cacheCalls + 3);
  assert.equal(result.cacheFileCount, before.cacheFileCount + 1);
});

void test("CSS approval is scoped to the original workflow and exact three jobs", (t) => {
  const foreign = fixture(t);
  write(foreign, document(), ".github/workflows/foreign-css-cache.yml");
  assert.throws(() => verifyDocsWasmConsumers(foreign), /unreviewed dynamic/);
  for (const job of ["new-css-job", "constructor"]) {
    const directory = fixture(t);
    const value = document();
    value.jobs[job] = value.jobs.engine;
    delete value.jobs.engine;
    write(directory, value);
    assert.throws(() => verifyDocsWasmConsumers(directory), /unreviewed dynamic/);
  }
});

void test("CSS matrices cannot add profiles, axes, include, exclude or generated inputs", (t) => {
  for (const matrix of [
    { profile: ["dev", "release", "docs-wasm"] },
    { profile: ["dev"] },
    { profile: ["dev", "release"], other: ["docs-wasm"] },
    { profile: ["dev", "release"], include: [{ profile: "docs-wasm" }] },
    { profile: ["dev", "release"], exclude: [{ profile: "dev" }] },
    "${{ fromJSON(inputs.matrix) }}",
  ]) {
    const directory = fixture(t);
    const value = document();
    value.jobs.engine.strategy.matrix = matrix;
    write(directory, value);
    assert.throws(() => verifyDocsWasmConsumers(directory), /CSS cache profile matrix changed/);
  }
});

void test("CSS complete inputs refuse delegation, redirects and unreviewed dynamic roles", (t) => {
  for (const change of [
    (step) => {
      step.uses = app;
    },
    (step) => {
      step.uses = "./head/.github/actions/setup-rust-sticky-cache";
    },
    (step) => {
      step.with.key = "unknown-${{ matrix.profile }}";
    },
    (step) => {
      step.with.key = "engines-${{ inputs.profile }}";
    },
    (step) => {
      step.with.key = "engines-${{ matrix.profile || 'docs-wasm' }}";
    },
    (step) => {
      step.with["cache-key-suffix"] = "other-target";
    },
    (step) => {
      step.with["target-path"] = "elsewhere";
    },
    (step) => {
      step.with["secondary-key"] = "docs-wasm";
    },
    (step) => {
      step.with["secondary-target-path"] = "elsewhere";
    },
  ]) {
    const directory = fixture(t);
    const value = document();
    change(value.jobs.engine.steps[0]);
    write(directory, value);
    assert.throws(
      () => verifyDocsWasmConsumers(directory),
      /unreviewed dynamic|CSS cache role inputs/,
    );
  }
  for (const key of ["docs-wasm", "Docs-Wasm"]) {
    const directory = fixture(t);
    const value = document();
    value.jobs.engine.steps[0].with.key = key;
    write(directory, value);
    assert.throws(() => verifyDocsWasmConsumers(directory), /Docs WASM/);
  }
});
