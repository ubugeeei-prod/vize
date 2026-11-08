/**
 * The checkers the scaling lane times, how each is invoked on a single
 * project and on a project-references monorepo, and the pinned binaries they
 * resolve to.
 *
 * - `vize check` runs from the built `vize` package (the release N-API build
 *   of the same `vize::cli` entry as the Rust binary) with the target
 *   checkout's own pinned tsgo (`corsa-runtime` catalog). On a monorepo it
 *   gets the solution-style root tsconfig, which it expands into one program
 *   per referenced project; it has no build mode.
 * - The TypeScript 7.1 hosts load the built Vize mapper or Vue's mapper.
 * - vue-tsc is the version pinned for the README benchmark, on its own
 *   TypeScript.
 *
 * A Nuxt project takes its packages from the npm-locked tools/benchmarks/nuxt-app,
 * gets `nuxt prepare` before any check, and adds a `nuxt build` lane. Its
 * single-project checkers take `.nuxt/tsconfig.app.json`; vize check takes the
 * solution root as a user runs it.
 */

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { availableParallelism } from "node:os";
import { dirname, join } from "node:path";

import { fileSha256 } from "./benchmark-binary.mjs";
import { linkPackage } from "./install-package-files.ts";
import { NUXT_APP_DIR } from "./nuxt-bench-app.ts";
import type { ScaleKind } from "./scale-project-generator.ts";
import { NUXT_APP_TSCONFIG } from "./scale-project-nuxt.ts";
import {
  CONTENT_MAPPERS,
  type MapperHosts,
  TYPECHECK_HOSTS_DIR,
  mapperTsconfigName,
  prepareMapperPackages,
  resolveMapperHosts,
} from "./typecheck-mapper-hosts.ts";

export type ToolId =
  | "vize-check"
  | `vize-check-servers-${1 | 2 | 4 | 8}`
  | "tsc-vize-mapper"
  | "tsrs-fast-vize-mapper"
  | "tsrs-fast-parallel-vize-mapper"
  | "vue-tsc"
  | "tsc-vue-mapper"
  | "nuxt-build"
  | "nuxt-build-vize";
export type ScalingTool = {
  id: ToolId;
  label: string;
  /** A Vize curve: superlinear growth is filed as a p0 issue. */
  vize: boolean;
  format: "json" | "text" | "build";
};
export const TOOLS: readonly ScalingTool[] = [
  { id: "vize-check", label: "vize check", vize: true, format: "json" },
  { id: "tsc-vize-mapper", label: "tsc 7.1 + Vize mapper", vize: true, format: "text" },
  { id: "tsrs-fast-vize-mapper", label: "tsrs --fast + Vize mapper", vize: true, format: "text" },
  {
    id: "tsrs-fast-parallel-vize-mapper",
    label: "tsrs --fast --checkers=max + Vize mapper",
    vize: true,
    format: "text",
  },
  { id: "vue-tsc", label: "vue-tsc", vize: false, format: "text" },
  { id: "tsc-vue-mapper", label: "tsc 7.1 + Vue mapper", vize: false, format: "text" },
];

const CORSA_SERVER_COUNTS = [1, 2, 4, 8] as const;

/** Focused native topology lanes selected explicitly by benchmark dispatches. */
export const CORSA_TOPOLOGY_TOOLS: readonly ScalingTool[] = CORSA_SERVER_COUNTS.map(
  (servers): ScalingTool => ({
    id: `vize-check-servers-${servers}`,
    label: `vize check (--servers ${servers}, --checkers 1)`,
    vize: true,
    format: "json",
  }),
);

/** Every selectable lane; the default scaling surface remains [`TOOLS`]. */
export const ALL_TOOLS: readonly ScalingTool[] = [...TOOLS, ...CORSA_TOPOLOGY_TOOLS];
/** Timed on Nuxt projects only; gated on a built client entry instead of diagnostics. */
export const NUXT_BUILD_VIZE: ScalingTool = {
  id: "nuxt-build-vize",
  label: "nuxt build + @vizejs/nuxt",
  vize: true,
  format: "build",
};
export const NUXT_BUILD: ScalingTool = {
  id: "nuxt-build",
  label: "nuxt build",
  vize: false,
  format: "build",
};

export type ScalingEnv = {
  target: string;
  node: string;
  vizeCli: string;
  corsa: string;
  vueTsc: string;
  vueDir: string;
  hosts: MapperHosts;
  /** tools/benchmarks/nuxt-app/node_modules when installed. */
  nuxtModules: string | null;
  /** The target's built @vizejs/nuxt, or null when it is not built. */
  vizeNuxtDir: string | null;
};
export type Invocation = { bin: string; args: string[]; stats: boolean };

const vueMapper = CONTENT_MAPPERS.find((mapper) => mapper.id === "vue");
const vizeMapper = CONTENT_MAPPERS.find((mapper) => mapper.id === "vize");

function must(path: string, what: string): string {
  if (!existsSync(path)) throw new Error(`scaling lane: ${what} not found at ${path}`);
  return path;
}

