/**
 * Upstream `@vitejs/plugin-vue` surface enumeration for the Vize Vite-plugin
 * parity gate (#3227).
 *
 * The surface is read from the *installed* copy of `@vitejs/plugin-vue` instead
 * of a hand-written list, so a new upstream option cannot silently escape the
 * ledger: the parity test requires exactly one ledger entry per enumerated
 * option, `Api` member, and plugin hook.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import ts from "typescript";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const ledgerPath = path.join(
  repoRoot,
  "tests",
  "_fixtures",
  "vite-plugin-vue-option-parity.json",
);

export const upstreamPackage = "@vitejs/plugin-vue";
export const trackingIssue = 3227;

/** Option groups whose members are enumerated as `<group>.<member>` leaves. */
const optionGroups = ["features", "script", "style", "template"] as const;
/** Plugin object keys that are not Rollup/Vite hooks. */
const nonHookPluginKeys = new Set(["api", "name"]);

const requireFromBench = createRequire(
  path.join(repoRoot, "tools", "benchmarks", "scripts", "package.json"),
);

/** Instantiate the pinned upstream plugin for behavioral parity probes. */
export function createUpstreamPlugin(
  options: Record<string, unknown> = {},
): Record<string, unknown> {
  const upstreamModule = requireFromBench(upstreamPackage) as
    | ((options?: unknown) => Record<string, unknown>)
    | { default: (options?: unknown) => Record<string, unknown> };
  const factory = typeof upstreamModule === "function" ? upstreamModule : upstreamModule.default;
  assert.equal(typeof factory, "function", `${upstreamPackage} must export a plugin factory`);
  return factory(options);
}

export interface UpstreamSurface {
  apiMembers: string[];
  hooks: string[];
  options: string[];
  version: string;
}

/** The `@vitejs/plugin-vue` version pinned by the `vite-stack` pnpm catalog. */
export function catalogVersion(): string {
  const workspaceYaml = fs.readFileSync(path.join(repoRoot, "pnpm-workspace.yaml"), "utf8");
  const pin = workspaceYaml
    .split("\n")
    .map((line) => line.match(/^\s{4}"?(?<name>[^":]+)"?:\s*"(?<version>[^"]+)"\s*$/))
    .find((match) => match?.groups?.name === upstreamPackage)?.groups?.version;
  assert.ok(pin, `${upstreamPackage} must stay pinned in the pnpm catalog`);
  assert.match(pin, /^\d+\.\d+\.\d+$/u, `${upstreamPackage} must be pinned to an exact version`);
  return pin;
}

function installedVersion(): string {
  const manifest = requireFromBench(`${upstreamPackage}/package.json`) as { version: string };
  assert.equal(
    manifest.version,
    catalogVersion(),
    `the installed ${upstreamPackage} must match the catalog pin`,
  );
  return manifest.version;
}

function createOptionsProgram(): { checker: ts.TypeChecker; source: ts.SourceFile } {
  // `bench` owns the `@vitejs/plugin-vue` devDependency, so resolve the probe
  // module from there.
  const probePath = path.join(
    repoRoot,
    "tools",
    "benchmarks",
    "scripts",
    "__vize_plugin_vue_option_probe__.ts",
  );
  const probeSource = `import type { Options } from "${upstreamPackage}";\nexport declare const probe: Options;\n`;
  const host = ts.createCompilerHost({}, true);
  const isProbe = (fileName: string) => path.resolve(fileName) === probePath;
  const getSourceFile = host.getSourceFile.bind(host);
  host.getSourceFile = (fileName, languageVersion, onError, shouldCreate) =>
    isProbe(fileName)
      ? ts.createSourceFile(fileName, probeSource, languageVersion, true)
      : getSourceFile(fileName, languageVersion, onError, shouldCreate);
  const fileExists = host.fileExists.bind(host);
  host.fileExists = (fileName) => isProbe(fileName) || fileExists(fileName);
  const readFile = host.readFile.bind(host);
  host.readFile = (fileName) => (isProbe(fileName) ? probeSource : readFile(fileName));

  const program = ts.createProgram({
    rootNames: [probePath],
    options: {
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      noEmit: true,
      skipLibCheck: true,
      strict: true,
      types: [],
    },
    host,
  });
  const source = program.getSourceFile(probePath);
  assert.ok(source, "the option probe module must be part of the program");
  const diagnostics = program
    .getSemanticDiagnostics(source)
    .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, " "));
  assert.deepEqual(diagnostics, [], `resolving ${upstreamPackage} option types failed`);
  return { checker: program.getTypeChecker(), source };
}

function optionsType(checker: ts.TypeChecker, source: ts.SourceFile): ts.Type {
  const statement = source.statements.find(ts.isVariableStatement);
  assert.ok(statement, "the option probe module must declare the probe binding");
  return checker.getTypeAtLocation(statement.declarationList.declarations[0]);
}

function memberNames(checker: ts.TypeChecker, type: ts.Type): string[] {
  return checker
    .getPropertiesOfType(type)
    .map((property) => property.getName())
    .toSorted();
}

/**
 * Every documented `Options` leaf, as dotted paths. Nested option groups
 * (`template`, `script`, `style`, `features`) expand one level so the ledger
 * tracks `template.compilerOptions` rather than an opaque `template` blob.
 */
