import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { officialPageMeta, pageMetaOracleIdentity } from "./page-meta-oracle.ts";
import {
  errorText,
  list,
  record,
  text,
  type CompilerSfc,
  type JsonRecord,
} from "./oracle-support.ts";

interface MetadataCall {
  readonly key: string;
  readonly args: readonly unknown[];
}

interface MetadataFixture extends JsonRecord {
  readonly name: string;
  readonly source: string;
  readonly modules: Readonly<Record<string, JsonRecord>>;
  readonly calls?: readonly MetadataCall[];
  readonly oracleShadowingLimit?: unknown;
  readonly expectedNativeModuleCode?: unknown;
  readonly requiredImports?: unknown;
  readonly oracleImports?: unknown;
}

interface Statement {
  readonly type: string;
  readonly start: number;
  readonly end: number;
  readonly source: string;
}

interface RuntimeObservation {
  readonly trace: string[];
  readonly fields: { key: string; type: string }[];
  readonly data: unknown;
  readonly calls: { key: string; args: readonly unknown[]; value: unknown }[];
}

interface NativeCompileResult {
  readonly errors: readonly unknown[];
  readonly macroArtifacts: readonly { moduleCode: string }[];
}

type OxcParseSync = (
  filename: string,
  code: string,
) => {
  errors: readonly unknown[];
  program: { body: readonly { type: string; start: number; end: number }[] };
};

function metadataFixture(value: unknown, index: number): MetadataFixture {
  const fixture = record(value, `dependencies[${index}]`);
  text(fixture.name, `dependencies[${index}].name`);
  text(fixture.source, `dependencies[${index}].source`);
  const modules = record(fixture.modules, `dependencies[${index}].modules`);
  for (const [specifier, exports] of Object.entries(modules)) record(exports, specifier);
  if (fixture.calls !== undefined)
    for (const call of list(fixture.calls, "calls")) {
      const { key, args } = record(call, "call");
      text(key, "call.key");
      list(args, "call.args");
    }
  return fixture as MetadataFixture;
}

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(
  root,
  "tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies",
);
const requireOracle = createRequire(
  new URL("./fixtures/heading-cascade/package.json", import.meta.url),
);
const compiler: CompilerSfc = requireOracle("@vue/compiler-sfc");
assert.equal(compiler.version, "3.6.0-alpha.2");
const fromNuxt = createRequire(
  new URL("./fixtures/app-component-order/package.json", import.meta.url),
);
const { parseSync }: { parseSync: OxcParseSync } = fromNuxt("oxc-parser");
const cases = list(
  JSON.parse(fs.readFileSync(path.join(directory, "dependencies.json"), "utf8")),
  "dependencies.json",
).map(metadataFixture);
const output = path.join(root, "target/vuefes-ssr-style-dependencies/page-meta-dependencies");
fs.mkdirSync(output, { recursive: true });
const write = (name: string, value: unknown): void =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");

function statements(code: string): Statement[] {
  const parsed = parseSync("metadata.ts", code);
  assert.deepEqual(parsed.errors, []);
  return parsed.program.body.map((node) => ({
    type: node.type,
    start: node.start,
    end: node.end,
    source: code.slice(node.start, node.end),
  }));
}

async function evaluate(code: string, fixture: MetadataFixture): Promise<RuntimeObservation> {
  const trace: string[] = [];
  const context = vm.createContext({});
  const module = new vm.SourceTextModule(code, { context });
  const dependencies = new Map<string, vm.SyntheticModule>();
  await module.link((specifier) => {
    let dependency = dependencies.get(specifier);
    if (!dependency) {
      const exports = fixture.modules[specifier];
      assert(exports, `unexpected retained side effect: ${specifier}`);
      dependency = new vm.SyntheticModule(
        Object.keys(exports),
        function (this: vm.SyntheticModule) {
          trace.push(specifier);
          for (const [name, value] of Object.entries(exports)) this.setExport(name, value);
        },
        { context },
      );
      dependencies.set(specifier, dependency);
    }
    return dependency;
  });
  await module.evaluate();
  const metadata = record(record(module.namespace, "module namespace").default, "page metadata");
  const fields = Object.keys(metadata).map((key) => ({ key, type: typeof metadata[key] }));
  const data: unknown = JSON.parse(JSON.stringify(metadata));
  const calls = (fixture.calls ?? []).map(({ key, args }) => {
    const method = metadata[key];
    assert.equal(typeof method, "function", `metadata.${key} is callable`);
    return { key, args, value: (method as (...values: readonly unknown[]) => unknown)(...args) };
  });
  return { trace, fields, data, calls };
}

interface OfficialObservation extends MetadataFixture {
  readonly statements: Statement[];
  runtime?: RuntimeObservation;
  runtimeRefusal?: { name: string; message: string; stack: string | undefined };
}

