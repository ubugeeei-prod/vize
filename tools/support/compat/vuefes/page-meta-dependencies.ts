import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { officialPageMeta, pageMetaOracleIdentity } from "./page-meta-oracle.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(
  root,
  "tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies",
);
const requireOracle = createRequire(
  new URL("./fixtures/heading-cascade/package.json", import.meta.url),
);
const compiler = requireOracle("@vue/compiler-sfc");
assert.equal(compiler.version, "3.6.0-alpha.2");
const fromNuxt = createRequire(
  new URL("./fixtures/app-component-order/package.json", import.meta.url),
);
const { parseSync } = fromNuxt("oxc-parser");
const cases = JSON.parse(fs.readFileSync(path.join(directory, "dependencies.json")));
const output = path.join(root, "target/vuefes-ssr-style-dependencies/page-meta-dependencies");
fs.mkdirSync(output, { recursive: true });
const write = (name, value) =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");

function statements(code) {
  const parsed = parseSync("metadata.ts", code);
  assert.deepEqual(parsed.errors, []);
  return parsed.program.body.map((node) => ({
    type: node.type,
    start: node.start,
    end: node.end,
    source: code.slice(node.start, node.end),
  }));
}

async function evaluate(code, fixture) {
  const trace = [];
  const context = vm.createContext({});
  const module = new vm.SourceTextModule(code, { context });
  const dependencies = new Map();
  await module.link((specifier) => {
    if (!dependencies.has(specifier)) {
      const exports = fixture.modules[specifier];
      assert(exports, `unexpected retained side effect: ${specifier}`);
      dependencies.set(
        specifier,
        new vm.SyntheticModule(
          Object.keys(exports),
          function () {
            trace.push(specifier);
            for (const [name, value] of Object.entries(exports)) this.setExport(name, value);
          },
          { context },
        ),
      );
    }
    return dependencies.get(specifier);
  });
  await module.evaluate();
  const metadata = module.namespace.default;
  const fields = Object.keys(metadata).map((key) => ({ key, type: typeof metadata[key] }));
  const data = JSON.parse(JSON.stringify(metadata));
  const calls = (fixture.calls ?? []).map(({ key, args }) => ({
    key,
    args,
    value: metadata[key](...args),
  }));
  return { trace, fields, data, calls };
}

export async function collectOfficialMetadataCases() {
  const observations = [];
  for (const fixture of cases) {
    const parsed = compiler.parse(fixture.source, { filename: `${fixture.name}.vue` });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, fixture.source);
    const script = compiler.compileScript(parsed.descriptor, { id: fixture.name });
    const official = await officialPageMeta(script.content);
    const observation = {
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
      observation.runtimeRefusal = { name: error.name, message: error.message, stack: error.stack };
      write("official.json", { identity: pageMetaOracleIdentity, observations });
      throw error;
    }
  }
  return { identity: { ...pageMetaOracleIdentity, vueCompiler: compiler.version }, observations };
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
    JSON.parse(fs.readFileSync(path.join(directory, "official.dependencies.json"))),
    "the complete fresh pinned official packet equals the retained corpus",
  );
  if (process.argv.includes("--check-oracle")) {
    process.stdout.write(`${cases.length} complete fresh pinned Nuxt metadata oracles passed\n`);
    process.exit(0);
  }
  const packetIndex = process.argv.indexOf("--native-packet");
  let compileNative;
  if (packetIndex >= 0) {
    const packetPath = process.argv[packetIndex + 1];
    assert(packetPath, "an actual complete Rust public-adapter packet is required");
    const bytes = fs.readFileSync(packetPath);
    const packet = JSON.parse(bytes);
    assert.equal(packet.actual.length, cases.length);
    write("native-identity.json", {
      boundary: "captured Rust public-adapter compiler packet; not a NAPI build qualification",
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
    compileNative = (fixture, index) => {
      assert.deepEqual(packet.actual[index].fixture, fixture);
      return packet.actual[index].compileResult;
    };
  } else {
    const requireNative = createRequire(import.meta.url);
    const bindings = fs
      .readdirSync(path.join(root, "npm/native"))
      .filter((name) => /^vize-vitrine\..*\.node$/.test(name));
    assert.equal(bindings.length, 1, "one existing freshly built native binding");
    const nativePath = path.join(root, "npm/native", bindings[0]);
    const nativeBytes = fs.readFileSync(nativePath);
    write("native-identity.json", {
      source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
      filename: bindings[0],
      bytes: nativeBytes.length,
      sha256: createHash("sha256").update(nativeBytes).digest("hex"),
      boundary: "existing single build:ci producer binding; this differential performs no build",
    });
    const native = requireNative(nativePath);
    compileNative = (fixture) =>
      native.compileSfc(fixture.source, {
        filename: `${fixture.name}.vue`,
        nuxtPageMeta: true,
      });
  }
  const nativePacket = [];
  for (const [index, fixture] of cases.entries()) {
    const compiled = compileNative(fixture, index);
    const observation = { ...fixture, compileResult: compiled };
    nativePacket.push(observation);
    write("native.json", nativePacket);
    assert.deepEqual(compiled.errors, []);
    assert.equal(compiled.macroArtifacts.length, 1);
    const code = compiled.macroArtifacts[0].moduleCode;
    observation.statements = statements(code);
    observation.runtime = await evaluate(code, fixture);
    write("native.json", nativePacket);
    const body = (rows) => rows.map(({ type, source }) => ({ type, source }));
    const reference = officialPacket.observations[index];
    if (fixture.oracleShadowingLimit) {
      assert.equal(code, fixture.expectedNativeModuleCode);
      assert.deepEqual(observation.runtime.trace, fixture.requiredImports);
      assert.deepEqual(reference.runtime.trace, fixture.oracleImports);
      for (const key of ["fields", "data", "calls"])
        assert.deepEqual(observation.runtime[key], reference.runtime[key]);
    } else {
      assert.deepEqual(body(observation.statements), body(reference.statements));
      assert.deepEqual(observation.runtime, reference.runtime);
    }
  }
  process.stdout.write(`${cases.length} complete native/Nuxt metadata dependency cases passed\n`);
}
