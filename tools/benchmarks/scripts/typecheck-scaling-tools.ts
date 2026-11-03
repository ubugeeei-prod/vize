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
 */

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
} from "node:fs";
import { dirname, join } from "node:path";

import { fileSha256 } from "./benchmark-binary.mjs";
import { linkPackage } from "./install-package-files.ts";
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
  | "tsc-vize-mapper"
  | "tsrs-fast-vize-mapper"
  | "vue-tsc"
  | "tsc-vue-mapper";
export type ScalingTool = {
  id: ToolId;
  label: string;
  /** A Vize curve: superlinear growth is filed as a p0 issue. */
  vize: boolean;
  format: "json" | "text";
};
export const TOOLS: readonly ScalingTool[] = [
  { id: "vize-check", label: "vize check", vize: true, format: "json" },
  { id: "tsc-vize-mapper", label: "tsc 7.1 + Vize mapper", vize: true, format: "text" },
  { id: "tsrs-fast-vize-mapper", label: "tsrs --fast + Vize mapper", vize: true, format: "text" },
  { id: "vue-tsc", label: "vue-tsc", vize: false, format: "text" },
  { id: "tsc-vue-mapper", label: "tsc 7.1 + Vue mapper", vize: false, format: "text" },
];

export type ScalingEnv = {
  target: string;
  node: string;
  vizeCli: string;
  corsa: string;
  vueTsc: string;
  vueDir: string;
  hosts: MapperHosts;
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
  };
}

/** `vue` with the packages pnpm placed beside it, then both mappers and the 7.1 `typescript`. */
export function prepareProjectPackages(dir: string, env: ScalingEnv): void {
  const modules = join(dir, "node_modules");
  const siblings = dirname(env.vueDir);
  for (const entry of readdirSync(siblings)) {
    if (entry === "typescript" || entry.startsWith(".")) continue;
    if (entry.startsWith("@")) {
      for (const scoped of readdirSync(join(siblings, entry))) {
        linkPackage(realpathSync(join(siblings, entry, scoped)), join(modules, entry, scoped));
      }
    } else {
      mkdirSync(modules, { recursive: true });
      const target = join(modules, entry);
      if (!existsSync(target)) symlinkSync(realpathSync(join(siblings, entry)), target, "dir");
    }
  }
  prepareMapperPackages(dir, env.hosts);
}

export function invocation(
  tool: ScalingTool,
  env: ScalingEnv,
  cwd: string,
  kind: "project" | "monorepo",
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
              join(cwd, "tsconfig.json"),
            ],
        stats: !build,
      };
  }
}

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
    node: entry(process.version, process.execPath),
  };
}
