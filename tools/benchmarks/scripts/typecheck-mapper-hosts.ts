/**
 * TypeScript 7.1 content-mapper lanes: the hosts (`tsc`, `tsrs`) and the
 * mappers they load (Vize's `vize` package, Vue's `@vue/content-mapper`).
 *
 * Hosts and Vue's mapper come from the pinned npm lockfile in
 * tools/benchmarks/typecheck-hosts. The Vize mapper is the workspace's built
 * `npm/cli` package, installed into each project as package files. Each
 * project also gets the hosts' `typescript`, which Vue's mapper loads from
 * the project, as `npm i -D typescript@beta` provides it.
 */

import { existsSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { ENGINE_CLASSES } from "./check-gate-report.ts";
import { installPackageFiles, linkPackage } from "./install-package-files.ts";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

export const TYPECHECK_HOSTS_DIR = resolve(benchDir, "..", "typecheck-hosts");
export const VIZE_CLI_PACKAGE_DIR = join(rootDir, "npm", "cli");

type ContentMapper = { id: string; label: string; package: string };
/** A checker setting: lane id suffix (empty for the host default) and its flags on a runner with `cpus` vCPUs. */
type Setting = { id: string; args: (cpus: number) => string[] };
type MapperHost = {
  id: "tsc" | "tsrs";
  label: string;
  engineClass: "tsgo-native" | "tsrs-native";
  settings: Setting[];
};
export type MapperHosts = {
  tsc: string;
  tsrs: string;
  typescriptDir: string;
  vueMapperDir: string;
  vizeCliDir: string;
};
type CommandOptions = { cwd: string; env?: Record<string, string> };
type RunCommand<Result> = (binary: string, args: string[], options: CommandOptions) => Result;

export const CONTENT_MAPPERS: ContentMapper[] = [
  { id: "vize", label: "Vize mapper", package: "vize" },
  { id: "vue", label: "Vue mapper", package: "@vue/content-mapper" },
];

const DEFAULT_SETTING: Setting = { id: "", args: () => [] };

/** Checker counts beyond the default; `max` is the runner's vCPU count. */
export const CHECKER_SETTINGS: Setting[] = ["4", "8", "16", "max"].map((count) => ({
  id: `checkers-${count}`,
  args: (cpus) => ["--checkers", count === "max" ? String(cpus) : count],
}));

export const settingLabel = (setting: Setting, cpus: number): string =>
  setting.args(cpus).join(" ") || "default";

export const MAPPER_HOSTS: MapperHost[] = [
  {
    id: "tsc",
    label: "tsc",
    engineClass: "tsgo-native",
    settings: [DEFAULT_SETTING, ...CHECKER_SETTINGS],
  },
  {
    id: "tsrs",
    label: "tsrs",
    engineClass: "tsrs-native",
    settings: [DEFAULT_SETTING, { id: "fast", args: () => ["--fast"] }],
  },
];

export const mapperLaneId = (host: MapperHost, mapper: ContentMapper, setting: Setting): string =>
  `${host.id}-${mapper.id}-mapper${setting.id ? `-${setting.id}` : ""}`;
export const vizeCheckerLaneId = (setting: Setting): string => `vize-check-${setting.id}`;
export const mapperTsconfigName = (mapper: ContentMapper): string =>
  `tsconfig.${mapper.id}-mapper.json`;

/** The check gate's engine classes plus the tsrs host it does not run. */
export const ENGINE_CLASS_LABELS: Record<string, string> = {
  ...ENGINE_CLASSES,
  "tsrs-native": "native TypeScript port (tsrs)",
};

/** Every mapper and `vize check --checkers` lane id with its engine class. */
export const SETTING_LANE_ENGINE_CLASSES: Record<string, MapperHost["engineClass"]> = {
  ...Object.fromEntries(
    CHECKER_SETTINGS.map((setting) => [vizeCheckerLaneId(setting), "tsgo-native" as const]),
  ),
  ...Object.fromEntries(
    MAPPER_HOSTS.flatMap((host) =>
      CONTENT_MAPPERS.flatMap((mapper) =>
        host.settings.map((setting) => [mapperLaneId(host, mapper, setting), host.engineClass]),
      ),
    ),
  ),
};

function hostPaths(hostsDir: string): { tsc: string; tsrs: string; vueMapper: string } {
  const modules = join(hostsDir, "node_modules");
  return {
    tsc: join(modules, "typescript", "bin", "tsc"),
    tsrs: join(modules, "tsrs", "bin", "tsrs"),
    vueMapper: join(modules, "@vue", "content-mapper"),
  };
}

/** Host binaries for provenance; null entries when the hosts are not installed. */
export function mapperHostBins(hostsDir = TYPECHECK_HOSTS_DIR): {
  tscMapperHostBin: string | null;
  tsrsBin: string | null;
  vueContentMapperDir: string | null;
} {
  const paths = hostPaths(hostsDir);
  return {
    tscMapperHostBin: existsSync(paths.tsc) ? paths.tsc : null,
    tsrsBin: existsSync(paths.tsrs) ? paths.tsrs : null,
    vueContentMapperDir: existsSync(paths.vueMapper) ? realpathSync(paths.vueMapper) : null,
  };
}

/** Resolve the pinned hosts and both mappers; a missing piece is an error. */
export function resolveMapperHosts(
  hostsDir = TYPECHECK_HOSTS_DIR,
  cliDir = VIZE_CLI_PACKAGE_DIR,
): MapperHosts {
  const paths = { ...hostPaths(hostsDir), vizeCli: join(cliDir, "dist", "cli.mjs") };
  for (const [name, path] of Object.entries(paths)) {
    if (!existsSync(path)) {
      throw new Error(
        `type-check mapper lanes: ${name} not found at ${path}. Run \`npm ci --prefix tools/benchmarks/typecheck-hosts\` and \`vp run --workspace-root build:vite-plugin\` first.`,
      );
    }
  }
  return {
    tsc: paths.tsc,
    tsrs: paths.tsrs,
    typescriptDir: realpathSync(join(hostsDir, "node_modules", "typescript")),
    vueMapperDir: realpathSync(paths.vueMapper),
    vizeCliDir: cliDir,
  };
}

/** Install both mappers into one project and write one tsconfig per mapper. */
export function prepareMapperPackages(dir: string, hosts: MapperHosts): void {
  const modules = join(dir, "node_modules");
  installPackageFiles(hosts.vizeCliDir, modules, "vize");
  linkPackage(hosts.vueMapperDir, join(modules, "@vue", "content-mapper"));
  linkPackage(hosts.typescriptDir, join(modules, "typescript"));
  for (const mapper of CONTENT_MAPPERS) {
    writeFileSync(
      join(dir, mapperTsconfigName(mapper)),
      `${JSON.stringify(
        {
          extends: "./tsconfig.json",
          contentMappers: [{ package: mapper.package, extensions: [".vue"] }],
        },
        null,
        2,
      )}\n`,
    );
  }
}

/** One lane per host and mapper; each checks the project through its mapper tsconfig. */
type Lane<Result> = {
  id: string;
  label: string;
  tool: string;
  setting: string;
  files: number;
  run: (cwd: string) => Result;
};

/** One lane per host, mapper and checker setting. */
export function createMapperLanes<Result>({
  fileCount,
  hosts,
  cpus,
  runCommand,
}: {
  fileCount: number;
  hosts: MapperHosts;
  cpus: number;
  runCommand: RunCommand<Result>;
}): Lane<Result>[] {
  return MAPPER_HOSTS.flatMap((host) =>
    CONTENT_MAPPERS.flatMap((mapper) =>
      host.settings.map((setting) => laneFor(host, mapper, setting)),
    ),
  );
  function laneFor(host: MapperHost, mapper: ContentMapper, setting: Setting): Lane<Result> {
    const tool = `${host.label} + ${mapper.label}`;
    const label = settingLabel(setting, cpus);
    return {
      id: mapperLaneId(host, mapper, setting),
      label: setting.id ? `${tool} (${label})` : tool,
      tool,
      setting: label,
      files: fileCount,
      run: (cwd: string) =>
        runCommand(
          hosts[host.id],
          [
            "--noEmit",
            ...setting.args(cpus),
            "--runExternalCode",
            "--pretty",
            "false",
            "-p",
            join(cwd, mapperTsconfigName(mapper)),
          ],
          { cwd },
        ),
    };
  }
}