/** Binaries of one built target checkout and the driver's pinned hosts. */
export function resolveScalingEnv(target: string, hostsDir = TYPECHECK_HOSTS_DIR): ScalingEnv {
  const cliDir = join(target, "npm", "cli");
  const benchModules = join(target, "tools", "benchmarks", "scripts", "node_modules");
  const vueTscDir = realpathSync(must(join(benchModules, "vue-tsc"), "vue-tsc"));
  return {
    target,
    node: process.execPath,
    vizeCli: must(join(cliDir, "bin", "vize"), "the vize package launcher"),
    corsa: must(
      join(
        target,
        "node_modules",
        "@typescript",
        `typescript-${process.platform}-${process.arch}`,
        "lib",
        process.platform === "win32" ? "tsc.exe" : "tsc",
      ),
      "the pinned tsgo for vize check",
    ),
    vueTsc: must(join(vueTscDir, "bin", "vue-tsc.js"), "vue-tsc's bin"),
    vueDir: realpathSync(must(join(benchModules, "vue"), "vue")),
    hosts: resolveMapperHosts(hostsDir, cliDir),
    vizeNuxtDir: existsSync(join(target, "npm", "framework", "nuxt", "dist"))
      ? join(target, "npm", "framework", "nuxt")
      : null,
    nuxtModules: existsSync(join(NUXT_APP_DIR, "node_modules", "nuxt"))
      ? join(NUXT_APP_DIR, "node_modules")
      : null,
  };
}

export const nuxtCli = (env: ScalingEnv): string =>
  must(
    join(env.nuxtModules ?? join(NUXT_APP_DIR, "node_modules"), "nuxt", "bin", "nuxt.mjs"),
    "nuxt (npm ci --prefix tools/benchmarks/nuxt-app)",
  );

/** Link every package of `source` into `modules`, scopes as real directories. */
function linkAll(source: string, modules: string, skip: (entry: string) => boolean): void {
  mkdirSync(modules, { recursive: true });
  for (const entry of readdirSync(source)) {
    if (entry.startsWith(".") || skip(entry)) continue;
    if (entry.startsWith("@")) {
      for (const scoped of readdirSync(join(source, entry))) {
        linkPackage(realpathSync(join(source, entry, scoped)), join(modules, entry, scoped));
      }
    } else if (!existsSync(join(modules, entry))) {
      symlinkSync(realpathSync(join(source, entry)), join(modules, entry), "dir");
    }
  }
}

/**
 * `vue` with the packages pnpm placed beside it (a Nuxt project: the whole
 * Nuxt install, linked package by package so build caches stay in the
 * project), then both mappers and the 7.1 `typescript`.
 */
export function prepareProjectPackages(dir: string, env: ScalingEnv, kind: ScaleKind): void {
  const modules = join(dir, "node_modules");
  if (kind === "nuxt") {
    linkAll(
      must(env.nuxtModules ?? "", "tools/benchmarks/nuxt-app/node_modules"),
      modules,
      (entry) => entry === "typescript",
    );
  } else {
    linkAll(dirname(env.vueDir), modules, (entry) => entry === "typescript");
  }
  prepareMapperPackages(dir, env.hosts);
  if (kind !== "nuxt") return;
  for (const mapper of CONTENT_MAPPERS) {
    writeFileSync(
      join(dir, mapperTsconfigName(mapper)),
      `${JSON.stringify({ extends: `./${NUXT_APP_TSCONFIG}`, contentMappers: [{ package: mapper.package, extensions: [".vue"] }] }, null, 2)}\n`,
    );
  }
}

