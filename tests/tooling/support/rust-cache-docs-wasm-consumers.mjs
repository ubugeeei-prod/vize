import { restoreVpVersionPin } from "./github-workflows.ts";
import { packageCommands } from "./package-task-source.ts";
import { stagedWorkflowAction, verifyWorkflowStage } from "./workflow-stage.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { basename, join, relative } from "node:path";
import { parse } from "yaml";

const cacheAction = "./.github/actions/setup-rust-sticky-cache";
const headCacheAction = "./head/.github/actions/setup-rust-sticky-cache";
const headScriptAction = "./head/.github/actions/setup-rust-script";
const qualificationWorkflows = new Set();
const qualificationActions = new Map([
  [
    "./qualification/.github/actions/setup-wild-http-recovery",
    ".github/actions/setup-wild-http-recovery/action.yml",
  ],
  [
    "./qualification/.github/actions/setup-rust-script",
    ".github/actions/setup-rust-script/action.yml",
  ],
]);
const performanceRecoveryAction = "./driver/.github/actions/setup-wild-http-recovery";
const shippedMeasureActions = new Map([
  [
    "./driver/.github/actions/setup-cargo-zigbuild",
    ".github/actions/setup-cargo-zigbuild/action.yml",
  ],
  [performanceRecoveryAction, ".github/actions/setup-wild-http-recovery/action.yml"],
]);
const appAction = "./.github/actions/app-e2e-row";
const appFile = ".github/actions/app-e2e-row/action.yml";
const docsFile = ".github/workflows/11-docs-build.yml";
const e2eFile = ".github/workflows/10-app-e2e.yml";
const hash = (value) => createHash("sha256").update(value).digest("hex");
const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const dynamicPrefixes = new Set([
  "davinci-production-pairs-",
  "fresco-dimensions-",
  "fuzz-",
  "native-smoke-",
  "fresh-install-smoke-",
  "real-project-matrix-",
  "release-cli-",
  "release-native-",
]);

