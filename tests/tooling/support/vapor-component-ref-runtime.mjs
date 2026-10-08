// Full original component-ref SFCs and independently compiled pinned Vue controls.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const evidence = { input, stock: [], maps: [], observations: [], qualified: false };
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
try {
  assert.equal(typeof input.production, "boolean");
  assert.equal(typeof input.inline, "boolean");
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc");
  assert.equal(compiler.version, "3.6.0-rc.9");
  evidence.versions = { compiler: compiler.version, runtime: fromVue("./package.json").version };
  assert.equal(evidence.versions.runtime, compiler.version);
  evidence.versions.compilerEntrySha256 = hash(readFileSync(fromVue.resolve("vue/compiler-sfc")));
  evidence.versions.runtimeManifestSha256 = hash(
    readFileSync(fromUi.resolve("vue-vapor-runtime/package.json")),
  );
  const oldRoot = new URL(
    "../../_fixtures/differential/compiler/vapor-for-template-refs-7882/",
    import.meta.url,
  );
  const manifest = JSON.parse(readFileSync(new URL("manifest.json", oldRoot)));
  for (const [name, source] of [
    ["App.vue.txt", input.cases[0].source],
    ["Child.vue.txt", input.child.source],
  ]) {
    const bytes = readFileSync(new URL(name, oldRoot));
    assert.equal(bytes.length, manifest.files[name].bytes);
    assert.equal(hash(bytes), manifest.files[name].sha256);
    assert.equal(source, bytes.toString("utf8"));
  }
  const ownRoot = new URL(
    "../../_fixtures/differential/compiler/vapor-component-template-refs-7882/",
    import.meta.url,
  );
  const ownManifest = JSON.parse(readFileSync(new URL("manifest.json", ownRoot)));
  const expectedBytes = readFileSync(new URL(ownManifest.reference.path, ownRoot));
  assert.equal(hash(expectedBytes), ownManifest.reference.sha256);
  evidence.planOracle = { manifest: ownManifest, expected: expectedBytes.toString("utf8") };
  const plans = JSON.parse(expectedBytes.toString("utf8"));
  assert.deepEqual(
    input.cases.map(({ name }) => name),
    ["original", "loop-and-scalar", "computed-prop-and-callback"],
  );
  assert.deepEqual(
    Object.keys(plans),
    input.cases.map(({ name }) => name),
  );
  assert.deepEqual(Object.keys(ownManifest.files), [
    "computed-prop-and-callback.vue.txt",
    "loop-and-scalar.vue.txt",
  ]);
  for (const fixture of input.cases.slice(1)) {
    const name = `${fixture.filename}.txt`;
    const bytes = readFileSync(new URL(name, ownRoot));
    assert.equal(bytes.length, ownManifest.files[name].bytes);
    assert.equal(hash(bytes), ownManifest.files[name].sha256);
    assert.equal(fixture.source, bytes.toString("utf8"));
  }

  function actual(fixture) {
    const packet = fixture.compiled;
    assert.equal(packet.parseError, null);
    assert.deepEqual(Object.keys(packet.plain), ["Ok"]);
    assert.deepEqual(Object.keys(packet.mapped), ["Ok"]);
    const plain = packet.plain.Ok;
    const mapped = packet.mapped.Ok;
    assert.deepEqual(plain.errors, []);
    assert.deepEqual(plain.warnings, []);
    assert.equal(plain.map, null);
    assert.deepEqual({ ...mapped, map: null }, plain, "all public map-on/off fields");
    assert.equal(mapped.map.version, 3);
    assert.deepEqual(mapped.map.sources, [fixture.filename]);
    assert.deepEqual(mapped.map.sourcesContent, [fixture.source]);
    assert.ok(mapped.map.mappings.length > 0);
    evidence.maps.push({ name: fixture.filename, map: mapped.map });
    return plain.code;
  }
  function official(fixture) {
    const parsed = compiler.parse(fixture.source, { filename: fixture.filename });
    const packet = {
      name: fixture.filename,
      source: fixture.source,
      recipe: { production: input.production, inline: true },
      parseErrors: parsed.errors,
    };
    evidence.stock.push(packet);
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, fixture.source);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.filename,
      isProd: input.production,
      inlineTemplate: true,
      genDefaultAs: "__reference",
    });
    const code = `${script.content}\nexport default __reference;`;
    Object.assign(packet, {
      code,
      map: script.map,
      bindings: script.bindings,
      scriptResult: script,
    });
    return code;
  }
  function mounted(kind, name, code, child, plan) {
    const request = {
      backend: "vapor",
      production: input.production,
      code,
      child,
      steps: plan.steps,
      ...(plan.exposedReads ? { exposedReads: plan.exposedReads } : {}),
    };
    const runner = new URL("./vapor-sfc-runtime.mjs", import.meta.url);
    const result = spawnSync(process.execPath, [runner.pathname], {
      input: JSON.stringify(request),
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    });
    const packet = {
      kind,
      name,
      request,
      status: result.status,
      signal: result.signal,
      error: result.error
        ? Object.assign({}, result.error, {
            message: result.error.message,
            stack: result.error.stack,
          })
        : null,
      stdout: result.stdout ?? null,
      stderr: result.stderr ?? null,
    };
    evidence.observations.push(packet);
    assert.equal(packet.error, null);
    assert.equal(packet.status, 0);
    assert.equal(packet.signal, null);
    assert.equal(packet.stderr, "");
    assert.deepEqual(JSON.parse(packet.stdout), plan.expected, `${kind}/${name}`);
  }
  const child = actual(input.child);
  const stockChild = official(input.child);
  for (const fixture of input.cases) {
    const plan = plans[fixture.name];
    const stock = official(fixture);
    mounted("official", fixture.name, stock, stockChild, plan);
    mounted("current", fixture.name, actual(fixture), child, plan);
  }
  evidence.qualified = true;
} catch (error) {
  evidence.failure = { ...error, name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  process.stdout.write(`${JSON.stringify(evidence)}\n`);
}
