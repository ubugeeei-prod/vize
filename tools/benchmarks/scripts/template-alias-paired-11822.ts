import { assertCampaignEnvironment } from "./template-alias-paired-11822-process.ts";
import type {
  Variant,
  NativeReceipt,
  ProbeRecord,
  ProbeIdentity,
  DiagnosticPacket,
  WorkspaceClosure,
  VariantId,
  Metrics,
  Resources,
  Location,
} from "./template-alias-paired-11822-types.ts";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, realpathSync } from "node:fs";
import { availableParallelism } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import {
  upstreamSha,
  json,
  write,
  git,
  inventory,
  diagnostics,
  metrics,
  capture,
  pristine,
  prepareWorkspace,
  installedNative,
  closureSnapshot,
} from "./template-alias-paired-11822-custody.ts";
import {
  archiveCampaignInputs,
  requiredDependencyDirectory,
} from "./template-alias-paired-11822-packages.ts";
import {
  verifiedSourcePair,
  writeCampaignProvenance,
} from "./template-alias-paired-11822-source.ts";
import {
  declarationParser,
  declarationClosure,
} from "./template-alias-paired-11822-declarations.ts";
import { collectTransforms, sharedHelperClosure } from "./template-alias-paired-11822-protocol.ts";
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const median = (values: number[]) =>
  values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];

