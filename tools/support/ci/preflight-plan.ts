// Plans the local PR preflight (tools/commands/ci/preflight.ts) from the same
// selectors CI uses, so a local pass means the deterministic PR gates pass:
//
// - .github/workflows/check.yml: fmt-rust, check-js, check-vize-apps, level deps
// - .github/workflows/pr-source-checks.yml: plan-pr-checks.ts (Rust/contracts/
//   tooling flags), plan-tooling-tests.mjs (affected tooling tests), CI
//   selection tests, the Clippy test-target ratchet hold
// - .github/workflows/pr-rust-checks.yml: Clippy and the contracts lane
// - .github/workflows/pr-rust-products.yml: Rust tests of the touched crates
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { planCheckInputs } from "../compat/github/plan-check-inputs.ts";
import { planPrChecks } from "../compat/github/plan-pr-checks.ts";
import { planAffectedRust, readCargoMetadata } from "../compat/github/plan-affected-rust.ts";
import { readJsWorkspace } from "../compat/github/plan-affected-js.ts";
import { planToolingTests } from "../compat/github/plan-tooling-tests.mjs";
import { touchedRatchetCrates } from "../compat/github/clippy-test-targets.ts";
import { workflowCatalogMatchesMerge } from "./preflight-merge.ts";

export type Step = {
  id: string;
  title: string;
  command?: string[];
  env?: Record<string, string>;
  cwd?: string;
  /** Needs the pinned Rust toolchain; reproduce it inside `nix develop`. */
  toolchain?: boolean;
  timeoutSeconds?: number;
  hint?: string;
  repro?: string;
  run?: (log: (line: string) => void) => Promise<boolean>;
};

export type Lane = {
  name: string;
  steps: Step[];
  note?: string;
  sequential?: boolean;
  stopOnFailure?: boolean;
  concurrency?: number;
};

export type PreflightOptions = {
  base: string;
  committed: boolean;
  jobs: number;
  toolingJobs: number;
  only: Set<string> | undefined;
  skip: Set<string>;
  rustScope: "touched" | "affected";
  failFast: boolean;
  list: boolean;
  help: boolean;
  nix: boolean | undefined;
  timeoutSeconds: number;
  tailLines: number;
  install: boolean;
  native: boolean;
};

export const LANES = ["gates", "node-tests", "rust", "apps"] as const;

export const usage = `Usage: node tools/commands/ci/preflight.ts [options]

Runs the deterministic PR checks CI would select for this branch's diff:
  gates       merge conflicts, rustfmt, vp check (oxfmt/oxlint), 500-line ratchet,
              workflow catalog (HEAD and merged with the base), tool layout,
              workspace lints, skeleton TODOs, Clippy ratchet hold, level deps
  node-tests  CI's planner-selected affected tooling tests (plan-tooling-tests.mjs),
              the contracts lane (assertion lint, typecheck divergence, ...),
              CI selection, check-js and node-engine tests: one node --test per file
  rust        cargo clippy / cargo test (dev profile, incl. doctests, custody pins
              and insta snapshots) for the touched crates, plus the conditional
              Patina/Glyph rustdoc and L1/legacy feature lanes
  apps        current-source vize CLI: opinionated source lint and check:ci:vize-apps

Options:
  --base <ref>          comparison base (default: origin/main; uses the merge base)
  --committed           only committed changes (default also includes the working tree)
  --only <lanes>        comma list of lanes to run (${LANES.join(", ")})
  --skip <lanes>        comma list of lanes to skip
  --rust-scope <s>      touched (default) or affected (also the reverse-dependency crates)
  --no-native           skip tooling tests that need the native addon build
  --jobs <n>            parallel gate steps (default: CPU count)
  --tooling-jobs <n>    parallel tooling test files (default: CPU count / 2)
  --timeout <s>         per-step timeout in seconds (default: 900, 0 = none)
  --fail-fast           stop scheduling new steps after the first failure
  --no-install          never run vp install / install rust-script
  --no-nix              do not re-enter \`nix develop\` (use the current toolchain)
  --list                print the selected steps and their commands, run nothing
  --tail <n>            log lines shown per failure (default: 40)
`;