export function invocation(
  tool: ScalingTool,
  env: ScalingEnv,
  cwd: string,
  kind: ScaleKind,
): Invocation {
  const build = kind === "monorepo";
  const host = (bin: string, config: string, extra: string[] = []): Invocation => ({
    bin,
    args: build
      ? ["-b", join(cwd, config), "--force", ...extra, "--runExternalCode", "--pretty", "false"]
      : [
          "--noEmit",
          ...extra,
          "--runExternalCode",
          "--extendedDiagnostics",
          "--pretty",
          "false",
          "-p",
          join(cwd, config),
        ],
    stats: !build,
  });
  if (!vueMapper || !vizeMapper) throw new Error("content mapper table changed");
  if (tool.id.startsWith("vize-check-servers-")) {
    const servers = tool.id.slice("vize-check-servers-".length);
    return {
      bin: env.node,
      args: [
        env.vizeCli,
        "check",
        "--format",
        "json",
        "--quiet",
        "--tsconfig",
        join(cwd, "tsconfig.json"),
        "--corsa-path",
        env.corsa,
        "--servers",
        servers,
        "--checkers",
        "1",
      ],
      stats: false,
    };
  }
  switch (tool.id) {
    case "vize-check":
      return {
        bin: env.node,
        args: [
          env.vizeCli,
          "check",
          "--format",
          "json",
          "--quiet",
          "--tsconfig",
          join(cwd, "tsconfig.json"),
          "--corsa-path",
          env.corsa,
        ],
        stats: false,
      };
    case "tsc-vize-mapper":
      return host(env.hosts.tsc, mapperTsconfigName(vizeMapper));
    case "tsrs-fast-vize-mapper":
      return host(env.hosts.tsrs, mapperTsconfigName(vizeMapper), ["--fast"]);
    case "tsrs-fast-parallel-vize-mapper":
      return host(env.hosts.tsrs, mapperTsconfigName(vizeMapper), [
        "--fast",
        "--checkers",
        String(availableParallelism()),
      ]);
    case "tsc-vue-mapper":
      return host(env.hosts.tsc, mapperTsconfigName(vueMapper));
    case "vue-tsc":
      return {
        bin: env.node,
        args: build
          ? [env.vueTsc, "-b", join(cwd, "tsconfig.json"), "--force", "--pretty", "false"]
          : [
              env.vueTsc,
              "--noEmit",
              "--extendedDiagnostics",
              "--pretty",
              "false",
              "-p",
              join(cwd, kind === "nuxt" ? NUXT_APP_TSCONFIG : "tsconfig.json"),
            ],
        stats: !build,
      };
    case "nuxt-build":
    case "nuxt-build-vize":
      if (kind !== "nuxt") throw new Error("nuxt build runs on Nuxt projects only");
      return { bin: env.node, args: [nuxtCli(env), "build", cwd], stats: false };
  }
  throw new Error(`unsupported scaling tool: ${tool.id}`);
}

/** Environment of every Nuxt command: no telemetry, a client build. */
export const NUXT_ENV = { CI: "1", NUXT_TELEMETRY_DISABLED: "1", NITRO_PRESET: "node-server" };

function version(bin: string, viaNode: string | null = null): string | null {
  try {
    const out = viaNode
      ? execFileSync(viaNode, [bin, "--version"], { encoding: "utf8" })
      : execFileSync(bin, ["--version"], { encoding: "utf8" });
    return out.trim().split("\n")[0] ?? null;
  } catch {
    return null;
  }
}

const manifestVersion = (dir: string): string | null => {
  try {
    return (JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as { version: string })
      .version;
  } catch {
    return null;
  }
};

/** The native executable a launcher package resolves to, matched by platform prefix. */
function nativeIn(scopeDir: string, prefix: string, file: string): string | null {
  if (!existsSync(scopeDir)) return null;
  for (const entry of readdirSync(scopeDir).sort()) {
    const candidate = join(scopeDir, entry, file);
    if (entry.startsWith(prefix) && existsSync(candidate)) return candidate;
  }
  return null;
}

export type Provenance = Record<
  string,
  { version: string | null; path: string | null; sha256: string | null }
>;

export function collectProvenance(env: ScalingEnv, hostsDir = TYPECHECK_HOSTS_DIR): Provenance {
  const hostModules = join(hostsDir, "node_modules");
  const platform = `${process.platform}-${process.arch}`;
  const nativeDir = join(env.target, "npm", "native");
  const binding = existsSync(nativeDir)
    ? readdirSync(nativeDir).find((entry) => entry.endsWith(".node"))
    : undefined;
  const vueTscDir = dirname(dirname(env.vueTsc));
  let typescriptDir: string | null = null;
  try {
    typescriptDir = dirname(realpathSync(join(dirname(vueTscDir), "typescript", "package.json")));
  } catch {
    typescriptDir = null;
  }
  const entry = (version: string | null, path: string | null) => ({
    version,
    path,
    sha256: path ? fileSha256(path) : null,
  });
  return {
    vize: entry(
      manifestVersion(join(env.target, "npm", "cli")),
      binding ? join(nativeDir, binding) : null,
    ),
    "tsgo (vize check)": entry(version(env.corsa), env.corsa),
    "typescript (mapper host)": entry(
      version(env.hosts.tsc),
      nativeIn(join(hostModules, "@typescript"), `typescript-${platform}`, join("lib", "tsc")),
    ),
    tsrs: entry(version(env.hosts.tsrs), nativeIn(join(hostModules, "@tsrs"), platform, "tsrs")),
    "@vue/content-mapper": entry(manifestVersion(env.hosts.vueMapperDir), null),
    "vue-tsc": entry(manifestVersion(vueTscDir), env.vueTsc),
    "typescript (vue-tsc)": entry(
      typescriptDir ? manifestVersion(typescriptDir) : null,
      typescriptDir ? join(typescriptDir, "lib", "typescript.js") : null,
    ),
    vue: entry(manifestVersion(env.vueDir), null),
    ...(env.nuxtModules
      ? {
          nuxt: entry(manifestVersion(join(env.nuxtModules, "nuxt")), null),
          "vue (nuxt-app)": entry(manifestVersion(join(env.nuxtModules, "vue")), null),
          "@vizejs/nuxt": entry(env.vizeNuxtDir ? manifestVersion(env.vizeNuxtDir) : null, null),
        }
      : {}),
    node: entry(process.version, process.execPath),
  };
}