export async function main(argv = process.argv.slice(2)) {
  assertCampaignEnvironment();
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
  const option = (name: string): string => {
    const value = options[name];
    assert.ok(typeof value === "string", "missing --" + name);
    return value;
  };
  const paths = Object.fromEntries(
    ["base", "head", "upstream", "provider", "output"].map((name) => [name, resolve(option(name))]),
  );
  const { base, head, upstream, provider, output } = paths;
  const typescript = declarationParser(
    join(requiredDependencyDirectory(join(head, "npm/cli"), "typescript"), "lib/typescript.js"),
  );
  assert.equal(
    process.env.EXECUTED_WORKFLOW_SHA,
    option("head-sha"),
    "executed workflow differs from the approved source head",
  );
  mkdirSync(output);
  const sources = {
    base: pristine(base, option("base-sha")),
    head: pristine(head, option("head-sha")),
    upstream: pristine(upstream, upstreamSha),
  };
  git(head, "merge-base", "--is-ancestor", option("base-sha"), option("head-sha"));
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
  const native: Record<string, NativeReceipt> = {
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
      ["vize-vitrine.linux-x64-gnu.node"],
      "napi",
    ),
  };
  const { expectedDiagnostics } = (await import(
    pathToFileURL(join(upstream, "scripts/diagnostics.mjs")).href
  )) as { expectedDiagnostics: Location[] };
  assert.equal(expectedDiagnostics.length, 12);
  const builds = verifiedSourcePair(base, head, sources);
  const variants: Variant[] = [
    { id: "published679", package: realpathSync(join(provider, "node_modules/vize")) },
    { id: "currentmain", package: builds.currentmain.cliPackage },
    { id: "candidate", package: builds.candidate.cliPackage },
    { id: "vue" },
  ];
  const roots = [
    provider,
    ...Object.values(builds).map((build) => dirname(dirname(build.cliPackage))),
  ];
  const files = [
    process.execPath,
    ...Object.values(native).map((entry) => entry.binary.path),
    ...Object.values(builds).map((build) => build.originalImage.path),
  ];
  const frozenClosure = closureSnapshot(roots, files);
  write(join(output, "execution-closure-pre.json"), frozenClosure);
  const width = () => ({
    availableParallelism: availableParallelism(),
    affinity: readFileSync("/proc/self/status", "utf8")
      .split("\n")
      .find((line) => line.startsWith("Cpus_allowed_list:")),
    quota: existsSync("/sys/fs/cgroup/cpu.max")
      ? readFileSync("/sys/fs/cgroup/cpu.max", "utf8")
      : null,
    cpuset: existsSync("/sys/fs/cgroup/cpuset.cpus.effective")
      ? readFileSync("/sys/fs/cgroup/cpuset.cpus.effective", "utf8")
      : null,
  });
  const cpuWidth = width();
  const observe = async (
    id: string,
    command: string,
    args: string[],
    cwd: string,
    env: NodeJS.ProcessEnv = {},
  ) => {
    const before = closureSnapshot(roots, files);
    write(join(output, id + ".custody-pre.json"), before);
    assert.deepEqual(
      before,
      frozenClosure,
      "execution image/launcher/provider changed before invocation",
    );
    assert.deepEqual(width(), cpuWidth, "effective CPU width changed before invocation");
    try {
      return await capture(join(output, id + ".raw"), command, args, cwd, env);
    } finally {
      const after = closureSnapshot(roots, files);
      write(join(output, id + ".custody-post.json"), after);
      assert.deepEqual(
        after,
        before,
        "execution image/launcher/provider changed during invocation",
      );
      assert.deepEqual(width(), cpuWidth, "effective CPU width changed during invocation");
    }
  };
  const inputFiles = inventory(join(upstream, "src"));
  assert.equal(inputFiles.filter(({ path }) => path.endsWith(".vue")).length, 2500);
  archiveCampaignInputs({ upstream, output, provider, variants, builds, scriptDirectory, head });
  const probes: Record<string, ProbeRecord> = {};
  for (const id of ["tsc", "tsrs", "published679", "currentmain", "candidate"] as const) {
    const workspace = join(output, id + ".version-workspace");
    mkdirSync(workspace);
    const variant = variants.find(
      (variant): variant is Exclude<Variant, { id: "vue" }> =>
        variant.id !== "vue" && variant.id === id,
    );
    const command = variant ? process.execPath : native[id].binary.path;
    const args = variant
      ? [join(scriptDirectory, "template-alias-paired-11822-probe.ts"), variant.package]
      : ["--version"];
    const run = await observe(id + ".version", command, args, workspace);
    assert.equal(run.status, 0, id + " native/wrapper version probe failed");
    if (id === "tsc") assert.match(run.text, /7\.1\.0-beta/);
    if (id === "published679") assert.match(run.text, /0\.679\.0/);
    let identity = null;
    if (variant) {
      const line = /^TEMPLATE_ALIAS_NATIVE_IDENTITY (.+)$/m.exec(run.text);
      assert.ok(line, "normal Node/NAPI probe identity absent");
      identity = JSON.parse(line[1]) as ProbeIdentity;
      const sourceBuild = id === "currentmain" || id === "candidate" ? builds[id] : undefined;
      const expected = id === "published679" ? native.published679.binary : sourceBuild?.image;
      assert.ok(expected);
      assert.ok(
        identity.images.some(
          (image) => image.path === realpathSync(expected.path) && image.sha256 === expected.sha256,
        ),
      );
      assert.ok(
        identity.sharedObjects.includes(realpathSync(expected.path)),
        "expected NAPI image was not actually mapped",
      );
      if (sourceBuild) assert.equal(identity.token, sourceBuild.token);
    }
    probes[id] = {
      stdout: run.stdout,
      stderr: run.stderr,
      command: run.command,
      identity,
      nativeDeclarations: variant
        ? declarationClosure(
            requiredDependencyDirectory(variant.package, "@vizejs/native"),
            typescript,
          )
        : null,
    };
  }
  writeCampaignProvenance({
    output,
    provider,
    sources,
    probes,
    cpuWidth,
    builds,
    inputFiles,
    native,
  });
  const observations: {
    id: string;
    checker: string;
    variant: VariantId;
    pair: number;
    closure: WorkspaceClosure;
    packet: DiagnosticPacket;
    resources: Resources;
    metrics: Metrics;
  }[] = [];
  const oracle = new Map<string, DiagnosticPacket>();
  const captures: {
    id: string;
    transforms: ReturnType<typeof collectTransforms>;
    metrics: Metrics;
    timingEligible: boolean;
  }[] = [];
  const expectedPaths = inputFiles
    .filter(({ path }) => path.endsWith(".vue"))
    .map(({ path }) => "src/" + path);
  for (const checker of ["tsc", "tsrs"]) {
    for (const variant of variants) {
      const id = `${checker}-${variant.id}-capture`;
      const workspace = join(output, id + ".workspace");
      const captureDirectory = join(output, id + ".protocol");
      const closure = prepareWorkspace(upstream, provider, workspace, variant, captureDirectory);
      const run = await observe(
        id,
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
      assert.ok(run.status === 1 || run.status === 2, "require semantic diagnostic exit");
      oracle.set(checker + ":" + variant.id, {
        status: run.status,
        diagnostics: diagnostics(run.text, workspace, expectedDiagnostics),
      });
      captures.push({
        id,
        transforms: collectTransforms(
          captureDirectory,
          workspace,
          join(output, id + ".transforms"),
          expectedPaths,
          variant.id === "vue" ? null : sharedHelperClosure(workspace, typescript),
        ),
        metrics: metrics(run.text),
        timingEligible: false,
      });
    }
  }
  for (const checker of ["tsc", "tsrs"]) {
    assert.deepEqual(
      oracle.get(checker + ":candidate"),
      oracle.get(checker + ":currentmain"),
      "preflight full base/head diagnostic identity changed",
    );
    const before = captures.find((entry) => entry.id === checker + "-currentmain-capture");
    const after = captures.find((entry) => entry.id === checker + "-candidate-capture");
    assert.ok(before && after);
    assert.deepEqual(
      after.transforms.directiveAuthority,
      before.transforms.directiveAuthority,
      "authored directive authority changed",
    );
  }
  for (const checker of ["tsc", "tsrs"]) {
    for (let pair = 0; pair < 5; pair++) {
      const ordered = pair % 2 ? [...variants].reverse() : variants;
      for (const variant of ordered) {
        const id = `${checker}-${variant.id}-${pair}`;
        const workspace = join(output, id + ".workspace");
        const closure = prepareWorkspace(upstream, provider, workspace, variant);
        const run = await observe(
          id,
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
        assert.ok(run.status === 1 || run.status === 2, "require semantic diagnostic exit");
        assert.ok(
          run.usage.wall_s > 0 &&
            run.usage.user_s + run.usage.system_s > 0 &&
            run.usage.max_rss_kib > 0,
          "primary resource denominators must be positive",
        );
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
        medianCheckerMemoryKiB: median(cohort.map((row) => row.metrics.checkerMemoryKiB)),
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
  ] as const)
    pristine(directory, sources[role].sha);
  assert.deepEqual(
    readFileSync(join(upstream, "pnpm-lock.yaml")),
    readFileSync(join(provider, "pnpm-lock.yaml")),
  );
  write(join(output, "execution-closure-post.json"), closureSnapshot(roots, files));
  assert.deepEqual(closureSnapshot(roots, files), frozenClosure);
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
      assert.ok(before && after, "incomplete paired population");
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
