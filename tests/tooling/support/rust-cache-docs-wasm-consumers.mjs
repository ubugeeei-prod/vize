import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { parse } from "yaml";

const cacheAction = "./.github/actions/setup-rust-sticky-cache";
const headCacheAction = "./head/.github/actions/setup-rust-sticky-cache";
const headScriptAction = "./head/.github/actions/setup-rust-script";
const appAction = "./.github/actions/app-e2e-row";
const appFile = ".github/actions/app-e2e-row/action.yml";
const docsFile = ".github/workflows/build-docs.yml";
const e2eFile = ".github/workflows/e2e.yml";
const hash = (value) => createHash("sha256").update(value).digest("hex");
const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const dynamicPrefixes = new Set([
  "davinci-production-pairs-",
  "fresco-dimensions-",
  "fuzz-",
  "native-smoke-",
  "fresh-install-smoke-",
  "oxc-pure-capture-",
  "real-project-matrix-",
  "release-cli-",
  "release-native-",
]);

// Complete command/metadata producers reviewed at edc003416dafd322aa6cd100dde488fb112a2abb.
// The WASM command was re-reviewed as its TypeScript port and imports (#8840).
// A changed producer requires review, rather than accepting a new dynamic role.
const producerHashes = {
  "tools/commands/ci/github/app-e2e-plan.rs":
    "46983c8019bfb809f418eccdab5afeb73b3cea5dcc7c4aa8354d2726bf9c617b",
  "tools/support/github/build-vitrine-wasm.ts":
    "1cdcb19eb0e60ad688a965fbdaa2dc7de90f306c910461c25a11f2ad651c61f7",
  "tools/support/tasks/build-vitrine-wasm.ts":
    "ca6d0627ce29ca69b46f5189fd2a42055e4733dd0e416e3370348d22c8c869a2",
  "tools/support/command-runtime.ts":
    "256b427af28975329f8c51b7043407801a78b12e4b75e4c6178d92c5a7938f54",
  "npm/native/scripts/build-ci.mjs":
    "5e9bf50544adbd1e225f7864bb76ebf73f477eeae19000c74de15edcf3e0ada4",
};
export const docsWasmRecipeFiles = [
  ...Object.keys(producerHashes),
  "npm/native/package.json",
  "npm/native/scripts/formatter-history-ci-build.mjs",
  "examples/vite-musea/package.json",
];

function read(root, file) {
  let path = root;
  const parts = file.split("/");
  for (const [index, part] of parts.entries()) {
    path = join(path, part);
    const stat = lstatSync(path);
    assert(!stat.isSymbolicLink(), `${file}: physical source required`);
    assert(
      index === parts.length - 1 ? stat.isFile() : stat.isDirectory(),
      `${file}: physical source required`,
    );
  }
  return readFileSync(join(root, file), "utf8");
}

function documents(root, directory) {
  const metadataRoot = lstatSync(join(root, ".github"));
  assert(
    metadataRoot.isDirectory() && !metadataRoot.isSymbolicLink(),
    "physical cache consumer metadata required",
  );
  const result = new Map();
  const walk = (path) => {
    const stat = lstatSync(path);
    assert(!stat.isSymbolicLink(), "physical cache consumer metadata required");
    if (stat.isDirectory()) {
      for (const entry of readdirSync(path).sort()) walk(join(path, entry));
    } else if (/\.ya?ml$/.test(path)) {
      assert(stat.isFile(), "regular cache consumer metadata required");
      result.set(relative(root, path).replaceAll("\\", "/"), parse(readFileSync(path, "utf8")));
    }
  };
  walk(join(root, directory));
  return result;
}

function steps(document) {
  return [
    ...Object.entries(document?.jobs ?? {}).flatMap(([job, value]) =>
      (value.steps ?? []).map((step) => ({ job, step })),
    ),
    ...(document?.runs?.steps ?? []).map((step) => ({ job: "action", step })),
  ];
}

function ordinaryRole(key) {
  if (typeof key !== "string") return false;
  if (!key.includes("${{")) return token.test(key) && key.toLowerCase() !== "docs-wasm";
  if (/^release-(?:cli|native)-\$\{\{ inputs\.target \}\}$/.test(key)) return true;
  const prefix = key.slice(0, key.indexOf("${{"));
  return (
    dynamicPrefixes.has(prefix) &&
    /^(?:[A-Za-z0-9._-]+|\$\{\{ matrix\.[A-Za-z0-9_.-]+ \}\})+$/.test(key)
  );
}

