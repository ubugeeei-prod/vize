import assert from "node:assert/strict";
import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, realpathSync } from "node:fs";
import { cpus, release, totalmem } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import {
  upstreamSha,
  json,
  write,
  evidence,
  git,
  inventory,
  diagnostics,
  metrics,
  capture,
  pristine,
  prepareWorkspace,
  installedNative,
  collectTransforms,
} from "./template-alias-paired-11822-custody.mjs";
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const median = (values) => values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];

export async function main(argv = process.argv.slice(2)) {
  assert.equal(process.platform, "linux", "performance campaign is Linux Actions only");
  assert.equal(process.arch, "x64");
  assert.equal(process.env.GITHUB_ACTIONS, "true", "performance campaign is Actions only");
  assert.equal(process.env.TEMPLATE_ALIAS_RUNNER, "blacksmith-32vcpu-ubuntu-2404");
  for (const key of [
    "TEMPLATE_ALIAS_CAPTURE",
    "VIZE_COLD_CAPTURE",
    "VIZE_CHILD_ALLOCATION_OBSERVATION",
    "VIZE_ROUTE_MEMORY_OBSERVATION",
    "VIZE_OWNED_READ_BUFFER_TRACE",
  ]) {
    assert.equal(process.env[key], undefined, `${key} would instrument primary measurements`);
  }
  const { values: options } = parseArgs({
    args: argv,
    options: Object.fromEntries(
      ["base", "head", "base-sha", "head-sha", "upstream", "provider", "output"].map((name) => [
        name,
        { type: "string" },
      ]),
    ),
  });
  for (const name of ["base", "head", "base-sha", "head-sha", "upstream", "provider", "output"])
    assert.ok(options[name], `missing --${name}`);
  const paths = Object.fromEntries(
    ["base", "head", "upstream", "provider", "output"].map((name) => [
      name,
      resolve(options[name]),
    ]),
  );
  const { base, head, upstream, provider, output } = paths;
  mkdirSync(output);
  const sources = {
    base: pristine(base, options["base-sha"]),
    head: pristine(head, options["head-sha"]),
    upstream: pristine(upstream, upstreamSha),
  };
  git(head, "merge-base", "--is-ancestor", options["base-sha"], options["head-sha"]);
  assert.deepEqual(
    readFileSync(join(upstream, "pnpm-lock.yaml")),
    readFileSync(join(provider, "pnpm-lock.yaml")),
  );
  assert.equal(json(join(provider, "node_modules/typescript/package.json")).version, "7.1.0-beta");
  assert.equal(
    json(join(provider, "node_modules/tsrs/package.json")).version,
    "0.21.0-ts7.1.0-beta",
  );
  assert.equal(
    json(join(provider, "node_modules/@vue/content-mapper/package.json")).version,
    "4.0.0-alpha.0",
  );
  assert.equal(json(join(provider, "node_modules/vize/package.json")).version, "0.679.0");
  const providerInputs = inventory(provider);
  write(join(output, "provider-pre.json"), providerInputs);
  const native = {
    tsc: installedNative(
      provider,
      providerInputs,
      "@typescript/typescript-linux-x64",
      "7.1.0-beta",
      ["tsc", "tsgo"],
    ),
    tsrs: installedNative(provider, providerInputs, "@tsrs/linux-x64-gnu", "0.21.0-ts7.1.0-beta", [
      "tsrs",
      "tsgo",
    ]),
    published679: installedNative(
      provider,
      providerInputs,
      "@vizejs/native-linux-x64-gnu",
      "0.679.0",
      ["vize"],
    ),
  };
  const { expectedDiagnostics } = await import(
    pathToFileURL(join(upstream, "scripts/diagnostics.mjs"))
  );
  assert.equal(expectedDiagnostics.length, 12);
  const variants = [
    { id: "published679", package: realpathSync(join(provider, "node_modules/vize")) },
    { id: "currentmain", package: join(base, "npm/cli"), binary: join(base, "target/ci-opt/vize") },
    { id: "candidate", package: join(head, "npm/cli"), binary: join(head, "target/ci-opt/vize") },
    { id: "vue" },
  ];
  const binaries = {};
  const builds = {};
  for (const variant of variants.filter(({ binary }) => binary)) {
    const bytes = readFileSync(variant.binary);
    assert.deepEqual(bytes.subarray(0, 4), Buffer.from([0x7f, 0x45, 0x4c, 0x46]));
    assert.equal(bytes[4], 2);
    assert.equal(bytes[5], 1);
    assert.equal(bytes.readUInt16LE(18), 62);
    binaries[variant.id] = {
      ...evidence(variant.binary),
      elfHeader: bytes.subarray(0, 20).toString("hex"),
    };
    const buildLog = join(
      variant.id === "currentmain" ? base : head,
      "target",
      "build-11822.jsonl",
    );
    const messages = readFileSync(buildLog, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    const artifacts = messages.filter(
      (message) =>
        message.reason === "compiler-artifact" &&
        message.target.name === "vize" &&
        message.executable,
    );
    assert.equal(artifacts.length, 1, "require one attributable native CLI build");
    assert.equal(realpathSync(artifacts[0].executable), realpathSync(variant.binary));
    assert.deepEqual(artifacts[0].features.toSorted(), ["default", "glyph", "maestro"]);
    builds[variant.id] = {
      log: evidence(buildLog),
      artifact: artifacts[0],
      rustc: readFileSync(
        join(variant.id === "currentmain" ? base : head, "target", "rustc-11822.txt"),
        "utf8",
      ),
    };
  }
  assert.deepEqual(builds.currentmain.artifact.profile, builds.candidate.artifact.profile);
  assert.equal(builds.currentmain.rustc, builds.candidate.rustc);
  const inputFiles = inventory(join(upstream, "src"));
  assert.equal(inputFiles.filter(({ path }) => path.endsWith(".vue")).length, 2500);
  cpSync(join(upstream, "src"), join(output, "inputs/src"), { recursive: true });
  for (const name of [
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "tsconfig.base.json",
    "tsconfig.vize-content-mapper.json",
    "tsconfig.vue-content-mapper.json",
    "scripts/diagnostics.mjs",
  ]) {
    mkdirSync(dirname(join(output, "inputs", name)), { recursive: true });
    copyFileSync(join(upstream, name), join(output, "inputs", name));
  }
  for (const variant of variants) {
    const packageRoot =
      variant.package ?? realpathSync(join(provider, "node_modules/@vue/content-mapper"));
    cpSync(packageRoot, join(output, "inputs/packages", variant.id), {
      recursive: true,
      dereference: true,
    });
  }
  for (const name of [
    "template-alias-paired-11822.mjs",
    "template-alias-paired-11822-custody.mjs",
    "template-alias-paired-11822-tee.mjs",
    "template-alias-paired-11822.test.mjs",
  ]) {
    copyFileSync(join(scriptDirectory, name), join(output, "inputs", name));
  }
  copyFileSync(
    join(head, ".github/workflows/template-alias-paired-11822.yml"),
    join(output, "inputs/workflow.yml"),
  );
  for (const [id, binary] of [
    ...Object.entries(native).map(([id, value]) => [id, value.binary.path]),
    ...variants.filter(({ binary }) => binary).map(({ id, binary }) => [id, binary]),
  ]) {
    const workspace = join(output, id + ".version-workspace");
    mkdirSync(workspace);
    const run = await capture(join(output, id + ".version-raw"), binary, ["--version"], workspace);
    assert.equal(run.status, 0, `${id} actual native version probe failed`);
    if (id === "tsc") assert.match(run.text, /7\.1\.0-beta/);
    if (id === "published679") assert.match(run.text, /0\.679\.0/);
    if (native[id])
      native[id].versionProbe = { stdout: run.stdout, stderr: run.stderr, command: run.command };
    else
      binaries[id].versionProbe = { stdout: run.stdout, stderr: run.stderr, command: run.command };
  }
  write(join(output, "provenance.json"), {
    sources,
    binaries,
    builds,
    inputFiles,
    native,
    providerLock: evidence(join(provider, "pnpm-lock.yaml")),
    provider: {
      package: evidence(join(provider, "package.json")),
      modules: evidence(join(provider, "node_modules/.modules.yaml")),
      packages: Object.fromEntries(
        ["typescript", "tsrs", "@vue/content-mapper", "vize", "vue"].map((name) => [
          name,
          {
            manifest: evidence(join(provider, "node_modules", name, "package.json")),
            resolvedRoot: realpathSync(join(provider, "node_modules", name)),
          },
        ]),
      ),
      checkers: Object.fromEntries(
        ["tsc", "tsrs"].map((name) => [
          name,
          evidence(realpathSync(join(provider, "node_modules/.bin", name))),
        ]),
      ),
      published679Measured: true,
    },
    node: { version: process.version, executable: evidence(process.execPath) },
    runner: {
      label: process.env.TEMPLATE_ALIAS_RUNNER,
      name: process.env.RUNNER_NAME,
      environment: process.env.RUNNER_ENVIRONMENT,
      cpus: cpus(),
      release: release(),
      totalmem: totalmem(),
    },
    workflow: {
      sha: process.env.GITHUB_WORKFLOW_SHA,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
    },
    hostEnvironment: Object.fromEntries(
      ["GOMAXPROCS", "RAYON_NUM_THREADS", "NODE_OPTIONS", "CARGO_BUILD_JOBS"].map((key) => [
        key,
        process.env[key] ?? null,
      ]),
    ),
    policy: {
      pairs: 5,
      warmups: 0,
      compilerIncremental: false,
      freshProcessAndWorkspace: true,
      osPageCacheFlushed: false,
      fastFlag: false,
      rss: "GNU time maximum resident set for waited process and descendants; not simultaneous process-tree RSS sum",
      comparison: "Blacksmith paired campaign; not a reproduction of the reporter's M5 Pro times",
      materializedChecker:
        "not executed; mapper directives remain part of the authored diagnostic oracle",
    },
  });
  const observations = [];
  const oracle = new Map();
  for (const checker of ["tsc", "tsrs"]) {
    for (let pair = 0; pair < 5; pair++) {
      const ordered = pair % 2 ? [...variants].reverse() : variants;
      for (const variant of ordered) {
        const id = `${checker}-${variant.id}-${pair}`;
        const workspace = join(output, id + ".workspace");
        const closure = prepareWorkspace(upstream, provider, workspace, variant);
        const run = await capture(
          join(output, id + ".raw"),
          join(provider, "node_modules/.bin", checker),
          [
            "--runExternalCode",
            "--noEmit",
            "--pretty",
            "false",
            "--extendedDiagnostics",
            "-p",
            closure.configFile,
          ],
          workspace,
        );
        assert.deepEqual(
          inventory(join(workspace, "src")),
          closure.authoredInputs,
          "checker/mapper changed authored inputs",
        );
        assert.deepEqual(
          inventory(closure.packageRoot),
          closure.package,
          "checker/mapper changed helper package",
        );
        assert.ok([1, 2].includes(run.status), "require semantic diagnostic exit");
        const packet = {
          status: run.status,
          diagnostics: diagnostics(run.text, workspace, expectedDiagnostics),
        };
        const key = checker + ":" + variant.id;
        if (oracle.has(key))
          assert.deepEqual(
            packet,
            oracle.get(key),
            "diagnostics changed between repeated invocations",
          );
        else oracle.set(key, packet);
        const observation = {
          id,
          checker,
          variant: variant.id,
          pair,
          closure,
          packet,
          resources: run.usage,
          metrics: metrics(run.text),
        };
        observations.push(observation);
        write(join(output, id + ".json"), observation);
        const before = oracle.get(checker + ":currentmain");
        const after = oracle.get(checker + ":candidate");
        if (before && after)
          assert.deepEqual(after, before, "full base/head diagnostic identity changed");
      }
    }
    assert.deepEqual(
      oracle.get(checker + ":candidate"),
      oracle.get(checker + ":currentmain"),
      "full base/head diagnostic identity changed",
    );
  }
  const captures = [];
  for (const checker of ["tsc", "tsrs"]) {
    for (const variant of variants) {
      const id = `${checker}-${variant.id}-capture`;
      const workspace = join(output, id + ".workspace");
      const captureDirectory = join(output, id + ".protocol");
      const closure = prepareWorkspace(upstream, provider, workspace, variant, captureDirectory);
      const run = await capture(
        join(output, id + ".raw"),
        join(provider, "node_modules/.bin", checker),
        [
          "--runExternalCode",
          "--noEmit",
          "--pretty",
          "false",
          "--extendedDiagnostics",
          "-p",
          closure.configFile,
        ],
        workspace,
        { TEMPLATE_ALIAS_CAPTURE: captureDirectory },
      );
      assert.deepEqual(
        inventory(join(workspace, "src")),
        closure.authoredInputs,
        "capture changed authored inputs",
      );
      assert.deepEqual(
        inventory(closure.packageRoot),
        closure.package,
        "capture changed helper package",
      );
      assert.deepEqual(
        { status: run.status, diagnostics: diagnostics(run.text, workspace, expectedDiagnostics) },
        oracle.get(checker + ":" + variant.id),
        "capture changed diagnostic work",
      );
      captures.push({
        id,
        transforms: collectTransforms(
          captureDirectory,
          workspace,
          join(output, id + ".transforms"),
        ),
        metrics: metrics(run.text),
        timingEligible: false,
      });
    }
  }
  const summary = [];
  for (const checker of ["tsc", "tsrs"])
    for (const variant of variants) {
      const cohort = observations.filter(
        (row) => row.checker === checker && row.variant === variant.id,
      );
      summary.push({
        checker,
        variant: variant.id,
        samples: cohort.length,
        medianWallSeconds: median(cohort.map((row) => row.resources.wall_s)),
        medianCpuSeconds: median(
          cohort.map((row) => row.resources.user_s + row.resources.system_s),
        ),
        medianMaxRssKiB: median(cohort.map((row) => row.resources.max_rss_kib)),
        medianMapperWaitSeconds: median(cohort.map((row) => row.metrics.mapperWaitSeconds)),
        medianCheckSeconds: median(cohort.map((row) => row.metrics.checkSeconds)),
        types: cohort.map((row) => row.metrics.types),
        instantiations: cohort.map((row) => row.metrics.instantiations),
      });
    }
  for (const [role, directory] of [
    ["base", base],
    ["head", head],
    ["upstream", upstream],
  ])
    pristine(directory, sources[role].sha);
  assert.deepEqual(
    readFileSync(join(upstream, "pnpm-lock.yaml")),
    readFileSync(join(provider, "pnpm-lock.yaml")),
  );
  const providerAfter = inventory(provider);
  write(join(output, "provider-post.json"), providerAfter);
  assert.deepEqual(providerAfter, providerInputs, "installed transitive provider payload changed");
  const paired = ["tsc", "tsrs"].map((checker) => {
    const ratios = Array.from({ length: 5 }, (_, pair) => {
      const before = observations.find(
        (row) => row.checker === checker && row.variant === "currentmain" && row.pair === pair,
      );
      const after = observations.find(
        (row) => row.checker === checker && row.variant === "candidate" && row.pair === pair,
      );
      return {
        pair,
        wallRatio: after.resources.wall_s / before.resources.wall_s,
        cpuRatio:
          (after.resources.user_s + after.resources.system_s) /
          (before.resources.user_s + before.resources.system_s),
        rssRatio: after.resources.max_rss_kib / before.resources.max_rss_kib,
      };
    });
    return {
      checker,
      ratios,
      medianWallRatio: median(ratios.map((row) => row.wallRatio)),
      medianCpuRatio: median(ratios.map((row) => row.cpuRatio)),
      medianRssRatio: median(ratios.map((row) => row.rssRatio)),
    };
  });
  write(join(output, "summary.json"), {
    summary,
    paired,
    captures,
    fullDiagnosticIdentity: true,
    performanceAcceptance: "requires review of all paired raw observations",
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    const index = process.argv.indexOf("--output");
    const directory = index >= 0 ? process.argv[index + 1] : null;
    if (directory && existsSync(directory)) {
      write(join(directory, "failure.json"), {
        name: error.name,
        message: error.message,
        stack: error.stack,
      });
      const providerIndex = process.argv.indexOf("--provider");
      const provider = providerIndex >= 0 ? process.argv[providerIndex + 1] : null;
      if (provider && existsSync(provider)) {
        write(join(directory, "provider-failure-post.json"), inventory(provider));
      }
    }
    console.error(error);
    process.exitCode = 1;
  });
}
