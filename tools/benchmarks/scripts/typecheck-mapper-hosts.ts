/**
 * TypeScript 7.1 content-mapper lanes: the hosts (`tsc`, `tsrs`) and the
 * mappers they load (Vize's `vize` package, Vue's `@vue/content-mapper`).
 *
 * Hosts and Vue's mapper come from the pinned npm lockfile in
 * tools/benchmarks/typecheck-hosts. The Vize mapper is the workspace's built
 * `npm/cli` package, installed into each project the way npm installs it: a
 * real directory of its published `files`, with its dependencies beside it.
 * A symlink to the source package would resolve helper imports such as `vue`
 * from the source tree instead of from the project.
 */

import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

export const TYPECHECK_HOSTS_DIR = resolve(benchDir, "..", "typecheck-hosts");
export const VIZE_CLI_PACKAGE_DIR = join(rootDir, "npm", "cli");

type ContentMapper = { id: string; label: string; package: string };
type MapperHost = {
  id: "tsc" | "tsrs";
  label: string;
  engineClass: "tsgo-native" | "tsrs-native";
  args: string[];
};
export type MapperHosts = { tsc: string; tsrs: string; vueMapperDir: string; vizeCliDir: string };
type CommandOptions = { cwd: string; env?: Record<string, string> };
type RunCommand<Result> = (binary: string, args: string[], options: CommandOptions) => Result;

export const CONTENT_MAPPERS: ContentMapper[] = [
  { id: "vize", label: "Vize mapper", package: "vize" },
  { id: "vue", label: "Vue mapper", package: "@vue/content-mapper" },
];

export const MAPPER_HOSTS: MapperHost[] = [
  { id: "tsc", label: "tsc", engineClass: "tsgo-native", args: [] },
  { id: "tsrs", label: "tsrs", engineClass: "tsrs-native", args: ["--fast"] },
];

export const mapperLaneId = (host: MapperHost, mapper: ContentMapper): string =>
  `${host.id}-${mapper.id}-mapper`;
export const mapperTsconfigName = (mapper: ContentMapper): string =>
  `tsconfig.${mapper.id}-mapper.json`;

/** Every mapper lane id with its engine class, in measurement order. */
export const MAPPER_LANE_ENGINE_CLASSES: Record<string, MapperHost["engineClass"]> =
  Object.fromEntries(
    MAPPER_HOSTS.flatMap((host) =>
      CONTENT_MAPPERS.map((mapper) => [mapperLaneId(host, mapper), host.engineClass]),
    ),
  );

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
    vueMapperDir: realpathSync(paths.vueMapper),
    vizeCliDir: cliDir,
  };
}

/** A symlinked scope (`node_modules/@vue -> store`) becomes a real dir of links, so adding to it never writes into the store. */
function ensureRealDirectory(dir: string): void {
  if (existsSync(dir) && lstatSync(dir).isSymbolicLink()) {
    const source = realpathSync(dir);
    rmSync(dir);
    mkdirSync(dir);
    for (const entry of readdirSync(source)) symlinkSync(join(source, entry), join(dir, entry));
  }
  mkdirSync(dir, { recursive: true });
}

function linkPackage(source: string, target: string): void {
  if (existsSync(target)) return;
  ensureRealDirectory(dirname(target));
  symlinkSync(source, target, "dir");
}

function installPackageFiles(sourceDir: string, modules: string, name: string): void {
  const target = join(modules, name);
  if (existsSync(target)) return;
  const manifest = JSON.parse(readFileSync(join(sourceDir, "package.json"), "utf8")) as {
    files: string[];
    dependencies?: Record<string, string>;
  };
  for (const entry of ["package.json", ...manifest.files]) {
    if (!existsSync(join(sourceDir, entry))) {
      throw new Error(
        `type-check mapper lanes: ${name} is not built (${join(sourceDir, entry)} missing)`,
      );
    }
    cpSync(join(sourceDir, entry), join(target, entry), { recursive: true });
  }
  for (const dependency of Object.keys(manifest.dependencies ?? {})) {
    linkPackage(
      realpathSync(join(sourceDir, "node_modules", dependency)),
      join(modules, dependency),
    );
  }
}

/** Install both mappers into one project and write one tsconfig per mapper. */
export function prepareMapperPackages(dir: string, hosts: MapperHosts): void {
  const modules = join(dir, "node_modules");
  installPackageFiles(hosts.vizeCliDir, modules, "vize");
  linkPackage(hosts.vueMapperDir, join(modules, "@vue", "content-mapper"));
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
export function createMapperLanes<Result>({
  fileCount,
  hosts,
  runCommand,
}: {
  fileCount: number;
  hosts: MapperHosts;
  runCommand: RunCommand<Result>;
}): Array<{ id: string; label: string; files: number; run: (cwd: string) => Result }> {
  return MAPPER_HOSTS.flatMap((host) =>
    CONTENT_MAPPERS.map((mapper) => ({
      id: mapperLaneId(host, mapper),
      label: `${host.label} + ${mapper.label}`,
      files: fileCount,
      run: (cwd: string) =>
        runCommand(
          hosts[host.id],
          [
            "--noEmit",
            ...host.args,
            "--runExternalCode",
            "--pretty",
            "false",
            "-p",
            join(cwd, mapperTsconfigName(mapper)),
          ],
          { cwd },
        ),
    })),
  );
}