export function parseOptions(argv: readonly string[]): PreflightOptions {
  const cpus = os.availableParallelism();
  const options: PreflightOptions = {
    base: "origin/main",
    committed: false,
    jobs: cpus,
    toolingJobs: Math.max(2, Math.floor(cpus / 2)),
    only: undefined,
    skip: new Set(),
    rustScope: "touched",
    failFast: false,
    list: false,
    help: false,
    nix: undefined,
    timeoutSeconds: 900,
    tailLines: 40,
    install: true,
    native: true,
  };
  const lanes = (value: string | undefined) => {
    const names = (value ?? "").split(",").filter(Boolean);
    for (const name of names)
      if (!(LANES as readonly string[]).includes(name)) throw new Error(`unknown lane: ${name}`);
    return new Set(names);
  };
  const number = (value: string | undefined, flag: string) => {
    const parsed = Number(value);
    if (!Number.isInteger(parsed) || parsed < 0) throw new Error(`${flag} expects an integer`);
    return parsed;
  };
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    const value = () => argv[++index];
    if (argument === "--base") options.base = value() ?? options.base;
    else if (argument === "--committed") options.committed = true;
    else if (argument === "--only") options.only = lanes(value());
    else if (argument === "--skip") options.skip = lanes(value());
    else if (argument === "--rust-scope") {
      const scope = value();
      if (scope !== "touched" && scope !== "affected")
        throw new Error("--rust-scope: touched|affected");
      options.rustScope = scope;
    } else if (argument === "--no-native") options.native = false;
    else if (argument === "--jobs") options.jobs = Math.max(1, number(value(), argument));
    else if (argument === "--tooling-jobs")
      options.toolingJobs = Math.max(1, number(value(), argument));
    else if (argument === "--timeout") options.timeoutSeconds = number(value(), argument);
    else if (argument === "--fail-fast") options.failFast = true;
    else if (argument === "--no-install") options.install = false;
    else if (argument === "--no-nix") options.nix = false;
    else if (argument === "--list" || argument === "--dry-run") options.list = true;
    else if (argument === "--tail") options.tailLines = number(value(), argument);
    else if (argument === "--help" || argument === "-h") options.help = true;
    else throw new Error(`unknown option: ${argument}\n\n${usage}`);
  }
  return options;
}

const git = (root: string, ...args: string[]) =>
  execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 });
const nulList = (output: string) => output.split("\0").filter(Boolean);

/** The PR diff CI plans against (merge base to HEAD), plus local edits unless --committed. */
export function changedFiles(root: string, mergeBase: string, committed: boolean): string[] {
  const filter = ["--no-renames", "--name-only", "--diff-filter=ACDMRT", "-z"];
  const paths = new Set(nulList(git(root, "diff", ...filter, mergeBase, "HEAD")));
  if (!committed) {
    for (const file of nulList(git(root, "diff", ...filter, mergeBase))) paths.add(file);
    for (const file of nulList(git(root, "ls-files", "--others", "--exclude-standard", "-z")))
      paths.add(file);
  }
  return [...paths].sort();
}

const RUST_SCRIPT_VERSION = "0.34.0";

