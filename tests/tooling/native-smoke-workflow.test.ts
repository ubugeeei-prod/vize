import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { parse as parseToml } from "@iarna/toml";
import { runCiBuild } from "../../npm/native/scripts/build-ci.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function workflowJobBody(workflow: string, jobName: string): string {
  const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
  assert.notEqual(jobStart, -1, `${jobName} job missing`);
  const remaining = workflow.slice(jobStart + 1);
  const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
  return remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
}

test("native smoke keeps MoonBit out of the npm fresh install path", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const hostJob = workflowJobBody(workflow, "host-native-smoke");
  const freshJob = workflowJobBody(workflow, "fresh-install-smoke");

  assert.match(hostJob, /target:\s*darwin-x64/);
  assert.doesNotMatch(hostJob, /setup-moonbit|moon (?:run|test)/);
  assert.match(freshJob, /target:\s*darwin-x64/);
  assert.doesNotMatch(freshJob, /setup-moonbit/);
  assert.match(freshJob, /vp run --filter '\.\/npm\/cli' build/);
  // Installed-package assertions import the shared semver test helper. The
  // root owns that dependency; selecting only npm package closures misses it.
  const parsed = parse(workflow) as {
    jobs: Record<string, { steps: Array<{ run?: string }> }>;
  };
  const install = parsed.jobs["fresh-install-smoke"].steps
    .map((step) =>
      step.run
        ?.replace(/\\\r?\n/gu, " ")
        .replace(/\s+/gu, " ")
        .trim(),
    )
    .find((command) => command?.startsWith("vp install "));
  assert.ok(install, "fresh smoke must install its helper dependencies");
  assert.match(install, /(?:^| )--filter \.(?: |$)/u);
});

test("one Linux Node 24 row runs the packed Nuxt LSP release graph", () => {
  const native = parse(readRepoFile(".github", "workflows", "native-smoke.yml")) as {
    env: Record<string, string>;
    jobs: Record<
      string,
      { steps: Array<{ name?: string; if?: string; run?: string; uses?: string }> }
    >;
  };
  const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
    env: Record<string, string>;
  };
  assert.equal(native.env.RELEASE_JS_PACKAGES, release.env.RELEASE_JS_PACKAGES);

  const steps = native.jobs["fresh-install-smoke"].steps;
  const selected =
    "${{ matrix.platform.target == 'linux-x64-gnu' && matrix.node-version == '24' }}";
  const other = "${{ matrix.platform.target != 'linux-x64-gnu' || matrix.node-version != '24' }}";
  const fullBuild = steps.find((step) => step.name === "Build the packed Nuxt release graph");
  assert.deepEqual(fullBuild, {
    name: "Build the packed Nuxt release graph",
    if: selected,
    uses: "./.github/actions/release-build-packages",
  });
  assert.equal(steps.find((step) => step.name === "Build npm runtime packages")?.if, other);
  assert.equal(
    steps.find((step) => step.name === "Fresh install and runtime smoke published tarballs")?.if,
    other,
  );
  const nuxt = steps.find((step) => step.name === "Fresh packed Nuxt init, check, and LSP smoke");
  assert.equal(nuxt?.if, selected);
  assert.match(nuxt?.run ?? "", /read -ra js <<< "\$RELEASE_JS_PACKAGES"/u);
  assert.match(nuxt?.run ?? "", /npm\/native npm\/native\/npm\/\* "\$\{js\[@\]\}"/u);
});

const selected =
  "github.event_name == 'workflow_dispatch' && inputs.host_only && inputs.target == 'darwin-arm64'";
const workflowFile = () => readRepoFile(".github", "workflows", "native-smoke.yml");
const workflowObject = () => parse(workflowFile());
const evaluate = (
  expression: string,
  event: string,
  hostOnly: boolean | undefined,
  target: string | undefined,
) =>
  Boolean(
    runInNewContext(expression, {
      github: { event_name: event },
      inputs: { host_only: hostOnly, target },
    }),
  );

// #8840 moved the token laws from MoonBit to Node, so the selected producer
// no longer installs MoonBit. Reverse that and the earlier predicate change.
const nodeLaws = "        run: node --test tests/tooling/native-build-token.test.ts\n";
const moonLaws =
  "        working-directory: tools/moon\n" +
  "        run: moon test --target native --deny-warn cmd/github/build_napi_package\n";
const msvcArch = "          arch: ${{ matrix.msvc_arch }}\n";
const originalSetup =
  "      # MoonBit's native installer does not support Darwin x64. This matrix\n" +
  "      # target only builds and smokes the Rust/N-API native package, so keep\n" +
  "      # darwin-x64 covered without installing MoonBit.\n" +
  "      - uses: ./.github/actions/setup-moonbit\n" +
  "        if: matrix.target != 'darwin-x64'\n";

