import assert from "node:assert/strict";
import { lstatSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";
import type { BinPolicy } from "./dependency-bin-policy.ts";
import type { PublicPackageFacts } from "./dependency-bin-types.ts";
import type { CaptureInstallReceipt, Image, Recipe, PublicPayloadReceipt } from "./model.ts";
import type { Sink } from "./evidence.ts";
import { discard } from "./evidence.ts";
import { bytes, object, wholeJson } from "./value.ts";
import { cleanEnvironment } from "./capture.ts";
import { recipes } from "./recipes.ts";
import { collectRootCommandCensus, type RootCommandCensus } from "./command-census.ts";
import { captureRuntimePaths, type RuntimePathReceipt } from "./runtime-chain.ts";
import {
  recheckBinConfiguration,
  safeAgeWorkspace,
  validateBinConfiguration,
  type BinConfiguration,
} from "./bin-configuration.ts";

export interface ExecutableChains {
  schema: "vize.humanReports11252_11253.executableChains.v1";
  kind: "original" | "tsrs-supplement";
  recipe: Recipe;
  initialManifestBase64: string;
  requiredCommands: string[];
  census: RootCommandCensus;
  runtime: RuntimePathReceipt;
  authenticatedForCaptureOnly: true;
  completeInstalledFileSetQualified: false;
  accepted: false;
  productQualified: false;
  afterQualified: false;
  issueQualified: false;
}
interface Inputs {
  root: string;
  kind: ExecutableChains["kind"];
  recipe: Recipe;
  manifest: Buffer;
  configuration: BinConfiguration;
  binPolicy: BinPolicy;
  packages: PublicPayloadReceipt[];
  catalog: ReadonlyMap<string, PublicPackageFacts>;
  node: Image;
  pnpm: Image;
  env: NodeJS.ProcessEnv;
  sink: Sink;
}
function sourceExtraPaths(root: string, manifest: Buffer): string[] {
  const workspace = join(root, "pnpm-workspace.yaml"),
    stat = lstatSync(workspace, { throwIfNoEntry: false });
  if (!stat) return [];
  assert(stat.isFile() && !stat.isSymbolicLink());
  assert(safeAgeWorkspace(readFileSync(workspace), manifest));
  // c7 derive_layout::workspace_root_bin_paths; closed Cargo/Python defaults
  // are both disabled, so execution_paths returns these extra paths unchanged.
  return [join(root, "node_modules/.bin")];
}
function verifyRequired(
  census: RootCommandCensus,
  recipe: Recipe,
  kind: ExecutableChains["kind"],
  manifest: Buffer,
): string[] {
  assert.deepEqual(recipe, recipes[recipe.role]);
  const root = object(wholeJson(manifest));
  if (recipe.issue === 11252)
    assert.equal(object(root.scripts).check, "tsc --runExternalCode --noEmit -p tsconfig.json");
  else assert.equal(root.scripts, undefined);
  const required = recipe.issue === 11252 ? ["tsc", "vize"] : ["vize", "vue-tsc"];
  if (kind === "tsrs-supplement") {
    assert.equal(recipe.issue, 11252);
    required.push("tsrs");
  }
  const versions = new Map([
    ["typescript", recipe.typescript],
    ["vize", object(root.devDependencies).vize],
    ["vue-tsc", recipe.vueTsc],
    ["tsrs", recipe.tsrs],
  ]);
  for (const name of required) {
    const targets = census.targets.filter((row) => row.name === name);
    assert.equal(targets.length, 1, "mandatory original executable is UNKNOWN");
    const owner = name === "tsc" ? "typescript" : name;
    assert.equal(targets[0]!.ownerName, owner);
    assert.equal(targets[0]!.ownerVersion, versions.get(owner));
  }
  return required;
}
export function captureExecutableChains(input: Inputs): ExecutableChains {
  const { root, manifest, configuration, sink } = input;
  assert.equal(basename(root), "consumer-" + input.kind);
  assert.equal(root, configuration.root);
  validateBinConfiguration(configuration, bytes(configuration.rootPackage.rawBase64!));
  recheckBinConfiguration(configuration, manifest, sink);
  for (const pkg of input.packages) {
    const original = input.catalog.get(pkg.directory);
    assert(original && original.name === pkg.name && original.version === pkg.version);
    assert(original.manifest.equals(bytes(pkg.manifestRawBase64)));
    assert.equal(original.integrity, pkg.integrity);
  }
  const census = collectRootCommandCensus(root, manifest, input.packages, input.binPolicy, sink);
  const requiredCommands = verifyRequired(census, input.recipe, input.kind, manifest);
  const runtime = captureRuntimePaths(
    root,
    input.pnpm,
    input.node,
    input.env,
    sourceExtraPaths(root, manifest),
    sink,
  );
  const proof: ExecutableChains = {
    schema: "vize.humanReports11252_11253.executableChains.v1",
    kind: input.kind,
    recipe: input.recipe,
    initialManifestBase64: manifest.toString("base64"),
    requiredCommands,
    census,
    runtime,
    authenticatedForCaptureOnly: true,
    completeInstalledFileSetQualified: false,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
  };
  sink("mandatory-executable-chains-authenticated-for-capture", proof);
  return proof;
}
export function recheckExecutableChains(
  proof: ExecutableChains,
  install: CaptureInstallReceipt,
  expectedCaseManifest?: Buffer,
  sink: Sink = discard,
): void {
  assert.equal(proof.schema, "vize.humanReports11252_11253.executableChains.v1");
  assert.equal(proof.kind, install.kind);
  assert.equal(proof.authenticatedForCaptureOnly, true);
  for (const key of [
    "completeInstalledFileSetQualified",
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
  ] as const)
    assert.equal(proof[key], false);
  const manifest = expectedCaseManifest ?? bytes(proof.initialManifestBase64);
  const census = collectRootCommandCensus(
    install.root,
    manifest,
    install.packages,
    install.binPolicy,
    sink,
  );
  assert.deepEqual(
    verifyRequired(census, proof.recipe, install.kind, manifest),
    proof.requiredCommands,
  );
  // A registered after input may change only the full expected root manifest;
  // every executable owner, target, wrapper and physical alias must stay equal.
  assert.deepEqual({ ...census, rootPackageSha256: proof.census.rootPackageSha256 }, proof.census);
  const runtime = captureRuntimePaths(
    install.root,
    install.pnpm,
    install.node,
    { ...cleanEnvironment(sink), ...install.packageManagerEnvironment },
    sourceExtraPaths(install.root, manifest),
    sink,
  );
  assert.deepEqual(runtime, proof.runtime);
}