function rustScriptSetup(options: PreflightOptions): Step | undefined {
  const probe = spawnSync("rust-script", ["--version"], { encoding: "utf8" });
  if (probe.status === 0 && probe.stdout.includes(RUST_SCRIPT_VERSION)) return;
  const cache = path.join(
    process.env.XDG_CACHE_HOME ?? path.join(os.homedir(), ".cache"),
    "vize-preflight",
    `rust-script-${RUST_SCRIPT_VERSION}`,
  );
  // Children inherit this PATH, so tests that shell out to rust-script find it.
  process.env.PATH = `${path.join(cache, "bin")}${path.delimiter}${process.env.PATH}`;
  if (fs.existsSync(path.join(cache, "bin", "rust-script"))) return;
  if (!options.install) return;
  return {
    id: "setup:rust-script",
    title: `Install the pinned rust-script ${RUST_SCRIPT_VERSION} (CI's setup-rust-script) into ${cache}`,
    command: [
      "cargo",
      "install",
      "rust-script",
      "--version",
      RUST_SCRIPT_VERSION,
      "--locked",
      "--root",
      cache,
    ],
    toolchain: true,
  };
}

function jsInstallSetup(root: string, options: PreflightOptions): Step | undefined {
  const lock = fs.statSync(path.join(root, "pnpm-lock.yaml"));
  const installed = fs.statSync(path.join(root, "node_modules", ".modules.yaml"), {
    throwIfNoEntry: false,
  });
  const current = installed !== undefined && installed.mtimeMs >= lock.mtimeMs;
  if (current || !options.install) return;
  return {
    id: "setup:vp-install",
    title: "Install JS dependencies (node_modules is missing or older than pnpm-lock.yaml)",
    command: ["vp", "install", "--frozen-lockfile", "--prefer-offline"],
  };
}

function nodeTest(file: string, why: string, env: Record<string, string>): Step {
  return {
    id: `test:${file.replace(/^tests\/tooling\//u, "")}`,
    title: `${file} (${why})`,
    command: ["node", "--test", "--test-concurrency=1", file],
    // Only the length ratchet tests read the comparison base.
    env: file.includes("source-file-lengths") ? env : undefined,
  };
}

export type PreflightPlan = {
  nix: boolean;
  setup: Step[];
  lanes: Lane[];
  summary: string[];
  unmirrored: string[];
};

