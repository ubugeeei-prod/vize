import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";

import { parse } from "yaml";

interface Step {
  readonly name?: string;
  readonly if?: string;
  readonly run?: string;
  readonly uses?: string;
  readonly env?: Readonly<Record<string, string>>;
  readonly with?: Readonly<Record<string, unknown>>;
}

interface Job {
  readonly if?: string;
  readonly needs?: string | readonly string[];
  readonly steps?: readonly Step[];
  readonly uses?: string;
  readonly strategy?: { readonly matrix: { readonly include: readonly unknown[] } };
}
interface Workflow {
  readonly jobs: Readonly<Record<string, Job>>;
}

function workflow(file: string): Workflow {
  return parse(readFileSync(new URL(`../../.github/workflows/${file}`, import.meta.url), "utf8"));
}

const enabled = "needs.channel.outputs.enabled == 'true'";
const jsr = workflow("release-jsr.yml");
const publishSteps = jsr.jobs.publish?.steps ?? [];
const runs = (step: Step, command: RegExp) => command.test(step.run ?? "");

await test("every release runs the JSR publish dry-run, whatever the channel says", () => {
  assert.equal(jsr.jobs.publish?.if, undefined, "the publish job itself is never skipped");
  const dryRuns = publishSteps.filter((step) => runs(step, /deno publish --dry-run/));
  assert.equal(dryRuns.length, 1);
  assert.equal(dryRuns[0]?.if, undefined, "the dry-run has no condition");
});

await test("only an enabled channel publishes to JSR and runs the consumers", () => {
  const publications = publishSteps.filter(
    (step) => runs(step, /deno publish/) && !runs(step, /--dry-run/),
  );
  assert.equal(publications.length, 1);
  assert.equal(publications[0]?.if, enabled);
  assert.equal(jsr.jobs.consumer?.if, enabled);
});

await test("the release calls the JSR workflow after the npm packages it re-exports", () => {
  const release = workflow("release.yml").jobs["release-jsr"];
  assert.equal(release?.uses, "./.github/workflows/release-jsr.yml");
  const needs = new Set([release?.needs ?? []].flat());
  for (const job of ["release-npm-cli", "release-npm-native", "release-npm-vite-plugin"]) {
    assert.ok(needs.has(job), `release-jsr waits for ${job}`);
  }
});

for (const [file, job, condition] of [
  ["release-jsr.yml", "consumer", "matrix.runtime == 'deno'"],
  ["jsr-package.yml", "package", undefined],
] as const) {
  await test(`${file} applies the shared scoped age policy only to Deno`, () => {
    const steps = workflow(file).jobs[job]?.steps ?? [];
    const deno = steps.filter((step) => runs(step, /\bdeno run\b/));
    assert.equal(deno.length, 1);
    assert.equal(deno[0]?.if, condition);
    const script = deno[0]?.run ?? "";
    const copied = script.match(
      /\bcp\s+"?(?:\$GITHUB_WORKSPACE\/)?tools\/support\/release\/jsr\/consumer-deno\.json"?\s+"?([^"\s]+)/,
    );
    assert.ok(copied, "copy the canonical policy into the fresh consumer");
    const config = script.match(/\bdeno run\b[^\n]*--config(?:=|\s+)"?([^"\s]+)/);
    assert.equal(config?.[1], copied[1], "explicitly load the copied policy");
    assert.deepEqual(
      steps.filter((step) => runs(step, /consumer-deno\.json/)),
      deno,
      "Node/Bun and shared setup never load the Deno policy",
    );
    assert.ok(steps.every((step) => !runs(step, /--minimum-dependency-age/)));
  });
}

await test("the hosted package check installs frozen tooling before importing yaml", () => {
  const steps = workflow("jsr-package.yml").jobs.package?.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const install = steps.findIndex((step) => runs(step, /^vp install /));
  const tests = steps.findIndex((step) => runs(step, /^node --test /));
  assert.ok(setup >= 0 && setup < install && install < tests);
  assert.equal(steps[setup]?.with?.["run-install"], false);
  assert.equal(
    steps[install]?.run,
    "vp install --frozen-lockfile --prefer-offline --filter vize-workspace --ignore-scripts",
  );
});

