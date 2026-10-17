import assert from "node:assert/strict";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse, stringify } from "yaml";
import {
  docsWasmRecipeFiles,
  verifyDocsWasmConsumers,
} from "./support/rust-cache-docs-wasm-consumers.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const cache = "./.github/actions/setup-rust-sticky-cache";
const headCache = "./head/.github/actions/setup-rust-sticky-cache";
const app = "./.github/actions/app-e2e-row";
const docs = ".github/workflows/build-docs.yml";
function fixture(t) {
  const directory = mkdtempSync(join(os.tmpdir(), "vize-docs-wasm-consumers-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  cpSync(join(root, ".github"), join(directory, ".github"), { recursive: true });
  for (const file of docsWasmRecipeFiles) {
    mkdirSync(dirname(join(directory, file)), { recursive: true });
    cpSync(join(root, file), join(directory, file));
  }
  return directory;
}

function edit(directory, file, change) {
  const json = file.endsWith(".json");
  const bytes = readFileSync(join(directory, file), "utf8");
  const value = json ? JSON.parse(bytes) : parse(bytes);
  change(value);
  writeFileSync(join(directory, file), json ? JSON.stringify(value) + "\n" : stringify(value));
}

function consumer(directory, uses, key) {
  writeFileSync(
    join(directory, ".github/workflows/new-cache-consumer.yml"),
    stringify({
      jobs: {
        consumer: { steps: [{ uses, with: { [uses === app ? "cache-key" : "key"]: key } }] },
      },
    }),
  );
}

void test("actual complete consumers retain the sole WASM producer and separate NAPI recipes", () => {
  const result = verifyDocsWasmConsumers(root);
  assert.deepEqual(result.wasm, [{ file: docs, job: "build-playground" }]);
  assert(result.cacheCalls >= 86);
  assert(result.cacheFileCount >= 56);
  assert(result.app.length >= 4);
  assert(result.napi.length >= 2);
});

void test("retired readiness inputs cannot return or hide unrelated producer changes", (t) => {
  for (const [before, after, refusal] of [
    [
      "  - '.github/actions/app-e2e-row/**'\n",
      "  - '.github/actions/app-e2e-row/**'\n  - '.github/actions/setup-moonbit/**'\n",
      /retired MoonBit readiness input remains/,
    ],
    [
      "  - '.github/actions/setup-rust-sticky-cache/**'\n",
      "  - '.github/actions/unreviewed-cache/**'\n",
      /app-readiness-plan: reviewed matrix producer changed/,
    ],
  ]) {
    const directory = fixture(t);
    edit(directory, ".github/workflows/e2e.yml", (value) => {
      const changes = value.jobs["app-readiness-plan"].steps.find((step) => step.id === "changes");
      assert.equal(changes.with.filters.split(before).length, 2);
      changes.with.filters = changes.with.filters.replace(before, after);
    });
    assert.throws(() => verifyDocsWasmConsumers(directory), refusal);
  }
});

void test("ordinary literal roles and reviewed non-Docs dynamic prefixes remain eligible", (t) => {
  for (const [uses, key] of [
    [cache, "new-ordinary-role"],
    [cache, "docs-wasm-extra"],
    [cache, "docs-napi"],
    [headCache, "benchmark-new-ordinary-role"],
    [cache, "native-smoke-${{ matrix.target }}"],
    [app, "app-e2e-new-ordinary-role"],
  ]) {
    const directory = fixture(t);
    consumer(directory, uses, key);
    const result = verifyDocsWasmConsumers(directory);
    assert.equal(result.wasm.length, 1);
  }
});

void test("new direct, delegated and case-alias Docs WASM consumers require review", (t) => {
  for (const uses of [cache, headCache, app]) {
    for (const key of ["docs-wasm", "Docs-Wasm"]) {
      const directory = fixture(t);
      consumer(directory, uses, key);
      assert.throws(() => verifyDocsWasmConsumers(directory), /Docs WASM/);
    }
  }
});

void test("unknown dynamic roles cannot inherit a fixed-prefix or delegated approval", (t) => {
  for (const [uses, key] of [
    [cache, "${{ matrix.role }}"],
    [headCache, "${{ matrix.role }}"],
    [cache, "unknown-${{ matrix.role }}"],
    [cache, "native-smoke-${{ inputs.role || 'docs-wasm' }}"],
    [app, "${{ inputs.cache-key }}"],
    [app, "${{ matrix.cacheKey }}"],
  ]) {
    const directory = fixture(t);
    consumer(directory, uses, key);
    assert.throws(() => verifyDocsWasmConsumers(directory), /unreviewed/);
  }
});

void test("unreviewed nearby aliases and dynamic action references require review", (t) => {
  for (const uses of [
    "./other/.github/actions/setup-rust-sticky-cache",
    "ubugeeei-prod/vize/.github/actions/setup-rust-sticky-cache@main",
    "${{ inputs.action }}",
  ]) {
    const directory = fixture(t);
    consumer(directory, uses, "new-ordinary-role");
    assert.throws(() => verifyDocsWasmConsumers(directory), /unreviewed .*cache consumer/);
  }
});

void test("local action wrappers cannot hide a Docs WASM consumer outside scanned metadata", (t) => {
  for (const uses of [
    "./tools/new-cache-wrapper",
    "./.github/actions/../../tools/new-cache-wrapper",
    "../tools/new-cache-wrapper",
  ]) {
    const directory = fixture(t);
    const wrapper = join(directory, "tools/new-cache-wrapper");
    mkdirSync(wrapper, { recursive: true });
    writeFileSync(
      join(wrapper, "action.yml"),
      stringify({
        runs: { using: "composite", steps: [{ uses: cache, with: { key: "Docs-Wasm" } }] },
      }),
    );
    consumer(directory, uses, "new-ordinary-role");
    assert.throws(
      () => verifyDocsWasmConsumers(directory),
      /unreviewed local cache consumer action/,
    );
  }
  const directory = fixture(t);
  const wrapper = join(directory, ".github/actions/new-ordinary-wrapper");
  mkdirSync(wrapper, { recursive: true });
  writeFileSync(
    join(wrapper, "action.yml"),
    stringify({
      runs: { using: "composite", steps: [{ uses: cache, with: { key: "new-ordinary-role" } }] },
    }),
  );
  consumer(directory, "./.github/actions/new-ordinary-wrapper", "new-ordinary-role");
  assert.equal(verifyDocsWasmConsumers(directory).wasm.length, 1);
});

void test("an extra WASM edge in the original workflow or wrapper is refused", (t) => {
  for (const file of [docs, ".github/actions/app-e2e-row/action.yml"]) {
    const directory = fixture(t);
    edit(directory, file, (value) => {
      const steps = file === docs ? value.jobs["build-playground"].steps : value.runs.steps;
      steps.push({ uses: cache, with: { key: "Docs-Wasm" } });
    });
    assert.throws(() => verifyDocsWasmConsumers(directory), /Docs WASM/);
  }
});

void test("original WASM, npm native and secondary source commands remain whole", (t) => {
  for (const name of ["Build WASM", "Build npm packages", "Build musea-examples"]) {
    const directory = fixture(t);
    edit(directory, docs, (value) => {
      value.jobs["build-playground"].steps.find((step) => step.name === name).run = "echo stale";
    });
    assert.throws(() => verifyDocsWasmConsumers(directory), /whole Docs WASM producer changed/);
  }
});

void test("WASM conditions, target overrides and native command flags cannot bypass review", (t) => {
  for (const change of [
    (value) => {
      value.jobs["build-playground"].env = { CARGO_TARGET_DIR: "elsewhere" };
    },
    (value) => {
      value.jobs["build-playground"].steps.find((step) => step.name === "Build WASM").if = "false";
    },
    (value) => {
      value.jobs["build-playground"].steps.find((step) => step.id === "cache-wasm").with.path =
        "target";
    },
  ]) {
    const directory = fixture(t);
    edit(directory, docs, change);
    assert.throws(() => verifyDocsWasmConsumers(directory), /whole Docs WASM producer changed/);
  }
  const directory = fixture(t);
  const file = join(directory, "npm/native/scripts/formatter-history-ci-build.mjs");
  writeFileSync(file, readFileSync(file, "utf8").replace('  "napi,legacy",', '  "wasm",'));
  assert.throws(() => verifyDocsWasmConsumers(directory), assert.AssertionError);
});

void test("actual matrix planner and workflow producers cannot introduce new dynamic keys", (t) => {
  for (const plan of ["app-readiness-plan", "app-e2e-plan"]) {
    const directory = fixture(t);
    edit(directory, ".github/workflows/e2e.yml", (value) => {
      value.jobs[plan].steps.find((step) => step.id === "plan").run = 'echo "matrix=docs-wasm"';
    });
    assert.throws(() => verifyDocsWasmConsumers(directory), /matrix producer changed/);
  }
  const directory = fixture(t);
  const file = join(directory, "tools/commands/ci/github/app-e2e-plan.rs");
  writeFileSync(
    file,
    readFileSync(file, "utf8").replace('format!("app-e2e-{prefix}")', 'format!("docs-wasm")'),
  );
  assert.throws(() => verifyDocsWasmConsumers(directory), /command producer changed/);
});

void test("package versions may advance while changed native or secondary scripts are refused", (t) => {
  for (const file of ["npm/native/package.json", "examples/vite-musea/package.json"]) {
    const directory = fixture(t);
    edit(directory, file, (value) => {
      value.version = "999.0.0";
    });
    verifyDocsWasmConsumers(directory);
    edit(directory, file, (value) => {
      value.scripts[file.startsWith("npm/") ? "build:ci" : "check"] = "echo stale";
    });
    assert.throws(() => verifyDocsWasmConsumers(directory), assert.AssertionError);
  }
});

void test("the distinct NAPI caller is not silently reclassified as WASM", (t) => {
  const directory = fixture(t);
  edit(directory, docs, (value) => {
    value.jobs["build-docs"].steps.find((step) => step.uses === cache).with.key = "docs-wasm";
  });
  assert.throws(() => verifyDocsWasmConsumers(directory), /Docs WASM/);
});

void test("inherited producer environment and defaults cannot redirect the original recipes", (t) => {
  for (const file of [docs, ".github/workflows/e2e.yml"]) {
    for (const change of [
      (value) => {
        value.env.CARGO_TARGET_DIR = "elsewhere";
      },
      (value) => {
        value.defaults = { run: { "working-directory": "elsewhere" } };
      },
      (value) => {
        value.defaults = { run: { shell: "python" } };
      },
    ]) {
      const directory = fixture(t);
      edit(directory, file, change);
      assert.throws(
        () => verifyDocsWasmConsumers(directory),
        /inherited .* producer environment changed/,
      );
    }
  }
});

void test("reviewed command source and its ancestors must remain physical", (t) => {
  for (const file of ["npm/native/scripts/build-ci.mjs", "tools/commands"]) {
    const directory = fixture(t);
    const path = join(directory, file);
    renameSync(path, path + ".original");
    symlinkSync(path + ".original", path);
    assert.throws(() => verifyDocsWasmConsumers(directory), /physical source required/);
  }
});

void test("the original app slot keys and separate NAPI inputs remain reviewed", (t) => {
  const directory = fixture(t);
  edit(directory, ".github/workflows/e2e.yml", (value) => {
    value.jobs["app-readiness-producer"].steps.find((step) => step.uses === app).with["cache-key"] =
      "different-ordinary-role";
  });
  assert.throws(() => verifyDocsWasmConsumers(directory), /original app consumer/);
  const napiDirectory = fixture(t);
  edit(napiDirectory, docs, (value) => {
    value.jobs["build-docs"].steps.find((step) => step.uses === cache).with["cache-key-suffix"] =
      "stale";
  });
  assert.throws(() => verifyDocsWasmConsumers(napiDirectory), assert.AssertionError);
});

void test("the original dedicated WASM and native command producers cannot change silently", (t) => {
  for (const file of [
    "tools/support/github/build-vitrine-wasm.ts",
    "npm/native/scripts/build-ci.mjs",
  ]) {
    const directory = fixture(t);
    writeFileSync(
      join(directory, file),
      readFileSync(join(directory, file), "utf8") + "\n// changed command producer\n",
    );
    assert.throws(() => verifyDocsWasmConsumers(directory), /reviewed command producer changed/);
  }
});

void test("the metadata root itself must remain a physical repository directory", (t) => {
  const directory = fixture(t);
  const path = join(directory, ".github");
  renameSync(path, path + ".original");
  symlinkSync(path + ".original", path);
  assert.throws(
    () => verifyDocsWasmConsumers(directory),
    /physical cache consumer metadata required/,
  );
});

void test("the existing head installer alias cannot authorize a new context or hidden cache edge", (t) => {
  const directory = fixture(t);
  consumer(directory, "./head/.github/actions/setup-rust-script", "new-ordinary-role");
  assert.throws(() => verifyDocsWasmConsumers(directory), /unreviewed local cache consumer action/);
  const changed = fixture(t);
  edit(changed, ".github/actions/setup-rust-script/action.yml", (value) => {
    value.runs.steps.push({ uses: cache, with: { key: "Docs-Wasm" } });
  });
  assert.throws(() => verifyDocsWasmConsumers(changed), /Docs WASM/);
});