function reviewedCssRole(file, job, document, step) {
  if (file !== ".github/workflows/css-engine-vendor-candidate.yml" || step.uses !== cacheAction)
    return false;
  const key = ["engine", "bundler-before"].includes(job)
    ? "engines-${{ matrix.profile }}"
    : job === "observations"
      ? "observations-${{ matrix.profile }}"
      : null;
  if (!key) return false;
  assert.deepEqual(
    document.jobs[job].strategy?.matrix,
    { profile: ["dev", "release"] },
    "reviewed CSS cache profile matrix changed",
  );
  assert.deepEqual(
    step.with,
    {
      key,
      "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
    },
    "reviewed CSS cache role inputs changed",
  );
  return true;
}

function verifyRecipes(root, docs, e2e) {
  assert.deepEqual(
    { env: docs.env, defaults: docs.defaults },
    { env: { FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true }, defaults: undefined },
    "reviewed inherited Docs WASM producer environment changed",
  );
  assert.deepEqual(
    { env: e2e.env, defaults: e2e.defaults },
    {
      env: {
        CARGO_TERM_COLOR: "always",
        FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true,
        NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
        E2E_TARGET_SHA: "${{ inputs.target_sha || github.sha }}",
      },
      defaults: undefined,
    },
    "reviewed inherited matrix producer environment changed",
  );
  // Seal the whole dedicated producer, including conditions, environment and
  // generated-WASM cache. Do not freeze the other Docs job or product libraries.
  assert.equal(
    hash(JSON.stringify(docs.jobs["build-playground"])),
    "9fb6e8300bea6cbcaae07e8b019d78c4ebeba9d39e7caa9ecd3e6171f65b7023",
    "reviewed whole Docs WASM producer changed",
  );
  for (const [file, expected] of Object.entries(producerHashes))
    assert.equal(hash(read(root, file)), expected, `${file}: reviewed command producer changed`);
  for (const [job, expected] of [
    ["app-readiness-plan", "6eedaa2dfd736d68bd260d4cd52e180676ebcf61b770a929e708babc4e6e480c"],
    ["app-e2e-plan", "554e3df0e2b4638c138ac1c0f40ede1cade07152872b1705e3269b5ef2826595"],
  ])
    assert.equal(
      hash(JSON.stringify(e2e.jobs[job])),
      expected,
      `${job}: reviewed matrix producer changed`,
    );
  const native = JSON.parse(read(root, "npm/native/package.json"));
  assert.equal(native.scripts["build:ci"], "node ./scripts/build-ci.mjs");
  const example = JSON.parse(read(root, "examples/vite-musea/package.json"));
  assert.equal(example.scripts.build, "pnpm check && vp build --config vite.app.config.ts");
  assert.equal(
    example.scripts.check,
    "cargo run --quiet --target-dir ../../.docs-example-target --manifest-path ../../Cargo.toml -p vize -- check && vp check src vite.config.ts vite.app.config.ts vite.ui-gallery.config.ts vite.globals.config.ts musea.preview.ts playwright.config.ts",
  );
  const source = read(root, "npm/native/scripts/formatter-history-ci-build.mjs");
  const commands = [...source.matchAll(/export const ciBuildCommand = (\[[\s\S]*?\]);/g)];
  assert.equal(commands.length, 1, "one literal original native command authority");
  assert.deepEqual(JSON.parse(commands[0][1].replace(/,\s*]/g, "]")), [
    "napi",
    "build",
    "--platform",
    "--profile",
    "ci",
    "--manifest-path",
    "../../crates/vize_vitrine/Cargo.toml",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
    "--output-dir",
    ".",
    "--",
    "--message-format=json-render-diagnostics",
    "--locked",
  ]);
}