export function planPreflight(root: string, options: PreflightOptions): PreflightPlan {
  const enabled = (lane: string) =>
    (!options.only || options.only.has(lane)) && !options.skip.has(lane);
  const head = git(root, "rev-parse", "HEAD").trim();
  const base = git(root, "rev-parse", "--verify", `${options.base}^{commit}`).trim();
  const mergeBase = git(root, "merge-base", base, head).trim();
  const paths = changedFiles(root, mergeBase, options.committed);
  const metadata = readCargoMetadata(root);
  const pr = planPrChecks({
    paths,
    eventName: "pull_request",
    labels: [],
    workspace: readJsWorkspace(root),
    cargoMetadata: metadata,
  });
  const affectedRust = planAffectedRust(metadata, paths, "pull_request");
  const inputs = planCheckInputs(paths);
  const tooling = planToolingTests(paths, { tier: "pr", cwd: root });
  const nix = Boolean(process.env.IN_NIX_SHELL);
  const unmirrored: string[] = [];
  const summary = [
    `Preflight ${head.slice(0, 12)} against ${options.base} (merge base ${mergeBase.slice(0, 12)})`,
    `Changed paths: ${paths.length}${options.committed ? " (committed only)" : " (committed + working tree)"}`,
    `Rust: touched ${affectedRust.changedPackages.join(", ") || "none"}; affected ${affectedRust.scope} (${affectedRust.packages.length} crates)`,
    `Tooling tests: ${tooling.tests.length}/${tooling.totalTests} selected (${tooling.reason}); contracts lane: ${pr.sourceContracts}; CI selection tests: ${pr.source.tooling}`,
    `Check inputs: apps=${inputs.apps} engines=${inputs.engines} dependencies=${inputs.dependencies}`,
  ];

  // ---- gates: every PR, seconds each -------------------------------------
  const gates: Step[] = [];
  if (head !== mergeBase || paths.length)
    gates.push({
      id: "gate:merge-conflicts",
      title: `Merge ${options.base} into HEAD without conflicts (CI tests the merge commit)`,
      command: ["git", "merge-tree", "--write-tree", "--name-only", "--no-messages", base, head],
      hint: `rebase onto ${options.base} and resolve; generated files (workflow README) must be regenerated, not hand-merged`,
    });
  gates.push({
    id: "gate:fmt-rust",
    title: "fmt-rust: cargo fmt --all -- --check",
    command: ["cargo", "fmt", "--all", "--", "--check"],
    toolchain: true,
    hint: "cargo fmt --all",
  });
  // check-js runs `vp run check:iteration`: the workspace's own vp over the
  // whole repo (oxfmt + oxlint, ~15s locally) under the zero-warning budget.
  gates.push({
    id: "gate:check-js",
    title: "check-js: vp check (oxfmt + oxlint) over the repo, zero-warning budget",
    command: [
      "node",
      "tools/commands/ci/check-warning-budget.ts",
      "--",
      "./node_modules/.bin/vp",
      "check",
    ],
    hint: "./node_modules/.bin/vp check --fix (or vp fmt <files>)",
  });
  gates.push(
    {
      id: "gate:source-file-lengths",
      title: "500-line source file ratchet against the merge base",
      command: [
        "node",
        "tools/commands/ci/source-file-lengths.ts",
        "--check",
        "--base-ref",
        mergeBase,
      ],
      hint: "split the file (move tests beside the module, extract submodules); the ratchet never grows",
    },
    {
      id: "gate:workflow-catalog",
      title: ".github/workflows/README.md matches the generated workflow catalog",
      command: ["node", "tools/commands/ci/workflow-catalog.ts", "--check"],
      hint: "node tools/commands/ci/workflow-catalog.ts",
    },
    {
      id: "gate:workflow-catalog-merged",
      title: `workflow catalog is still fresh after merging ${options.base}`,
      repro: `git merge ${options.base} && node tools/commands/ci/workflow-catalog.ts --check`,
      hint: `rebase onto ${options.base}, then regenerate with node tools/commands/ci/workflow-catalog.ts`,
      run: async (log) => workflowCatalogMatchesMerge(root, base, head, log),
    },
    {
      id: "gate:tool-layout",
      title: "tools/commands layout (verify-tool-layout.ts)",
      command: ["node", "tools/commands/ci/verify-tool-layout.ts"],
    },
    {
      id: "gate:workspace-lints",
      title: "every crate inherits [lints] workspace = true",
      command: ["node", "tools/commands/ci/check-workspace-lints.ts", "--check"],
    },
    {
      id: "gate:skeleton-todos",
      title: "skeleton TODO ledger",
      command: ["node", "tools/commands/ci/check-skeleton-todos.ts", "--check"],
    },
    {
      id: "gate:clippy-test-target-hold",
      title: "Clippy test-target ratchet list only grows",
      command: ["node", "tools/support/compat/github/clippy-test-targets.ts", "hold", mergeBase],
    },
    {
      id: "gate:level-dependencies",
      title: "Davinci level dependency direction ratchet",
      command: ["node", "tools/support/compat/davinci/level-dependencies.ts", "--base", mergeBase],
      toolchain: true,
    },
  );

  // ---- node tests: one process per file ------------------------------------
  const selected = new Map<string, string>();
  const add = (files: readonly string[], why: string) => {
    for (const file of files)
      if (!selected.has(file) && fs.existsSync(path.join(root, file))) selected.set(file, why);
  };
  add(tooling.tests, "affected tooling");
  add(
    [
      "tests/tooling/github-workflows-source-selection.test.ts",
      "tests/tooling/vp-bootstrap-cache.test.mjs",
    ],
    "check-js",
  );
  add(
    [
      "tests/tooling/davinci-level-dependencies.test.mjs",
      "tests/tooling/davinci-directory-dependencies.test.ts",
    ],
    "level dependencies",
  );
  if (pr.sourceContracts) add(contractTests(root), "contracts lane");
  if (pr.source.tooling) add(ciSelectionTests(root), "CI selection");
  if (inputs.engines)
    add(
      [
        "node-engine-matrix",
        "package-manifests",
        "bundler-plugin-manifests",
        "dependency-security-compat",
        "comparison-base",
        "plan-source-checks",
        "plan-check-inputs",
        "warning-budget",
        "npm-audit-command",
      ].map((name) => `tests/tooling/${name}.test.ts`),
      "node-engine-compat",
    );
  // Shared by every tooling test, as in CI's affected-tooling worker.
  process.env.VIZE_TOOLING_TEST_TIER = "pr";
  process.env.NODE_OPTIONS = "--disable-warning=DEP0040 --disable-warning=DEP0169";
  const testEnv = { SOURCE_LENGTH_BASE_REF: mergeBase };
  const native = new Set<string>(tooling.nativeBuildTests);
  const plain: Step[] = [];
  const nativeSteps: Step[] = [];
  for (const [file, why] of selected) {
    if (!native.has(file)) plain.push(nodeTest(file, why, testEnv));
    else if (options.native) nativeSteps.push(nodeTest(file, `${why}, native addon`, testEnv));
  }
  if (native.size && !options.native)
    unmirrored.push(`${native.size} tooling test(s) that need the native addon (--no-native)`);

  // ---- rust: touched crates, dev profile -----------------------------------
  const crates =
    options.rustScope === "affected" && affectedRust.scope !== "workspace"
      ? affectedRust.packages
      : affectedRust.changedPackages;
  const rust: Step[] = [];
  const packageArgs = (names: readonly string[]) => names.flatMap((name) => ["-p", name]);
  if (crates.length) {
    rust.push({
      id: "rust:clippy",
      title: `cargo clippy ${crates.join(" ")} -D warnings (CI: Clippy affected crates)`,
      command: [
        "cargo",
        "clippy",
        "--locked",
        ...packageArgs(crates),
        "--",
        "-D",
        "warnings",
        "-D",
        "clippy::wildcard_imports",
      ],
      toolchain: true,
    });
    const ratchet = touchedRatchetCrates(
      { changedPackages: crates },
      JSON.parse(fs.readFileSync(path.join(root, "tools/config/clippy-test-targets.json"), "utf8"))
        .crates,
    );
    if (ratchet.length)
      rust.push({
        id: "rust:clippy-test-targets",
        title: `cargo clippy --all-targets for touched ratchet crates ${ratchet.join(" ")}`,
        command: [
          "cargo",
          "clippy",
          "--locked",
          ...packageArgs(ratchet),
          "--all-targets",
          "--",
          "-D",
          "warnings",
        ],
        toolchain: true,
      });
    const typechecker = crates.some((name) => name === "vize_canon" || name === "vize_croquis_cf");
    const tsc = path.join(
      root,
      "node_modules/@typescript",
      `typescript-${process.platform}-${process.arch}`,
      "lib/tsc",
    );
    const tsgoEnv: Record<string, string> =
      typechecker && fs.existsSync(tsc)
        ? { VIZE_TEST_REQUIRE_TSGO: "1", CORSA_PATH: tsc }
        : { VIZE_TEST_DISABLE_TSGO: "1" };
    rust.push({
      id: "rust:test",
      title: `cargo test ${crates.join(" ")} (unit, integration, doctests, custody pins, insta snapshots)`,
      command: ["cargo", "test", "--locked", ...packageArgs(crates)],
      // CI: insta never writes in CI and every mismatch fails.
      env: {
        INSTA_UPDATE: "no",
        INSTA_FORCE_PASS: "0",
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
        ...tsgoEnv,
      },
      toolchain: true,
      hint: "snapshot drift: cargo insta test -p <crate> --review; custody/sha256 pins: rerun the crate's pin regeneration and commit it",
    });
    rust.push(...conditionalRustSteps(crates));
  } else if (affectedRust.scope === "workspace") {
    unmirrored.push(
      `workspace-wide Rust tests (${affectedRust.reasons.join("; ")}); try --rust-scope affected is not narrower, run cargo test --workspace yourself`,
    );
  }
  if (affectedRust.scope === "affected" && options.rustScope === "touched") {
    const dependents = affectedRust.packages.filter((name: string) => !crates.includes(name));
    if (dependents.length)
      unmirrored.push(
        `Rust tests/Clippy of ${dependents.length} reverse-dependency crate(s) (--rust-scope affected runs them)`,
      );
  }

  // ---- apps: current-source CLI ------------------------------------------
  const apps: Step[] = [];
  if (inputs.apps) {
    const vize = path.join(root, "target/debug/vize");
    apps.push(
      {
        id: "apps:build-vize",
        title: "Build the current-source vize CLI (dev profile)",
        command: ["cargo", "build", "--locked", "-p", "vize"],
        toolchain: true,
      },
      {
        id: "apps:opinionated-source-lint",
        title: "Lint authored npm/docs/playground/example sources with the opinionated preset",
        command: ["node", "tools/commands/ci/opinionated-source-lint.ts"],
        env: { VIZE_BIN: vize },
      },
      {
        id: "apps:pack-config-apis",
        title: "Pack npm/cli and npm/builder/vite (check:ci:vize-apps imports them)",
        command: ["sh", "-c", "(cd npm/cli && vp pack) && (cd npm/builder/vite && vp pack)"],
      },
      {
        id: "apps:check-vize-apps",
        title:
          "check:ci:vize-apps (vize check/lint + vp check on examples/vite-musea and playground)",
        command: ["vp", "run", "--workspace-root", "check:ci:vize-apps"],
        env: { PATH: `${path.dirname(vize)}${path.delimiter}${process.env.PATH}` },
        repro: `cargo build -p vize && PATH="$PWD/target/debug:$PATH" vp run --workspace-root check:ci:vize-apps`,
      },
    );
  }

  unmirrored.push(...notMirrored(pr, inputs));
  const setup = [
    enabled("node-tests") || enabled("apps") || enabled("gates")
      ? jsInstallSetup(root, options)
      : undefined,
    enabled("node-tests") && selected.size ? rustScriptSetup(options) : undefined,
  ].filter((step): step is Step => step !== undefined);
  const nativeBuild: Step = {
    id: "node-tests:build-native",
    title: "Build the native addon for tooling tests that load it (build:native:test)",
    command: ["vp", "run", "--workspace-root", "build:native:test"],
    toolchain: true,
  };
  const lanes: Lane[] = [
    { name: "gates", steps: gates },
    { name: "node-tests", steps: plain, concurrency: options.toolingJobs },
    {
      name: "native-tests",
      steps: nativeSteps.length ? [nativeBuild, ...nativeSteps] : [],
      sequential: true,
      stopOnFailure: true,
    },
    {
      name: "rust",
      steps: rust,
      sequential: true,
      note: crates.length ? crates.join(", ") : "no touched crates",
    },
    {
      name: "apps",
      steps: apps,
      sequential: true,
      stopOnFailure: true,
      note: inputs.apps ? undefined : "no app inputs",
    },
  ].filter((lane) => enabled(lane.name === "native-tests" ? "node-tests" : lane.name));
  return { nix, setup, lanes, summary, unmirrored };
}

