/** Finite immutable source/provider/Misskey preparation; no ready-stamp shortcut. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { coldScaleSupportScriptFiles, prepareColdScale } from "./cold-typechecker-scale.ts";
import {
  hash,
  identity,
  inventory,
  misskeyRevision,
  providerPins,
  save,
  source,
} from "./typecheck-vs-vue-tsc-frozen-evidence.ts";
export const sourceBodies = [
  {
    file: "Cargo.toml",
    bytes: 13168,
    sha256: "6400cafce2abb1c16bb3c43dfc6498d0a0c695c88849bd5a6bdd4063b91377de",
  },
  {
    file: "Cargo.lock",
    bytes: 128818,
    sha256: "e9a8659e79a919541b6cf29340886dabad787a18fa3051bd77873fc3a76194a4",
  },
  {
    file: "npm/cli/package.json",
    bytes: 4062,
    sha256: "00845d6bef7571d1b554247e0b3fc5808557e910c10b331b63e970fb88919058",
  },
  {
    file: "npm/cli/virtual-ts.d.ts",
    bytes: 15334,
    sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  },
  {
    file: "npm/cli/virtual-ts.d.mts",
    bytes: 15334,
    sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  },
  {
    file: "npm/cli/virtual-ts.d.cts",
    bytes: 15334,
    sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  },
  {
    file: "npm/cli/virtual-ts-globals.d.ts",
    bytes: 18966,
    sha256: "d647737ea035f166c94c131013af3265a3a33783dbad307b2eb3c6d558fc476a",
  },
  {
    file: "npm/cli/virtual-ts-patterns.d.ts",
    bytes: 12522,
    sha256: "e42c2c431553563663e28f75b4e68f8728d928a2867c857c76ef8cf30fe0f545",
  },
  {
    file: "npm/cli/virtual-ts-slots.d.ts",
    bytes: 4134,
    sha256: "3b2f3387f8e720bdffd4ed864b74af6fa19895a89ba6f2f64151fd8dd98bdf9f",
  },
  {
    file: "npm/cli/content-mapper-helpers.d.ts",
    bytes: 1109,
    sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  },
  {
    file: "npm/cli/content-mapper-helpers.d.mts",
    bytes: 1109,
    sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  },
  {
    file: "npm/cli/content-mapper-helpers.d.cts",
    bytes: 1109,
    sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  },
  {
    file: "npm/cli/generated/content-mapper-types.d.ts",
    bytes: 11202,
    sha256: "7457fd86aafb4b2a5b1edbddedf538a45a072517645dd0ece2fd4c074a0f632e",
  },
];
export type Project = {
  id: string;
  root: string;
  cwd: string;
  tsconfig: string;
  control: string;
  sourceInventory: ReturnType<typeof inventory>;
};
export function setup(out: string, name: string, executable: string, args: string[], cwd: string) {
  const r = spawnSync(executable, args, {
    cwd,
    env: { ...process.env, npm_config_engine_strict: "false" },
    timeout: 2_400_000,
    maxBuffer: 128 * 1024 * 1024,
  });
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(path.join(out, name + ".stdout.bin"), r.stdout ?? Buffer.alloc(0), {
    flag: "wx",
  });
  fs.writeFileSync(path.join(out, name + ".stderr.bin"), r.stderr ?? Buffer.alloc(0), {
    flag: "wx",
  });
  save(path.join(out, name + ".json"), {
    executable,
    args,
    cwd,
    status: r.status,
    signal: r.signal,
    error: r.error?.message ?? null,
  });
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  assert.equal(r.status, 0, `setup ${name}; complete raw retained`);
  return (r.stdout ?? Buffer.alloc(0)).toString("utf8").trim();
}
export function verifySource(root: string, provider?: string) {
  assert.equal(
    spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).stdout.trim(),
    source,
  );
  assert.equal(
    spawnSync("git", ["status", "--porcelain", "--untracked-files=no"], {
      cwd: root,
      encoding: "utf8",
    }).stdout,
    "",
  );
  const bodies = sourceBodies.map((pin) => {
    const actual = identity(path.join(root, pin.file));
    assert.equal(actual.bytes, pin.bytes);
    assert.equal(actual.sha256, pin.sha256);
    return { ...pin, path: actual.path };
  });
  const lock = fs.readFileSync(path.join(root, "Cargo.lock"), "utf8");
  const names = [
    "corsa",
    "corsa_client",
    "corsa_core",
    "corsa_jsonrpc",
    "corsa_lsp",
    "corsa_orchestrator",
    "corsa_runtime",
  ];
  const sdkRoles = names.map((name) => {
    const section = lock
      .split("[[package]]")
      .find((block) => block.startsWith(`\nname = "${name}"\n`));
    assert(section);
    assert.match(section, /\nversion = "2\.0\.0-beta\.4"\n/u);
    const checksum = /\nchecksum = "([a-f0-9]{64})"\n/u.exec(section);
    assert(checksum);
    return { name, version: "2.0.0-beta.4", checksum: checksum[1] };
  });
  const edges: { file: string; specifier: string; target: string }[] = [];
  if (provider) {
    const ts = createRequire(path.join(provider, "package.json"))("typescript") as {
      version: string;
      preProcessFile(
        text: string,
        imports: boolean,
        js: boolean,
      ): { referencedFiles: unknown[]; importedFiles: { fileName: string }[] };
    };
    assert.equal(ts.version, "6.0.3");
    const declarations = sourceBodies.filter(
      (row) => row.file.startsWith("npm/cli/") && row.file !== "npm/cli/package.json",
    );
    for (const declaration of declarations) {
      const parsed = ts.preProcessFile(
        fs.readFileSync(path.join(root, declaration.file), "utf8"),
        true,
        true,
      );
      assert.equal(parsed.referencedFiles.length, 0);
      for (const ref of parsed.importedFiles.filter((row) => row.fileName.startsWith("."))) {
        assert(ref.fileName.endsWith(".js"));
        const target = path.posix.normalize(
          path.posix.join(
            path.posix.dirname(declaration.file),
            ref.fileName.slice(0, -3) + ".d.ts",
          ),
        );
        assert(declarations.some((row) => row.file === target));
        edges.push({ file: declaration.file, specifier: ref.fileName, target });
      }
    }
    assert.equal(edges.length, 13);
  }
  return {
    source,
    bodies,
    sdkRoles,
    edges,
    sdkScope:
      "whole Cargo files/seven locked SDK4 roles and ten declaration bodies; source identity, not native loaded-library membership",
  };
}
export function verifyProvider(provider: string) {
  assert.equal(hash(fs.readFileSync(path.join(provider, "package.json"))), providerPins.package);
  assert.equal(hash(fs.readFileSync(path.join(provider, "package-lock.json"))), providerPins.lock);
  const require = createRequire(path.join(provider, "package.json"));
  const versions = Object.fromEntries(
    ["vue", "vue-tsc", "typescript", "typescript-native", "vize", "@vizejs/native"].map((name) => [
      name,
      require(name + "/package.json").version,
    ]),
  );
  assert.deepEqual(versions, {
    vue: "3.6.0-rc.13",
    "vue-tsc": "3.3.13",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    vize: "0.596.0",
    "@vizejs/native": "0.596.0",
  });
  const nativeRequire = createRequire(require.resolve("typescript-native/package.json"));
  const native = path.join(
    path.dirname(nativeRequire.resolve("@typescript/typescript-linux-x64/package.json")),
    "lib/tsc",
  );
  assert.equal(identity(native).sha256, providerPins.native);
  const lock = JSON.parse(fs.readFileSync(path.join(provider, "package-lock.json"), "utf8"));
  const packages = Object.entries(
    lock.packages as Record<string, { version?: string; integrity?: string; resolved?: string }>,
  ).map(([name, row]) => ({ name, ...row }));
  assert(
    packages
      .filter((p) => p.name)
      .every((p) => typeof p.integrity === "string" && p.integrity.startsWith("sha512-")),
    "every resolved provider package retains SRI",
  );
  return {
    versions,
    packages,
    native: identity(native),
    nativeAssets: inventory(path.dirname(native)),
    node: identity(process.execPath),
    engine: "Vize native Go7.1beta/a183 versus vue-tsc JavaScript TS6.0.3; cross-engine comparison",
  };
}
export function prepareProjects(out: string, provider: string) {
  const root = path.join(out, "synthetic-project");
  const corpus = prepareColdScale(root, provider, 2500 + coldScaleSupportScriptFiles, false);
  assert.equal(corpus.files, 4323);
  assert.equal(corpus.sfcFiles, 2500);
  save(path.join(out, "synthetic-corpus.json"), corpus);
  const synthetic: Project = {
    id: "retained-2500",
    root,
    cwd: corpus.app,
    tsconfig: "tsconfig.json",
    control: path.join(corpus.app, "src/__FrozenComparisonControl.vue"),
    sourceInventory: inventory(root, ["node_modules", ".git"]),
  };
  const misskey = path.join(out, "misskey");
  fs.mkdirSync(misskey);
  const logs = path.join(out, "setup");
  setup(logs, "misskey-init", "git", ["init", "-q"], misskey);
  setup(
    logs,
    "misskey-remote",
    "git",
    ["remote", "add", "origin", "https://github.com/misskey-dev/misskey"],
    misskey,
  );
  setup(
    logs,
    "misskey-fetch",
    "git",
    ["fetch", "--no-tags", "--depth=1", "origin", misskeyRevision],
    misskey,
  );
  setup(logs, "misskey-checkout", "git", ["checkout", "--detach", misskeyRevision], misskey);
  assert.equal(setup(logs, "misskey-head", "git", ["rev-parse", "HEAD"], misskey), misskeyRevision);
  const lockBefore = identity(path.join(misskey, "pnpm-lock.yaml"));
  const manager = ["--yes", "pnpm@10.30.1"];
  assert.equal(setup(logs, "pnpm-version", "npx", [...manager, "--version"], misskey), "10.30.1");
  setup(
    logs,
    "misskey-install",
    "npx",
    [...manager, "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    misskey,
  );
  setup(
    logs,
    "misskey-prepare",
    "npx",
    [
      ...manager,
      "--filter",
      "misskey-js",
      "--filter",
      "misskey-reversi",
      "--filter",
      "misskey-bubble-game",
      "--filter",
      "i18n",
      "exec",
      "tsgo",
      "--project",
      "tsconfig.json",
      "--outDir",
      "built",
      "--declaration",
      "true",
      "--emitDeclarationOnly",
      "true",
    ],
    misskey,
  );
  assert.equal(identity(path.join(misskey, "pnpm-lock.yaml")).sha256, lockBefore.sha256);
  assert.equal(
    setup(logs, "misskey-tracked-clean", "git", ["diff", "--name-only", "HEAD"], misskey),
    "",
  );
  const localRequire = createRequire(path.join(misskey, "packages/frontend/package.json"));
  const vue = identity(localRequire.resolve("vue/package.json"));
  save(path.join(out, "misskey-source.json"), {
    revision: misskeyRevision,
    lock: lockBefore,
    package: identity(path.join(misskey, "package.json")),
    config: identity(path.join(misskey, "packages/frontend/tsconfig.json")),
    installedVue: { ...vue, version: JSON.parse(fs.readFileSync(vue.path, "utf8")).version },
    dependencyInventory: inventory(path.join(misskey, "node_modules")),
    packageManager: "pnpm10.30.1 via pinned npx; project lock/SRI retained",
    preparationEngine: "project-locked tsgo; setup only, not timed incumbent",
  });
  const real: Project = {
    id: "misskey-810faa",
    root: misskey,
    cwd: misskey,
    tsconfig: "packages/frontend/tsconfig.json",
    control: path.join(misskey, "packages/frontend/src/__FrozenComparisonControl.vue"),
    sourceInventory: inventory(misskey, ["node_modules", ".git"]),
  };
  for (const project of [synthetic, real])
    save(path.join(out, project.id + "-source.json"), project);
  return [synthetic, real];
}

/** TS6's existing config parser owns this finite output-path fact; no cache clearing. */
export function cacheFacts(project: Project, provider: string) {
  const ts = createRequire(path.join(provider, "package.json"))("typescript") as {
    sys: { readFile(path: string): string | undefined };
    readConfigFile(
      path: string,
      read: (path: string) => string | undefined,
    ): { config: unknown; error?: unknown };
    parseJsonConfigFileContent(
      config: unknown,
      host: unknown,
      base: string,
      options: undefined,
      file: string,
    ): {
      options: Record<string, unknown>;
      errors: { code: number; messageText: unknown; start?: number; length?: number }[];
    };
    getTsBuildInfoEmitOutputFilePath(options: Record<string, unknown>): string | undefined;
  };
  const config = path.resolve(project.cwd, project.tsconfig),
    read = ts.readConfigFile(config, ts.sys.readFile);
  const parsed = ts.parseJsonConfigFileContent(
    read.config,
    ts.sys,
    path.dirname(config),
    undefined,
    config,
  );
  const errors = parsed.errors.map((row) => ({
    code: row.code,
    messageText: row.messageText,
    start: row.start,
    length: row.length,
  }));
  const buildInfo = ts.getTsBuildInfoEmitOutputFilePath(parsed.options);
  const output = buildInfo ? path.resolve(buildInfo) : null;
  const owned =
    output &&
    output.startsWith(project.root + path.sep) &&
    (parsed.options.incremental === true || parsed.options.composite === true);
  return {
    config: identity(config),
    options: parsed.options,
    errors,
    readError: read.error ?? null,
    declaredBuildInfo: output,
    excludedOwnedOutputs: !read.error && errors.length === 0 && owned ? [output] : [],
    scope:
      "TS6 resolved output paths only; Vize effective native incremental outputs remain separately observed; no cold-cache assertion",
  };
}
export function inputSnapshot(projectRoot: string, provider: string) {
  return { project: inventory(projectRoot, [".git"]), provider: inventory(provider) };
}
export function assertInputs(
  before: ReturnType<typeof inputSnapshot>,
  after: typeof before,
  ownedCacheOutputs: string[],
) {
  const rows = (snapshot: typeof before, role: "project" | "provider") =>
    snapshot[role].rows.filter((row) => {
      const file = path.resolve(snapshot[role].root, String(row.path));
      if (!ownedCacheOutputs.includes(file)) return true;
      assert.equal(role, "project");
      assert.equal(row.kind, "file", "owned cache output cannot mask a symlink/nonregular input");
      return false;
    });
  for (const role of ["project", "provider"] as const) {
    assert.equal(before[role].root, after[role].root);
    assert.equal(
      hash(JSON.stringify(rows(before, role))),
      hash(JSON.stringify(rows(after, role))),
      `${role} full source/config/dependency/engine input identity changed; full snapshots retained`,
    );
  }
}
