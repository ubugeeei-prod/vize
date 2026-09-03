import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

import type { VizeLintConfig, VizeLintConfigOptions } from "../vite-plus.ts";
import type { VizeLintConfigFragment, VizeLintFlatConfig } from "../vite-plus-flat-config.ts";

/**
 * Test support for the Vite+ `lint` block contract.
 *
 * These helpers build a throwaway project under `os.tmpdir()` and run the real
 * `oxlint` binary against it. Vite+ forwards its `lint` block to Oxlint
 * unchanged, so writing the block to `.oxlintrc.json` and invoking `oxlint`
 * exercises the same configuration path `vp lint` takes without needing Vite+
 * installed in the test environment.
 *
 * The project gets a `node_modules/oxlint-plugin-vize` symlink so the config can
 * be written to disk verbatim, bare specifier included, instead of being rewritten
 * to an absolute path that no real user would have.
 */

const packageDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const workspaceRoot = path.resolve(packageDir, "..", "..");
const distEntry = path.join(packageDir, "dist", "index.mjs");
const packageRequire = createRequire(path.join(packageDir, "package.json"));

export interface StableDiagnostic {
  code: string;
  filename: string;
  labels: { column: number; line: number }[];
  message: string;
  severity: string;
}

interface OxlintJsonDiagnostic {
  code: string;
  filename: string;
  labels: { span: { column: number; line: number } }[];
  message: string;
  severity: string;
}

/**
 * Loads `createVizeLintConfig` from the built bundle rather than from source.
 *
 * The bundle is what Oxlint loads through `jsPlugins`, so both halves of the
 * assertion have to come from the same artifact. `npm/oxlint`'s `test` script runs
 * `vp pack` first, matching `test.ts`, `nuxt-preset.test.ts`, and
 * `type-aware.test.ts`.
 */
export async function createVizeLintConfigFromDist(): Promise<
  (options?: VizeLintConfigOptions) => VizeLintConfig
> {
  return (await loadVitePlusConfigHelpersFromDist()).createVizeLintConfig;
}

export interface VitePlusConfigHelpers {
  createVizeLintConfig: (options?: VizeLintConfigOptions) => VizeLintConfig;
  createVizeLintFlatConfig: (options?: VizeLintConfigOptions) => VizeLintFlatConfig;
  defineVizeLintConfig: (
    ...entries: readonly (VizeLintConfigFragment | VizeLintFlatConfig)[]
  ) => VizeLintConfig;
  flatConfigs: Record<string, VizeLintFlatConfig>;
}

export async function loadVitePlusConfigHelpersFromDist(): Promise<VitePlusConfigHelpers> {
  const bundle = (await import(pathToFileURL(distEntry).href)) as VitePlusConfigHelpers;
  return bundle;
}

/**
 * Type-checks a strict Vite+ consumer against the packed declarations.
 *
 * Source-level checks miss declaration bundling mistakes and can accidentally
 * rely on workspace-only resolution. This project imports the package through
 * its public name from a throwaway `node_modules`, exactly like a consumer.
 */