await test("pinned Deno runtime proof retains independent fresh-cache controls and public evidence", () => {
  const steps = workflow("jsr-package.yml").jobs.package?.steps ?? [];
  const deno = steps.findIndex((step) => step.uses?.startsWith("denoland/setup-deno@"));
  const proof = steps.findIndex(
    (step) => step.name === "Verify scoped dependency age with the actual Deno runtime",
  );
  assert.ok(deno >= 0 && deno < proof);
  assert.equal(steps[deno]?.with?.["deno-version"], "2.9.7");
  assert.equal(
    steps[proof]?.run,
    'node tools/support/release/jsr/verify-consumer-age.mjs "$RUNNER_TEMP/jsr-consumer-age"',
  );
  assert.equal(
    steps[proof]?.env?.JSR_CONSUMER_PUBLISHED_VERSION,
    "${{ github.event_name == 'workflow_dispatch' && inputs.published-version || '' }}",
  );
  const staged = steps.find((step) => step.name === "Import the staged entry points under Deno");
  assert.equal(staged?.env?.DENO_DIR, "${{ runner.temp }}/staged-jsr-deno");
  const upload = steps.find(
    (step) => step.name === "Preserve actual Deno consumer commands and outcomes",
  );
  assert.equal(upload?.if, "${{ always() }}");
  assert.equal(upload?.with?.name, "jsr-consumer-age-${{ github.sha }}-${{ github.run_attempt }}");
  assert.equal(upload?.with?.path, "${{ runner.temp }}/jsr-consumer-age/");
});

const consumerFiles = [
  "tools/support/release/jsr/consumer.ts",
  "tools/support/release/jsr/consumer-deno.json",
] as const;
const digest = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");

await test("only the consumer checkout narrows the original six-cell release workflow", () => {
  const consumer = jsr.jobs.consumer;
  const [checkout, ...steps] = consumer.steps ?? [];
  assert.equal(checkout?.uses, "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd");
  assert.deepEqual(checkout.with, {
    ref: "${{ inputs.source_sha || github.sha }}",
    "persist-credentials": false,
    "sparse-checkout": consumerFiles.join("\n") + "\n",
    "sparse-checkout-cone-mode": false,
  });
  const {
    "sparse-checkout": _paths,
    "sparse-checkout-cone-mode": _cone,
    ...original
  } = checkout.with;
  // Whole job objects from main725126c and frozen release source d3eb1c4 are identical.
  // These protect every original ref, default fetch, runtime cell and install/oracle command.
  assert.deepEqual(
    [
      jsr.jobs.channel,
      jsr.jobs.publish,
      { ...consumer, steps: [{ ...checkout, with: original }, ...steps] },
    ].map(digest),
    [
      "deeb84e361bc452eab69cb4ec329902259ef15f443f5bafdd79ba5e46bc949ab",
      "2b7aed1b0120850036d3e89f5134aafc68010f530ec36f87e055486371c23a05",
      "fabc364c2cc5046b5a9ec4688dd5d4698fbc93bad0a8c8787443835b46fb7d06",
    ],
  );
  assert.deepEqual(consumer.strategy?.matrix.include, [
    { os: "ubuntu-24.04", runtime: "node", node: "22" },
    { os: "ubuntu-24.04", runtime: "node", node: "24" },
    { os: "blacksmith-12vcpu-macos-15", runtime: "node", node: "24" },
    { os: "blacksmith-32vcpu-windows-2025", runtime: "node", node: "24" },
    { os: "ubuntu-24.04", runtime: "deno", node: "24" },
    { os: "ubuntu-24.04", runtime: "bun", node: "24" },
  ]);
});