function authenticateWorkflow(source: string) {
  for (const anchor of [nodeLaws, msvcArch]) assert.equal(source.split(anchor).length, 2);
  assert.doesNotMatch(source, /setup-moonbit|moon (?:run|test)/u);
  const reversed = source.replace(nodeLaws, moonLaws).replace(msvcArch, msvcArch + originalSetup);
  // Authenticated whole workflow, including the single packed Nuxt row; all
  // inputs, commands, pins, outputs and other conditions stay exact.
  const originalDigest = "89a0cdf0739297cbe3f5ed596fc1c81d2d8b1ada3eb2647e36b74427117a1ad1";
  assert.equal(createHash("sha256").update(reversed).digest("hex"), originalDigest);
}

void test("the complete workflow retains the original setup and token-law provenance", () => {
  const source = workflowFile();
  authenticateWorkflow(source);
  for (const removed of [
    "cargo build --profile ci -p vize",
    "--max 2.36",
    "--runtime-checks",
    "node --test tests/tooling/native-build-token.test.ts",
  ])
    assert.throws(() => authenticateWorkflow(source.replace(removed, "removed-command")), {
      code: "ERR_ASSERTION",
    });
});

void test("only the real selected producer runs the token laws while default qualification remains eighteen rows", () => {
  const { on, jobs } = workflowObject();
  assert.deepEqual(Object.keys(on), ["workflow_dispatch", "schedule"]);
  assert.equal(on.workflow_dispatch.inputs.host_only.default, false);
  assert.equal(on.workflow_dispatch.inputs.target.default, "all");
  assert.deepEqual(on.workflow_dispatch.inputs.target.options, ["all", "darwin-arm64"]);
  const host = jobs["host-native-smoke"];
  const fresh = jobs["fresh-install-smoke"];
  const producer = jobs["selected-host-native-producer"];
  assert.equal(
    producer.steps,
    host.steps,
    "selected producer must retain the actual shared step anchor",
  );
  assert.equal(
    host.steps.some((step: { uses?: string }) => step.uses === "./.github/actions/setup-moonbit"),
    false,
  );
  const tokenLaws = host.steps.find(
    (step: { name?: string }) => step.name === "Test native image token producer",
  );
  assert.equal(tokenLaws.if, selected);
  assert.equal(tokenLaws.run, "node --test tests/tooling/native-build-token.test.ts");
  assert.equal(tokenLaws["working-directory"], undefined);
  const targets = [
    "linux-x64-gnu",
    "linux-arm64-gnu",
    "darwin-x64",
    "darwin-arm64",
    "win32-x64-msvc",
    "win32-arm64-msvc",
  ];
  assert.deepEqual(
    host.strategy.matrix.include.map((row: { target: string }) => row.target),
    targets,
  );
  assert.deepEqual(
    fresh.strategy.matrix.platform.map((row: { target: string }) => row.target),
    targets,
  );
  assert.deepEqual(fresh.strategy.matrix["node-version"], ["22", "24"]);
  assert.equal(host.strategy["fail-fast"], false);
  assert.equal(fresh.strategy["fail-fast"], false);
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const hostOnly of [undefined, false, true])
      for (const target of [undefined, "all", ...targets]) {
        const expectedSelected =
          event === "workflow_dispatch" && hostOnly === true && target === "darwin-arm64";
        assert.equal(evaluate(tokenLaws.if, event, hostOnly, target), expectedSelected);
        assert.equal(evaluate(producer.if, event, hostOnly, target), expectedSelected);
        assert.equal(evaluate(host.if, event, hostOnly, target), !expectedSelected);
        assert.equal(evaluate(fresh.if, event, hostOnly, target), !expectedSelected);
        if (!expectedSelected)
          assert.equal(
            host.strategy.matrix.include.length +
              fresh.strategy.matrix.platform.length * fresh.strategy.matrix["node-version"].length,
            18,
          );
      }
  // Removing unused setup also removes its Windows amd64 override: caller ARM64 setup stays authoritative.
  assert.equal(
    host.steps.find((step: { name?: string }) => step.name === "Setup MSVC toolchain (Windows)")
      .with.arch,
    "${{ matrix.msvc_arch }}",
  );
  assert.equal(
    host.strategy.matrix.include.find(
      (row: { target: string }) => row.target === "win32-arm64-msvc",
    ).msvc_arch,
    "arm64",
  );
});