export function typecheckVitePlusConfigConsumer(source: string): void {
  const root = createTypecheckWorkspace();

  try {
    fs.writeFileSync(path.join(root, "package.json"), '{"type":"module"}\n');
    fs.writeFileSync(path.join(root, "vite.config.mts"), source);
    fs.writeFileSync(
      path.join(root, "tsconfig.json"),
      `${JSON.stringify(
        {
          compilerOptions: {
            module: "NodeNext",
            moduleResolution: "NodeNext",
            noEmit: true,
            // Vite+'s declaration graph references optional tooling packages
            // that consumers do not need for lint config. Consumer source is
            // still checked strictly, including the @ts-expect-error probes.
            skipLibCheck: true,
            strict: true,
            target: "ES2022",
          },
          include: ["vite.config.mts"],
        },
        null,
        2,
      )}\n`,
    );

    const tscEntry = packageRequire.resolve("typescript/bin/tsc");
    execFileSync(process.execPath, [tscEntry, "--project", "tsconfig.json", "--pretty", "false"], {
      cwd: root,
      encoding: "utf8",
      stdio: "pipe",
    });
  } catch (error) {
    const execError = error as { stdout?: string | Buffer; stderr?: string | Buffer };
    throw new Error(
      `Strict Vite+ consumer typecheck failed:\n${String(execError.stdout ?? "")}${String(execError.stderr ?? "")}`,
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

export function lintWorkspaceFixture(fixture: {
  config: VizeLintConfig;
  filename: string;
  source: string;
}): StableDiagnostic[] {
  const root = createLintWorkspace();

  try {
    const target = path.join(root, fixture.filename);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, fixture.source);
    fs.writeFileSync(
      path.join(root, ".oxlintrc.json"),
      `${JSON.stringify(fixture.config, null, 2)}\n`,
    );

    return parseDiagnostics(
      runOxlint(root, ["-c", ".oxlintrc.json", "-f", "json", fixture.filename]),
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

/**
 * Imports a copy of the built plugin from a directory where the Vize native
 * binding cannot be resolved, and reports how that import ended.
 *
 * The copy keeps `@oxlint/plugins` reachable so the failure is attributable to
 * the missing binding rather than to a missing peer.
 */
export function runIsolatedPluginLoadProbe(): { outcome: string; reason: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-no-binding-"));

  try {
    fs.cpSync(path.join(packageDir, "dist"), path.join(root, "dist"), { recursive: true });
    linkOxlintPluginsOnly(root);
    const probePath = path.join(root, "probe.mjs");
    fs.writeFileSync(probePath, createLoadProbeSource());

    const stdout = execFileSync(process.execPath, [probePath], {
      cwd: root,
      encoding: "utf8",
      stdio: "pipe",
    });
    return JSON.parse(stdout.trim()) as { outcome: string; reason: string };
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

function createLoadProbeSource(): string {
  // `reason` is the first sentence of the thrown message so the assertion stays
  // machine-independent: the full message lists the platform binding packages.
  return `import("./dist/index.mjs").then(
  () => {
    process.stdout.write(JSON.stringify({ outcome: "loaded", reason: "" }));
  },
  (error) => {
    process.stdout.write(
      JSON.stringify({
        outcome: error instanceof Error ? "threw" : "rejected-non-error",
        reason: error instanceof Error ? error.message.split(".")[0] : "",
      }),
    );
  },
);
`;
}

function createLintWorkspace(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vite-plus-lint-"));
  const nodeModules = path.join(root, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(packageDir, path.join(nodeModules, "oxlint-plugin-vize"), "junction");
  return root;
}

function createTypecheckWorkspace(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vite-plus-types-"));
  const nodeModules = path.join(root, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(packageDir, path.join(nodeModules, "oxlint-plugin-vize"), "junction");
  linkPackageFromManifest(nodeModules, "oxlint");
  linkPackageFromManifest(nodeModules, "vite-plus");
  linkOxlintPluginsOnly(root);
  return root;
}

function linkPackageFromManifest(nodeModules: string, packageName: string): void {
  const source = path.dirname(packageRequire.resolve(`${packageName}/package.json`));
  const target = path.join(nodeModules, ...packageName.split("/"));
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.symlinkSync(source, target, "junction");
}

function linkOxlintPluginsOnly(root: string): void {
  const scopeDir = path.join(root, "node_modules", "@oxlint");
  fs.mkdirSync(scopeDir, { recursive: true });
  fs.symlinkSync(resolveOxlintPluginsDir(), path.join(scopeDir, "plugins"), "junction");
}

function resolveOxlintPluginsDir(): string {
  let current = path.dirname(packageRequire.resolve("@oxlint/plugins"));

  for (;;) {
    if (fs.existsSync(path.join(current, "package.json"))) {
      return current;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error("Unable to locate the @oxlint/plugins package root.");
    }

    current = parent;
  }
}

function runOxlint(cwd: string, args: readonly string[]): string {
  const env = { ...process.env };
  // Oxlint switches to GitHub annotation output when this is set, which would
  // change the JSON payload under CI.
  delete env.GITHUB_ACTIONS;

  try {
    return String(
      execFileSync(findOxlintBin(), args, { cwd, encoding: "utf8", env, stdio: "pipe" }),
    );
  } catch (error) {
    const execError = error as { status?: number; stdout?: string | Buffer; stderr?: string };
    if (execError.status !== 1) {
      throw new Error(
        `oxlint exited with ${String(execError.status)}\n${String(execError.stderr ?? "")}`,
      );
    }

    return String(execError.stdout ?? "");
  }
}

function findOxlintBin(): string {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const match = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .sort((left, right) => right.localeCompare(left))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .find((entry) => fs.existsSync(entry));

  if (match == null) {
    throw new Error(`Unable to locate the oxlint binary in ${pnpmStoreDir}`);
  }

  return match;
}

/**
 * Projects Oxlint's JSON payload down to the fields that are stable across
 * machines. `number_of_rules`, `threads_count`, and `start_time` are not.
 */
function parseDiagnostics(stdout: string): StableDiagnostic[] {
  const payload = JSON.parse(stdout) as { diagnostics: OxlintJsonDiagnostic[] };

  return payload.diagnostics.map((diagnostic) => ({
    code: diagnostic.code,
    filename: diagnostic.filename,
    labels: diagnostic.labels.map((label) => ({
      column: label.span.column,
      line: label.span.line,
    })),
    message: diagnostic.message,
    severity: diagnostic.severity,
  }));
}