// Execute the original source with sealed public facades. This proves checkout
// closure and the consumer's assertions, never public registry/native acceptance.
const closureWorker = String.raw`
import fs from "node:fs";
import vm from "node:vm";
import { stripTypeScriptTypes } from "node:module";
const input = JSON.parse(fs.readFileSync(0, "utf8"));
const imports = [], calls = [], logs = [];
const record = (entry, args) => calls.push({ entry, args });
const context = vm.createContext({
  process: { argv: ["node", "consumer.ts", input.root, ...input.flags] },
  console: { log: (line) => logs.push(line) },
});
const exports = new Map([
  [input.root, { defineConfig: (...args) => {
    record("defineConfig", args);
    return input.fault === "config" ? { compiler: { vapor: true } } : args[0];
  } }],
  [input.root + "/config", { loadConfig: input.fault === "loadConfig" ? null : () => {
    throw new Error("consumer must not load checkout configuration");
  } }],
  [input.root + "/native", { compileSfc: (...args) => {
    record("compileSfc", args);
    return { errors: input.fault === "errors" ? ["injected"] : [], code: input.fault === "code" ? "" : args[0] };
  } }],
  [input.root + "/vite", { default: (...args) => {
    record("vite", args);
    return input.fault === "plugins" ? [] : [{ name: "sealed-facade" }];
  } }],
]);
try {
  const module = new vm.SourceTextModule(stripTypeScriptTypes(input.source), {
    context,
    importModuleDynamically: async (specifier) => {
      imports.push(specifier);
      if (!exports.has(specifier)) throw new Error("Unexpected consumer import: " + specifier);
      const values = exports.get(specifier);
      const imported = new vm.SyntheticModule(Object.keys(values), function () {
        for (const [key, value] of Object.entries(values)) this.setExport(key, value);
      }, { context });
      await imported.link(() => { throw new Error("facade must not import modules"); });
      await imported.evaluate();
      return imported;
    },
  });
  if (module.dependencySpecifiers.length) throw new Error("consumer must not have static imports");
  await module.link(() => { throw new Error("consumer must not have static imports"); });
  await module.evaluate();
  process.stdout.write(JSON.stringify({ imports, calls, logs }) + "\n");
} catch (error) {
  process.stdout.write(JSON.stringify({ imports, calls, logs, error: error.message }) + "\n");
  process.exitCode = 1;
}
`;

interface Observation {
  readonly imports: readonly string[];
  readonly calls: readonly { readonly entry: string; readonly args: readonly unknown[] }[];
  readonly logs: readonly string[];
  readonly error?: string;
}