function contractTests(root: string): string[] {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/pr-rust-checks.yml"), "utf8");
  const step =
    workflow.split("Require current-tree source, API and reviewed-doc contracts")[1] ?? "";
  const block = step.split("- name:")[0];
  return [...block.matchAll(/tests\/tooling\/[\w.-]+\.test\.m?[jt]s/gu)].map(([file]) => file);
}

function ciSelectionTests(root: string): string[] {
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/pr-source-checks.yml"),
    "utf8",
  );
  const block = (workflow.split("Test CI selection when its inputs change")[1] ?? "").split(
    "- name:",
  )[0];
  const files = new Set<string>();
  for (const [pattern] of block.matchAll(/tests\/tooling\/[\w.*-]+\.test\.m?[jt]s/gu)) {
    if (!pattern.includes("*")) files.add(pattern);
    else
      for (const file of fs.readdirSync(path.join(root, "tests/tooling")))
        if (path.matchesGlob(`tests/tooling/${file}`, pattern)) files.add(`tests/tooling/${file}`);
  }
  return [...files];
}

function conditionalRustSteps(crates: readonly string[]): Step[] {
  const has = (name: string) => crates.includes(name);
  const steps: Step[] = [];
  const cargo = (id: string, title: string, args: string[]): Step => ({
    id,
    title,
    command: ["cargo", ...args],
    toolchain: true,
  });
  if (has("vize_l1") || has("vize_armature"))
    steps.push(
      cargo("rust:l1-markup-parity", "Native L1 markup parity", [
        "test",
        "--locked",
        "-p",
        "vize_armature",
        "--features",
        "native-lex-parity,legacy",
        "--lib",
        "--test",
        "markup_lex_parity",
      ]),
    );
  if (has("vize_atelier_core"))
    steps.push(
      cargo("rust:legacy-transforms", "Shared legacy transform capabilities", [
        "test",
        "--locked",
        "-p",
        "vize_atelier_core",
        "--features",
        "legacy",
        "--lib",
        "legacy",
      ]),
    );
  if (has("vize_l1"))
    steps.push(
      cargo("rust:l1-surface-corpus", "L1 feature-enabled surface corpus", [
        "test",
        "--locked",
        "-p",
        "vize_l1",
        "--features",
        "legacy-differential",
        "--test",
        "surface_corpus",
      ]),
    );
  if (has("vize_glyph"))
    steps.push(
      cargo("rust:glyph-docs", "Glyph public API documentation", [
        "rustdoc",
        "--locked",
        "-p",
        "vize_glyph",
        "--lib",
        "--",
        "-D",
        "missing-docs",
      ]),
    );
  if (has("vize_patina"))
    steps.push(
      cargo("rust:patina-docs", "Patina public API documentation", [
        "rustdoc",
        "--locked",
        "-p",
        "vize_patina",
        "--lib",
        "--",
        "-D",
        "warnings",
      ]),
    );
  return steps;
}