void test("the actual default native producer executes only its unchanged NAPI and Node recipes", () => {
  const commands: Array<{ command: string; args: string[]; shell: boolean; cwd: string }> = [];
  const nativeDir = path.join(root, "npm/native");
  const controlledRun = ((
    command: string,
    args: string[],
    options: { shell: boolean; cwd: string },
  ) => {
    commands.push({ command, args: [...args], shell: options.shell, cwd: options.cwd });
    return { status: 0 };
  }) as typeof spawnSync;
  assert.equal(runCiBuild(nativeDir, { capture: false, run: controlledRun }), 0);
  assert.deepEqual(commands, [
    {
      command: "napi",
      args: [
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
      ],
      shell: process.platform === "win32",
      cwd: nativeDir,
    },
    {
      command: process.execPath,
      args: [path.join(nativeDir, "scripts/sync-entrypoint.mjs")],
      shell: false,
      cwd: nativeDir,
    },
  ]);
  const closure = new Set<string>();
  const visit = (file: string) => {
    if (closure.has(file)) return;
    closure.add(file);
    const source = readRepoFile(file);
    assert.doesNotMatch(source, /\bmoon(?:c|bit)?\b|MOON_(?:HOME|BIN)/u, file);
    for (const match of source.matchAll(/\b(?:from\s*|import\s*\(?\s*)["']([^"']+)["']/gu))
      if (match[1].startsWith("."))
        visit(path.posix.normalize(path.posix.join(path.posix.dirname(file), match[1])));
  };
  visit("npm/native/scripts/build-ci.mjs");
  visit("npm/native/scripts/sync-entrypoint.mjs");
  assert.ok(closure.has("npm/native/scripts/formatter-history-ci-build.mjs"));
  assert.ok(closure.has("npm/native/scripts/native-build-environment.mjs"));
  for (const file of [
    "crates/vize_vitrine/build.rs",
    "crates/vize_fresco/build.rs",
    "tools/commands/ci/github/verify-glibc-symbols.rs",
    "tools/commands/release/npm/smoke-release-install.rs",
    "tools/support/common.rs",
    "tools/commands/release/npm/npm_publish.rs",
    "tools/commands/release/npm/npm_smoke_init.rs",
    ".github/actions/setup-rust-script/action.yml",
    ".github/actions/setup-rust-sticky-cache/action.yml",
    ".github/actions/setup-runtime-package-managers/action.yml",
  ])
    assert.doesNotMatch(readRepoFile(file), /\bmoon(?:c|bit)?\b|MOON_(?:HOME|BIN)/u, file);
  const selectedLaws = readRepoFile("tests/tooling/native-build-token.test.ts");
  assert.equal([...selectedLaws.matchAll(/^void test\("/gmu)].length, 3);
  for (const law of [
    "native build token preserves every byte including zero-padded nibbles",
    "actual platform entropy yields fresh 128-bit native build tokens",
    "production build environment preserves inheritance and overrides old tokens",
  ])
    assert.ok(selectedLaws.includes(`void test("${law}"`));
});

void test("all transitive local normal and build dependencies keep Moon out of default native builds", () => {
  type Dependency = { path?: string; workspace?: boolean } | string;
  type Tables = {
    dependencies?: Record<string, Dependency>;
    "build-dependencies"?: Record<string, Dependency>;
  };
  type Manifest = Tables & {
    package?: { name: string; build?: string | boolean };
    target?: Record<string, Tables>;
    workspace?: { dependencies: Record<string, Dependency> };
    patch?: Record<string, Record<string, Dependency>>;
  };
  const workspace = parseToml(readRepoFile("Cargo.toml")) as Manifest;
  const inherited = workspace.workspace!.dependencies;
  const visited = new Set<string>();
  const pending = ["crates/vize/Cargo.toml", "crates/vize_vitrine/Cargo.toml"];
  // Registry and Git dependencies can enter repository source through Cargo patch overrides.
  for (const patches of Object.values(workspace.patch ?? {}))
    for (const dependency of Object.values(patches))
      if (typeof dependency !== "string" && dependency.path)
        pending.push(path.posix.normalize(path.posix.join(dependency.path, "Cargo.toml")));
  while (pending.length) {
    const file = pending.pop()!;
    if (visited.has(file)) continue;
    visited.add(file);
    const manifest = parseToml(readRepoFile(file)) as Manifest;
    assert.notEqual(manifest.package!.name, "vize_dialect_moonbit", file);
    const build = manifest.package!.build;
    const buildFile = path.posix.join(
      path.posix.dirname(file),
      typeof build === "string" ? build : "build.rs",
    );
    if (build !== false && fs.existsSync(path.join(root, buildFile)))
      assert.doesNotMatch(
        readRepoFile(buildFile),
        /\bmoon(?:c|bit)?\b|MOON_(?:HOME|BIN)/u,
        buildFile,
      );
    // Conservatively include optional dependencies and every target; dev-only oracles are not build dependencies.
    for (const tables of [manifest, ...Object.values(manifest.target ?? {})])
      for (const section of ["dependencies", "build-dependencies"] as const)
        for (const [name, declaration] of Object.entries(tables[section] ?? {})) {
          if (typeof declaration === "string") continue;
          const dependency = declaration.workspace ? inherited[name] : declaration;
          if (typeof dependency === "string" || !dependency.path) continue;
          const directory = declaration.workspace ? "." : path.posix.dirname(file);
          pending.push(
            path.posix.normalize(path.posix.join(directory, dependency.path, "Cargo.toml")),
          );
        }
  }
  assert.ok(visited.has("crates/vize_fresco/Cargo.toml"));
  assert.ok(visited.has("vendor/oxc_parser_compat/Cargo.toml"));
});