function observe(source: string, root: string, flags: readonly string[], fault = "") {
  const result = spawnSync(
    process.execPath,
    [
      "--experimental-vm-modules",
      "--disable-warning=ExperimentalWarning",
      "--input-type=module",
      "-e",
      closureWorker,
    ],
    { input: JSON.stringify({ source, root, flags, fault }), encoding: "utf8", timeout: 10_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.stderr, "");
  const observation: Observation = JSON.parse(result.stdout);
  return { status: result.status, observation };
}

function denoClosure(source: string) {
  const value: unknown = JSON.parse(source);
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  const config = value as Record<string, unknown>;
  const sorted = (values: string[]) =>
    values.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  assert.deepEqual(sorted(Object.keys(config)), ["minimumDependencyAge", "nodeModulesDir"]);
  assert.equal(config.nodeModulesDir, "none");
  const policy = config.minimumDependencyAge;
  assert.ok(policy && typeof policy === "object" && !Array.isArray(policy));
  const age = policy as Record<string, unknown>;
  assert.deepEqual(sorted(Object.keys(age)), ["age", "exclude"]);
  assert.equal(age.age, "P1D");
  assert.ok(Array.isArray(age.exclude));
  const excluded: readonly unknown[] = age.exclude;
  assert.ok(excluded.length > 0);
  for (const specifier of excluded) {
    assert.ok(typeof specifier === "string");
    assert.match(specifier, /^(?:jsr:@vizejs\/vize|npm:(?:vize|@vizejs\/[\w-]+))$/);
  }
}

await test("the physical two-file checkout executes every original consumer entry without repository imports", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "vize-jsr-standalone-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  for (const file of consumerFiles) {
    const destination = join(directory, file);
    mkdirSync(dirname(destination), { recursive: true });
    copyFileSync(new URL(`../../${file}`, import.meta.url), destination);
  }
  assert.deepEqual(
    readdirSync(directory, { recursive: true, encoding: "utf8" })
      .filter((file) => statSync(join(directory, file)).isFile())
      .map((file) => file.replaceAll("\\", "/"))
      .sort((a, b) => a.localeCompare(b)),
    [...consumerFiles].sort((a, b) => a.localeCompare(b)),
  );
  const source = readFileSync(join(directory, consumerFiles[0]), "utf8");
  assert.equal(source, readFileSync(new URL(`../../${consumerFiles[0]}`, import.meta.url), "utf8"));
  const sfc =
    '<script setup lang="ts">const title: string = "JSR consumer"</script><template><h1>{{ title }}</h1></template>';
  for (const [root, flags] of [
    ["@vizejs/vize", ["--vite"]],
    ["jsr:@vizejs/vize@0.467.0", []],
  ] as const) {
    assert.deepEqual(observe(source, root, flags), {
      status: 0,
      observation: {
        imports: [
          root,
          `${root}/config`,
          `${root}/native`,
          ...(flags.length ? [`${root}/vite`] : []),
        ],
        calls: [
          { entry: "defineConfig", args: [{ compiler: { vapor: false } }] },
          { entry: "compileSfc", args: [sfc, { filename: "App.vue" }] },
          ...(flags.length ? [{ entry: "vite", args: [] }] : []),
        ],
        logs: [`Verified ${root} (${flags.join(" ") || "no flags"})`],
      },
    });
  }
  const config = readFileSync(join(directory, consumerFiles[1]), "utf8");
  denoClosure(config);
  for (const change of [
    { extends: "../deno.json" },
    { importMap: "./imports.json" },
    { imports: { local: "./module.ts" } },
    { nodeModulesDir: "auto" },
  ]) {
    assert.throws(() => denoClosure(JSON.stringify({ ...JSON.parse(config), ...change })));
  }
});

await test("standalone consumer closure rejects added static, local, builtin and absolute imports", () => {
  const source = readFileSync(new URL(`../../${consumerFiles[0]}`, import.meta.url), "utf8");
  for (const specifier of [
    "./neighbor.ts",
    "../shared.ts",
    "node:fs",
    "/private/module.ts",
    "file:///private/module.ts",
    "C:\\private\\module.ts",
    "https://example.invalid/module.ts",
    "@vizejs/vize/private",
  ]) {
    for (const prefix of [
      `import ${JSON.stringify(specifier)};`,
      `await import(${JSON.stringify(specifier)});`,
    ]) {
      const result = observe(prefix + "\n" + source, "@vizejs/vize", ["--vite"]);
      assert.deepEqual(result, {
        status: 1,
        observation: {
          imports: prefix.startsWith("await") ? [specifier] : [],
          calls: [],
          logs: [],
          error: prefix.startsWith("await")
            ? `Unexpected consumer import: ${specifier}`
            : "consumer must not have static imports",
        },
      });
    }
  }
});

await test("sealed facades exercise all original consumer refusal assertions", () => {
  const source = readFileSync(new URL(`../../${consumerFiles[0]}`, import.meta.url), "utf8");
  for (const [fault, error] of [
    ["config", "defineConfig returns the authored configuration"],
    ["loadConfig", "the config entry point exports loadConfig"],
    ["errors", "the native compiler reports no errors: injected"],
    ["code", "the native compiler emits the component"],
    ["plugins", "the Vite entry point's default export creates the plugin"],
  ]) {
    const result = observe(source, "@vizejs/vize", ["--vite"], fault);
    assert.equal(result.status, 1);
    assert.equal(result.observation.error, error);
    assert.deepEqual(result.observation.logs, []);
  }
});