function enumerateOptions(): string[] {
  const { checker, source } = createOptionsProgram();
  const options = optionsType(checker, source);
  const leaves: string[] = [];

  for (const name of memberNames(checker, options)) {
    if (!(optionGroups as readonly string[]).includes(name)) {
      leaves.push(name);
      continue;
    }
    const property = checker.getPropertyOfType(options, name);
    assert.ok(property, `${name} must exist on the upstream Options type`);
    const groupType = checker.getNonNullableType(
      checker.getTypeOfSymbolAtLocation(property, source),
    );
    const members = memberNames(checker, groupType);
    assert.notEqual(members.length, 0, `${name} must expose at least one documented sub-option`);
    leaves.push(...members.map((member) => `${name}.${member}`));
  }

  return leaves.toSorted();
}

/** The `Api` members and plugin hooks the upstream plugin instance exposes. */
function enumeratePluginSurface(): { apiMembers: string[]; hooks: string[] } {
  const plugin = createUpstreamPlugin();
  const api = plugin.api;
  assert.ok(api && typeof api === "object", `${upstreamPackage} must expose its framework api`);

  return {
    // `Api` is a plain object of accessors, so own property names cover the
    // getter/setter pairs without walking `Object.prototype`.
    apiMembers: Object.getOwnPropertyNames(api).toSorted(),
    hooks: Object.keys(plugin)
      .filter((key) => !nonHookPluginKeys.has(key))
      .toSorted(),
  };
}

export function upstreamSurface(): UpstreamSurface {
  const { apiMembers, hooks } = enumeratePluginSurface();
  return { apiMembers, hooks, options: enumerateOptions(), version: installedVersion() };
}

export type LedgerStatus = "honored" | "intentional-divergence" | "unimplemented";

export interface LedgerEntry {
  evidence?: string;
  issue?: number;
  reason?: string;
  status: LedgerStatus;
}

export interface ParityLedger {
  api: Record<string, LedgerEntry>;
  hooks: Record<string, LedgerEntry>;
  options: Record<string, LedgerEntry>;
  schemaVersion: number;
  summary: Record<LedgerStatus, number>;
  upstream: {
    apiCount: number;
    hookCount: number;
    optionCount: number;
    package: string;
    version: string;
  };
}

export function readLedger(): ParityLedger {
  return JSON.parse(fs.readFileSync(ledgerPath, "utf8")) as ParityLedger;
}

const sections = ["api", "hooks", "options"] as const;

/**
 * Structural validation: exhaustive over the upstream surface, and no entry may
 * claim `honored` without naming the behavioral probe that proves it.
 */
export function validateLedger(ledger: ParityLedger, surface: UpstreamSurface): void {
  assert.equal(ledger.schemaVersion, 1);
  assert.deepEqual(ledger.upstream, {
    apiCount: surface.apiMembers.length,
    hookCount: surface.hooks.length,
    optionCount: surface.options.length,
    package: upstreamPackage,
    version: surface.version,
  });

  const expected: Record<(typeof sections)[number], string[]> = {
    api: surface.apiMembers,
    hooks: surface.hooks,
    options: surface.options,
  };
  const counts: Record<LedgerStatus, number> = {
    honored: 0,
    "intentional-divergence": 0,
    unimplemented: 0,
  };

  for (const section of sections) {
    assert.deepEqual(
      Object.keys(ledger[section]),
      expected[section],
      `every upstream ${section} entry needs exactly one sorted ledger entry`,
    );
    for (const [name, entry] of Object.entries(ledger[section])) {
      const label = `${section}.${name}`;
      assert.equal(typeof entry, "object", `${label} needs a structured ledger entry`);
      if (entry.status === "honored") {
        assert.deepEqual(Object.keys(entry).toSorted(), ["evidence", "status"]);
        assert.match(entry.evidence ?? "", /\S/u, `${label} must name its behavioral evidence`);
      } else if (entry.status === "intentional-divergence") {
        assert.deepEqual(Object.keys(entry).toSorted(), ["reason", "status"]);
        assert.match(entry.reason ?? "", /\S/u, `${label} needs a non-empty divergence reason`);
      } else if (entry.status === "unimplemented") {
        assert.deepEqual(Object.keys(entry).toSorted(), ["issue", "reason", "status"]);
        assert.equal(entry.issue, trackingIssue, `${label} must link the parity issue`);
        assert.match(entry.reason ?? "", /\S/u, `${label} needs a non-empty gap reason`);
      } else {
        assert.fail(`${label} has unsupported status ${JSON.stringify(entry.status)}`);
      }
      counts[entry.status] += 1;
    }
  }

  assert.deepEqual(ledger.summary, counts, "the ledger summary must match its entries");
}

/** Every `honored` entry, as `<section>.<name>` → evidence id. */
export function honoredEvidence(ledger: ParityLedger): Map<string, string> {
  const evidence = new Map<string, string>();
  for (const section of sections) {
    for (const [name, entry] of Object.entries(ledger[section])) {
      if (entry.status === "honored") {
        evidence.set(`${section}.${name}`, entry.evidence as string);
      }
    }
  }
  return evidence;
}