// Complete command/metadata producers reviewed at edc003416dafd322aa6cd100dde488fb112a2abb.
// The WASM command was re-reviewed as its TypeScript port and imports.
// A changed producer requires review, rather than accepting a new dynamic role.
// Producer authentication retains every original approved whole-file digest.
// J1 appends only re-reviewed generated-notice and workflow-path updates;
// the complete program, including all comments, remains sealed.
const producerHashes = {
  "examples/vite-musea/scripts/package-tasks.cjs": [
    "238c4edc5a633bece09a0f0cad5858e574cb2b2874a3eb2594b785e17e1c26ef",
    "899b639ce694ec20edfaa11e01336ee587f86372defad64d4428ba220a0478a8",
  ],
  ".github/actions/stage-workflow-scripts/action.yml": [
    "3964722bfe0474b2a026449850af68bae0eab7f6a7f94b2ae597dac941fdf58c",
    "631bc9b08a4a78c996b40ca2245079bcc61d5df435285d6a0d3fd5f517b118d1",
  ],
  ".github/actions/stage-workflow-scripts/stage.ts": [
    "25af079b7060133e9261d1c99d60dd8aade5298e85fe68cf604ceacc4f0654de",
    "3b36f28aad148dbb68b194206bc08a7d3fed275c1dc019a28d41ab96590b5bde",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/plan-full-rows.sh": [
    "48db48b3259ddf195d7b41563c396ef9c6058ee1ccd7123dcc8488e60d645a66",
    "fb74e8bacfa630535f68514899ac0fd7ee714c03c22c4b7e1369b3702be741c6",
    "d97dd5b74540264b9eec7c3674b378a0c0144d3520c6e7ab604abb29f6e737f6",
    "415afde008a3d7ad6f5867b87e7c2d1db01f3b03df7144caf03c91e35a6852d9",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/plan-readiness-rows.sh": [
    "1bdf100557be3c0d25eb0217a122750aedf04887daee0e34db294807665a834d",
    "0cd915f52568251954116c9b5e0a20071218df9be50f41e4a9fdb3f583ebb104",
    "dec2510c81d47bc70efd1c1d6591a98bfbf667cf453d07be146ed7b222b3eada",
    "bc254fc2fd481e35e89d949f6f554ea5739a22bb80c775147979263e0eef3cd1",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/validate-target-sha.sh": [
    "0a8e0b4584b88fd1712b97387a3b554c1fa8d642c932bb590758159a144420fa",
    "a1c84e218ce74cd77e8e687687cfb3850e8cb9068315aa5a2788275ff1795fbe",
    "a3001d2f40bd202a802d53e2cda49b0edf96f5729f55ab6361c3fa3b469450cd",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/verify-producer-sha.sh": [
    "79b2cd2e878ca1d71373872ded348d38d7bcdd3ddc622d0fec32523cf6c56ef2",
    "cdd052be7763e1a399336f2a55af189a92ec9702bb92cc7898c061ccbb32afff",
    "665e560c2ba249dfba54dabc311ced52a00f64ebf6d6e383a25ff05125eb3124",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/verify-readiness-merge-sha.sh": [
    "e1845eaf90a99bf275e2be08fcffbf7cd5a1d052678dc427d43de4d839cf4f37",
    "37bed01d35ddc40baa321dd7c8844878c44a31bc4b01187564f0b51d19c0baa4",
    "f98eefeeef4d8f6f97545124db089aba61f445896e7244cfe475ceba032acb16",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/write-full-plan-evidence.sh": [
    "007613ed91016c66a320038841944ab5225390d7461438093af557646c11d991",
    "3e9f329999396fcf93f9d66163abd21c7dacdc10067cb42aee15cd403050ac26",
    "b0e7bc04e2d4740371c263d6661efb38fb1a00e75f5fd86f52cc5206aa5ed49e",
    "0303f6e9676b5188b6ae29a7c1032536368bf0567b04ad40da84b61b0ecdb186",
  ],
  "tools/ci/workflows/10-app-e2e/scripts/write-readiness-plan-evidence.sh": [
    "e7110ceaa0266e4a479b2de79660834f82ace685e217be9a8967d14a56b206d6",
    "b2c01a84ad82128243d5302201ee01d62921bc4012f0cf7915c2b0e0c8bea8af",
    "84bad58cff17649cef276be7bfac7583b47733ffc8704bf99ee04eebd6f067ad",
    "e6a2ae6264bec09454583e87a6d0af41167d44c1b73f8f9361d19977860dcf93",
  ],
  "tools/ci/commands/github/app-e2e-plan.ts":
    "ca7682cc8f0834fa48a325954fbcc10765241ee1f9e29a0ebef9b60048cbc7d2",
  "tools/support/compat/github/app-e2e-plan.ts":
    "67799c81a7a38031b96e5a0f1cd1ea5bd2ae77a30eaf750a389f33f26e290ad7",
  "tools/support/github/build-vitrine-wasm.ts":
    "1cdcb19eb0e60ad688a965fbdaa2dc7de90f306c910461c25a11f2ad651c61f7",
  "tools/support/tasks/build-vitrine-wasm.ts":
    "ca6d0627ce29ca69b46f5189fd2a42055e4733dd0e416e3370348d22c8c869a2",
  "tools/support/command-runtime.ts":
    "d92427dbdd6a892734532745828d90c8f378a143a9fc0e85ef1693fc76466921",
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
  if (file !== ".github/workflows/06-css-engine-forks.yml" || step.uses !== cacheAction)
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
  // The seal predates the Vite+ CLI pin, so compare it with that input removed.
  const producer = structuredClone(docs.jobs["build-playground"]);
  assert.equal(restoreVpVersionPin(producer.steps), 1, "Docs WASM producer Vite+ pin");
  assert.equal(
    hash(JSON.stringify(producer)),
    "8039a8eba4e5a56c1a668ada91829b33f27c9a94450b34cc4a608a6762a330f0",
    "reviewed whole Docs WASM producer changed",
  );
  for (const [file, expected] of Object.entries(producerHashes)) {
    const actual = hash(read(root, file));
    const approved = Array.isArray(expected) ? expected : [expected];
    assert(approved.includes(actual), `${file}: reviewed command producer changed (${actual})`);
  }
  // Restore only the retired path for comparison with the unchanged reviewed seal.
  const readiness = structuredClone(e2e.jobs["app-readiness-plan"]);
  const changes = readiness.steps.find((step) => step.id === "changes");
  const filters = changes?.with?.filters;
  assert.equal(typeof filters, "string", "literal readiness inputs required");
  assert.equal(filters.includes("setup-moonbit"), false, "retired MoonBit readiness input remains");
  const anchor = "  - '.github/actions/app-e2e-row/**'\n";
  assert.equal(filters.split(anchor).length, 2, "reviewed readiness input anchor changed");
  changes.with.filters = filters.replace(
    anchor,
    anchor + "  - '.github/actions/setup-moonbit/**'\n",
  );
  // Re-reviewed for the CI moves: only the tools/ci paths and the 10-app-e2e file name changed.
  for (const [job, expected] of [
    ["app-readiness-plan", "a57074f33fd9843d50ecfec54cf60bf9a4639ab9a840aa073cad6d3e325cf0f5"],
    ["app-e2e-plan", "40363b39c9652dc32c1f5f6208cfeee24943ffd929be4363f925062a2af8527e"],
  ])
    assert.equal(
      hash(JSON.stringify(job === "app-readiness-plan" ? readiness : e2e.jobs[job])),
      expected,
      `${job}: reviewed matrix producer changed`,
    );
  const native = JSON.parse(read(root, "npm/native/package.json"));
  assert.equal(native.scripts["build:ci"], "node ./scripts/build-ci.mjs");
  const example = JSON.parse(read(root, "examples/vite-musea/package.json"));
  assert.equal(
    packageCommands(root, "examples/vite-musea", example.scripts.build, "build").join(" && "),
    "pnpm check && vp build --config vite.app.config.ts",
  );
  assert.equal(
    packageCommands(root, "examples/vite-musea", example.scripts.check, "check").join(" && "),
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
      if (step.uses === stagedWorkflowAction) {
        assert.ok(file.startsWith(".github/workflows/"));
        const group = document.jobs[job].steps;
        assert.equal(group.indexOf(step), 1);
        verifyWorkflowStage(basename(file, ".yml"), group[0], step);
        assert.deepEqual(source.get(".github/actions/stage-workflow-scripts/action.yml")?.runs, {
          using: "node24",
          main: "stage.ts",
        });
        continue;
      }
      if (step.uses.startsWith(".")) {
        const path = step.uses.slice(2);
        assert(
          step.uses === headCacheAction ||
            (step.uses === headScriptAction &&
              file === ".github/workflows/03-level-memory.yml" &&
              job === "memory" &&
              source.has(".github/actions/setup-rust-script/action.yml")) ||
            (qualificationWorkflows.has(file) &&
              qualificationActions.has(step.uses) &&
              source.has(qualificationActions.get(step.uses))) ||
            (step.uses === performanceRecoveryAction &&
              file === ".github/workflows/pr-performance-observation.yml" &&
              job === "observe" &&
              source.has(".github/actions/setup-wild-http-recovery/action.yml")) ||
            (file === ".github/workflows/shipped-profile-measure.yml" &&
              job === "measure" &&
              shippedMeasureActions.has(step.uses) &&
              source.has(shippedMeasureActions.get(step.uses))) ||
            (step.uses.startsWith("./.github/actions/") &&
              path.split("/").every((part) => part && part !== "." && part !== "..") &&
              (source.has(`${path}/action.yml`) || source.has(`${path}/action.yaml`))),
          "unreviewed local cache consumer action",
        );
      }
      if (
        file === ".github/actions/setup-rust-sticky-cache/action.yml" &&
        step.uses === `${cacheAction}/cache-policy`
      ) {
        const policy = source.get(
          ".github/actions/setup-rust-sticky-cache/cache-policy/action.yml",
        );
        assert.deepEqual(policy?.runs, { using: "node24", main: "main.ts" });
        continue;
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
          (file === ".github/workflows/03-ci-walltime.yml" && job === "docs")
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
    [".github/workflows/03-ci-walltime.yml", "docs"],
  ])
    assert(
      napi.some((entry) => entry.file === file && entry.job === job),
      "distinct Docs NAPI caller",
    );
  for (const [file, job, key] of [
    [e2eFile, "app-readiness-producer", "${{ matrix.cacheKey }}"],
    [e2eFile, "app-e2e-producer", "${{ matrix.cacheKey }}"],
  ])
    assert(
      app.some((entry) => entry.file === file && entry.job === job && entry.key === key),
      "original app consumer",
    );
  verifyRecipes(root, docs, e2e);
  return { wasm, napi, app, cacheCalls, cacheFileCount: cacheFiles.size };
}