export function verifyDocsWasmConsumers(root) {
  const source = new Map([
    ...documents(root, ".github/workflows"),
    ...documents(root, ".github/actions"),
  ]);
  const docs = source.get(docsFile);
  const e2e = source.get(e2eFile);
  assert(docs && e2e, "original Docs and app matrix producers remain present");
  const wasm = [];
  const napi = [];
  const app = [];
  let delegatedCache = 0;
  let cacheCalls = 0;
  const cacheFiles = new Set();
  for (const [file, document] of source) {
    for (const { job, step } of steps(document)) {
      if (typeof step.uses !== "string") continue;
      assert(!step.uses.includes("${{"), "unreviewed dynamic cache consumer action");
      if (step.uses.startsWith(".")) {
        const path = step.uses.slice(2);
        assert(
          step.uses === headCacheAction ||
            (step.uses === headScriptAction &&
              file === ".github/workflows/level-memory-bench.yml" &&
              job === "memory" &&
              source.has(".github/actions/setup-rust-script/action.yml")) ||
            (step.uses.startsWith("./.github/actions/") &&
              path.split("/").every((part) => part && part !== "." && part !== "..") &&
              (source.has(`${path}/action.yml`) || source.has(`${path}/action.yaml`))),
          "unreviewed local cache consumer action",
        );
      }
      if (!/(?:setup-rust-sticky-cache|app-e2e-row)(?:\/|@|$)/.test(step.uses)) continue;
      assert(
        [cacheAction, headCacheAction, appAction].includes(step.uses),
        "unreviewed cache consumer reference",
      );
      const isCache = step.uses !== appAction;
      const key = step.with?.[isCache ? "key" : "cache-key"];
      if (isCache) {
        cacheCalls += 1;
        cacheFiles.add(file);
      }
      if (file === appFile && step.uses === cacheAction && key === "${{ inputs.cache-key }}") {
        delegatedCache += 1;
        assert.deepEqual(step.with, {
          key: "${{ inputs.cache-key }}",
          "cache-key-suffix":
            "${{ runner.os }}-${{ runner.arch }}-${{ steps.rust-toolchain.outputs.cachekey }}",
        });
        continue;
      }
      if (typeof key === "string" && key.toLowerCase() === "docs-wasm") {
        assert.equal(step.uses, cacheAction, "delegated Docs WASM consumer requires review");
        assert.equal(file, docsFile, "new Docs WASM consumer requires review");
        assert.equal(job, "build-playground", "new Docs WASM job requires review");
        wasm.push({ file, job });
      } else if (step.uses === appAction && key === "${{ matrix.cacheKey }}") {
        assert.equal(file, e2eFile, "unreviewed delegated dynamic cache role");
        const plan = job === "app-readiness-producer" ? "app-readiness-plan" : "app-e2e-plan";
        assert(["app-readiness-producer", "app-e2e-producer"].includes(job));
        assert.equal(
          document.jobs[job].strategy.matrix,
          `\${{ fromJSON(needs.${plan}.outputs.matrix) }}`,
        );
        app.push({ file, job, key });
      } else {
        assert(
          ordinaryRole(key) || reviewedCssRole(file, job, document, step),
          "unreviewed dynamic or possible Docs WASM cache role",
        );
        if (step.uses === appAction) app.push({ file, job, key });
      }
      if (isCache && key === "docs-napi") {
        if (
          (file === docsFile && job === "build-docs") ||
          (file === ".github/workflows/ci-walltime-benchmark.yml" && job === "docs")
        )
          assert.deepEqual(step.with, {
            key: "docs-napi",
            "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
          });
        napi.push({ file, job });
      }
    }
  }
  assert.equal(wasm.length, 1, "exactly one reviewed Docs WASM consumer");
  assert.equal(delegatedCache, 1, "one reviewed app cache delegation");
  for (const [file, job] of [
    [docsFile, "build-docs"],
    [".github/workflows/ci-walltime-benchmark.yml", "docs"],
  ])
    assert(
      napi.some((entry) => entry.file === file && entry.job === job),
      "distinct Docs NAPI caller",
    );
  for (const [file, job, key] of [
    [e2eFile, "app-readiness-producer", "${{ matrix.cacheKey }}"],
    [e2eFile, "app-e2e-producer", "${{ matrix.cacheKey }}"],
    [
      ".github/workflows/typechecker-syntax-semantic-completion.yml",
      "native-syntax-completion",
      "app-e2e-full-check-all",
    ],
    [
      ".github/workflows/vuefes-ssr-style-dependencies.yml",
      "vuefes-ssr-style-dependencies",
      "app-e2e-full-vrt-vuefes",
    ],
  ])
    assert(
      app.some((entry) => entry.file === file && entry.job === job && entry.key === key),
      "original app consumer",
    );
  verifyRecipes(root, docs, e2e);
  return { wasm, napi, app, cacheCalls, cacheFileCount: cacheFiles.size };
}