function notMirrored(
  pr: ReturnType<typeof planPrChecks>,
  inputs: ReturnType<typeof planCheckInputs>,
): string[] {
  const lines: string[] = [];
  const lanes = pr.native.matrix.lane.filter((lane: string) => lane !== "none");
  if (lanes.length)
    lines.push(`native runtime lanes (${lanes.join(", ")}): source CLI + real Vue/Vite runtimes`);
  if (pr.js.groups.length)
    lines.push(
      `PR JS package lanes (${pr.js.groups.join(", ")}): package builds/tests via pr-js-checks.yml`,
    );
  if (pr.rust.packages.includes("vize_vitrine"))
    lines.push("WASM release build check (vize_vitrine)");
  if (inputs.dependencies)
    lines.push("security-audit (cargo audit / npm audit need the advisory network)");
  if (pr.rust.packages.includes("vize_maestro"))
    lines.push("native Program/Vue navigation lane (vize_maestro)");
  lines.push(
    "nextest cohort filters and the ci profile (local runs use cargo test with the dev profile)",
    "legacy crate deletion guard and legacy-fix fixture report (need the GitHub API / PR title)",
    "merge-queue-only gates: differential corpora, instruction ceilings, canonical corpus, Nuxt module, VRT",
  );
  return lines;
}
