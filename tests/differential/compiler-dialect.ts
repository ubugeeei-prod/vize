import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { LEGACY_BUILD_RECIPE } from "./build-receipt.ts";
import { readPinnedArtifact } from "./harness.mjs";

export interface DialectSelector {
  readonly vueVersion: string;
  readonly templateSyntax: string;
}

export interface RegisteredDialectInput {
  readonly id: string;
  readonly input: { readonly path: string; readonly sha256: string };
  readonly selector: DialectSelector;
  readonly expectedParserMode: string;
  readonly state: string;
  readonly dialects: readonly string[];
}

interface CompilerArgvFixture {
  readonly dialectInput?: { readonly selector: DialectSelector };
}

interface LoadedCompilerDialectFixture {
  readonly dialectInputId: string;
  readonly dialectInputSha256: string;
  readonly dialectTransport: string;
  readonly dialectConfig?: Buffer | null;
  readonly dialectConfigPath?: string | null;
  readonly dialectConfigSha256?: string | null;
}

interface ManifestDialectFixture {
  readonly inputs: {
    readonly dialectInput: {
      readonly id: string;
      readonly sha256: string;
      readonly selector: DialectSelector;
      readonly expectedParserMode: string;
      readonly transport: string;
    };
  };
  readonly adapters: {
    readonly legacy: string;
    readonly legacyRecipe?: string;
  };
  readonly dialectCoverage: {
    readonly state: string;
    readonly registryRef: string | null;
    readonly evidence: readonly {
      readonly kind: string;
      readonly dialects: readonly string[];
      readonly selector: string;
    }[];
  };
}

interface DialectInputInventory {
  readonly schema: string;
  readonly version: number;
  readonly cases: readonly RegisteredDialectInput[];
}

export const COMPILER_ARGV = [
  "build",
  "Layout.vue",
  "--format",
  "json",
  "--output",
  "out",
  "--ssr",
  "--no-config",
];

const INVENTORY_REF = "tests/_fixtures/differential/dialect-inputs/inventory.json";
const TEMPLATE_TRANSPORT = "sfc-template-wrapper-v1";
const TEMPLATE_PREFIX = Buffer.from("<template>\n");
const TEMPLATE_SUFFIX = Buffer.from("</template>\n");
export const DIALECT_VITE_CONFIG_PATH = "vite.config.mjs";

export function compilerArgv(fixture: CompilerArgvFixture): string[] {
  if (!fixture.dialectInput) return [...COMPILER_ARGV];
  const argv = [
    ...COMPILER_ARGV.slice(0, -1),
    "--template-syntax",
    fixture.dialectInput.selector.templateSyntax,
  ];
  return fixture.dialectInput.selector.vueVersion === "3"
    ? [...argv, "--no-config"]
    : [...argv, "--config", DIALECT_VITE_CONFIG_PATH];
}

function dialectViteConfig(registered: RegisteredDialectInput): Buffer | null {
  if (registered.selector.vueVersion === "3") return null;
  return Buffer.from(`export default { vue: { version: "${registered.selector.vueVersion}" } };\n`);
}

export function dialectResultInput(fixture: LoadedCompilerDialectFixture) {
  return {
    id: fixture.dialectInputId,
    sha256: fixture.dialectInputSha256,
    transport: fixture.dialectTransport,
    ...(fixture.dialectConfig
      ? {
          config: {
            path: fixture.dialectConfigPath,
            sha256: fixture.dialectConfigSha256,
          },
        }
      : {}),
  };
}

export function loadDialectInput(root: string, fixture: ManifestDialectFixture) {
  const registryPath = path.resolve(root, "../dialect-inputs/inventory.json");
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as DialectInputInventory;
  assert.equal(registry.schema, "vize.dialectInputInventory");
  assert.equal(registry.version, 1);
  const registered = registry.cases.find((entry) => entry.id === fixture.inputs.dialectInput.id);
  assert(registered, `unknown dialect input: ${fixture.inputs.dialectInput.id}`);
  assert.deepEqual(fixture.inputs.dialectInput, {
    id: registered.id,
    sha256: registered.input.sha256,
    selector: registered.selector,
    expectedParserMode: registered.expectedParserMode,
    transport: TEMPLATE_TRANSPORT,
  });
  assert.equal(registered.state, "input-only");

  const vue1 = registered.selector.vueVersion === "1";
  assert.equal(
    fixture.adapters.legacy,
    vue1 ? "compiler-cli-ssr-template-v1-legacy" : "compiler-cli-ssr-template-v1",
  );
  assert.equal(fixture.adapters.legacyRecipe, vue1 ? LEGACY_BUILD_RECIPE : undefined);

  const raw = readPinnedArtifact(path.dirname(registryPath), registered.input);
  const input = Buffer.concat([TEMPLATE_PREFIX, raw, TEMPLATE_SUFFIX]);
  const config = dialectViteConfig(registered);
  const recipe = fixture.adapters.legacyRecipe ? `; build ${fixture.adapters.legacyRecipe}` : "";
  const selector = config
    ? `${fixture.adapters.legacy} Vue ${registered.selector.vueVersion} via generated ${DIALECT_VITE_CONFIG_PATH}; --template-syntax ${registered.selector.templateSyntax}; parser ${registered.expectedParserMode}${recipe}`
    : `${fixture.adapters.legacy} default Vue ${registered.selector.vueVersion}; --template-syntax ${registered.selector.templateSyntax}; parser ${registered.expectedParserMode}${recipe}`;
  assert.deepEqual(fixture.dialectCoverage, {
    state: "verified",
    registryRef: INVENTORY_REF,
    evidence: [{ kind: "adapter-execution", dialects: registered.dialects, selector }],
  });
  return { raw, input, config, registered, transport: TEMPLATE_TRANSPORT };
}