export async function collectOfficialMetadataCases(): Promise<{
  identity: typeof pageMetaOracleIdentity & { vueCompiler: string };
  observations: OfficialObservation[];
}> {
  const observations: OfficialObservation[] = [];
  for (const fixture of cases) {
    const parsed = compiler.parse(fixture.source, { filename: `${fixture.name}.vue` });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, fixture.source);
    const script = compiler.compileScript(parsed.descriptor, { id: fixture.name });
    const official = await officialPageMeta(script.content);
    const observation: OfficialObservation = {
      ...fixture,
      fullOfficialScript: script,
      official,
      statements: statements(official.code),
    };
    observations.push(observation);
    write("official.json", { identity: pageMetaOracleIdentity, observations });
    try {
      observation.runtime = await evaluate(official.code, fixture);
    } catch (error) {
      observation.runtimeRefusal =
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : { name: "NonError", message: errorText(error), stack: undefined };
      write("official.json", { identity: pageMetaOracleIdentity, observations });
      throw error;
    }
  }
  return { identity: { ...pageMetaOracleIdentity, vueCompiler: compiler.version }, observations };
}

function nativeCompileResult(value: unknown): NativeCompileResult {
  const result = record(value, "compileResult");
  return {
    errors: list(result.errors, "compileResult.errors"),
    macroArtifacts: list(result.macroArtifacts, "compileResult.macroArtifacts").map(
      (artifact, index) => ({
        moduleCode: text(
          record(artifact, `macroArtifacts[${index}]`).moduleCode,
          `macroArtifacts[${index}].moduleCode`,
        ),
      }),
    ),
  };
}

function nativeCompiler(): (fixture: MetadataFixture, index: number) => unknown {
  const packetIndex = process.argv.indexOf("--native-packet");
  if (packetIndex >= 0) {
    const packetPath = process.argv[packetIndex + 1];
    assert(packetPath, "an actual complete Rust public-adapter packet is required");
    const bytes = fs.readFileSync(packetPath);
    const actual = list(record(JSON.parse(bytes.toString("utf8")), "packet").actual, "actual");
    assert.equal(actual.length, cases.length);
    write("native-identity.json", {
      boundary: "captured Rust public-adapter compiler packet; not a NAPI build qualification",
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
    return (fixture, index) => {
      const entry = record(actual[index], `actual[${index}]`);
      assert.deepEqual(entry.fixture, fixture);
      return entry.compileResult;
    };
  }
  const requireNative = createRequire(import.meta.url);
  const bindings = fs
    .readdirSync(path.join(root, "npm/native"))
    .filter((name) => /^vize-vitrine\..*\.node$/.test(name));
  assert.equal(bindings.length, 1, "one existing freshly built native binding");
  const [binding = ""] = bindings;
  const nativePath = path.join(root, "npm/native", binding);
  const nativeBytes = fs.readFileSync(nativePath);
  write("native-identity.json", {
    source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
    filename: binding,
    bytes: nativeBytes.length,
    sha256: createHash("sha256").update(nativeBytes).digest("hex"),
    boundary: "existing single build:ci producer binding; this differential performs no build",
  });
  const native: {
    compileSfc(source: string, options: { filename: string; nuxtPageMeta: boolean }): unknown;
  } = requireNative(nativePath);
  return (fixture) =>
    native.compileSfc(fixture.source, {
      filename: `${fixture.name}.vue`,
      nuxtPageMeta: true,
    });
}

const officialPacket = await collectOfficialMetadataCases();
write("official.json", officialPacket);
if (process.argv.includes("--write-oracle")) {
  fs.writeFileSync(
    path.join(directory, "official.dependencies.json"),
    JSON.stringify(officialPacket, null, 2) + "\n",
  );
} else {
  assert.deepEqual(
    JSON.parse(JSON.stringify(officialPacket)),
    JSON.parse(fs.readFileSync(path.join(directory, "official.dependencies.json"), "utf8")),
    "the complete fresh pinned official packet equals the retained corpus",
  );
  if (process.argv.includes("--check-oracle")) {
    process.stdout.write(`${cases.length} complete fresh pinned Nuxt metadata oracles passed\n`);
    process.exit(0);
  }
  const compileNative = nativeCompiler();
  const nativePacket: unknown[] = [];
  for (const [index, fixture] of cases.entries()) {
    const compileResult = compileNative(fixture, index);
    const observation: {
      compileResult: unknown;
      statements?: Statement[];
      runtime?: RuntimeObservation;
    } = { ...fixture, compileResult };
    nativePacket.push(observation);
    write("native.json", nativePacket);
    const compiled = nativeCompileResult(compileResult);
    assert.deepEqual(compiled.errors, []);
    assert.equal(compiled.macroArtifacts.length, 1);
    const code = compiled.macroArtifacts[0]?.moduleCode ?? "";
    observation.statements = statements(code);
    const runtime = await evaluate(code, fixture);
    observation.runtime = runtime;
    write("native.json", nativePacket);
    const body = (rows: readonly Statement[]) => rows.map(({ type, source }) => ({ type, source }));
    const reference = officialPacket.observations[index];
    assert.ok(reference?.runtime, "the official packet has a runtime observation per case");
    if (fixture.oracleShadowingLimit) {
      assert.equal(code, fixture.expectedNativeModuleCode);
      assert.deepEqual(runtime.trace, fixture.requiredImports);
      assert.deepEqual(reference.runtime.trace, fixture.oracleImports);
      for (const key of ["fields", "data", "calls"] as const)
        assert.deepEqual(runtime[key], reference.runtime[key]);
    } else {
      assert.deepEqual(body(observation.statements), body(reference.statements));
      assert.deepEqual(runtime, reference.runtime);
    }
  }
  process.stdout.write(`${cases.length} complete native/Nuxt metadata dependency cases passed\n`);
}
