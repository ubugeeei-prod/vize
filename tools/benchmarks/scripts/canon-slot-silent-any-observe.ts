/** Public-emission semantic ablation, not a repaired source binary or timing result. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createRequire } from "node:module";
import { identify, run, save, hash } from "./canon-instance-directives-io.ts";
import { isRecord, packet } from "./canon-instance-directives-model.ts";
import { transform } from "../../support/release/public_canonical_sdk/protocol.ts";
import { silentAnyCases } from "./canon-slot-silent-any-cases.ts";
import { queryCalibration } from "./canon-slot-silent-any-api.ts";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
function call(owner: ObjectValue, key: string, args: unknown[]): unknown {
  const fn = owner[key];
  assert.ok(typeof fn === "function");
  const result: unknown = Reflect.apply(fn, owner, args);
  return result;
}
import { diagnostics, stockCalibration } from "./canon-slot-silent-any-semantic.ts";
async function main(): Promise<void> {
  const [providerArg, outArg] = process.argv.slice(2);
  assert.ok(providerArg && outArg && process.argv.length === 4);
  const provider = path.resolve(providerArg),
    out = path.resolve(outArg);
  fs.mkdirSync(out);
  const req = createRequire(path.join(provider, "package.json"));
  const getter: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  const checker71 = string(call(object(getter), "default", []));
  const checker70 = path.join(provider, "node_modules/typescript-native-legacy/lib/tsc");
  const checkers = [
    { version: "7.0.2" as const, binary: checker70 },
    { version: "7.1.0-beta" as const, binary: checker71 },
  ];
  const identities = [
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "vue",
    "typescript",
    "typescript-native",
    "typescript-native-legacy",
    "vue-tsc",
  ].map((name) => {
    const file = path.join(provider, "node_modules", name, "package.json");
    return {
      name,
      version: object(JSON.parse(fs.readFileSync(file, "utf8"))).version,
      image: identify(file),
    };
  });
  assert.deepEqual(Object.fromEntries(identities.map((row) => [row.name, row.version])), {
    vize: "0.602.0",
    "@vizejs/native": "0.602.0",
    "@vizejs/native-linux-x64-gnu": "0.602.0",
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "typescript-native-legacy": "7.0.2",
    "vue-tsc": "3.3.13",
  });
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "NODE_OPTIONS",
    "NODE_PATH",
  ])
    assert.ok(!process.env[key]);
  const binding = object(req("@vizejs/native")),
    token = string(call(binding, "getNativeBuildToken", []));
  const identity = object(
    call(object(req("@vizejs/native/native-binding-identity.js")), "getBindingIdentity", [binding]),
  );
  assert.equal(identity.kind, "native");
  assert.equal(identity.token, token);
  assert.ok(token.length);
  const loaded = Object.keys(req.cache).filter(
    (file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine."),
  );
  assert.equal(loaded.length, 1);
  const loadedPath = loaded[0];
  assert.ok(loadedPath);
  const native = identify(loadedPath);
  const calibrated: ObjectValue[] = [];
  for (const checker of checkers) {
    const project = path.join(out, "calibration-" + checker.version);
    fs.mkdirSync(project);
    calibrated.push(
      await queryCalibration(
        checker.binary,
        project,
        path.join(out, "api-" + checker.version),
        checker.version,
      ),
    );
    if (checker.version === "7.0.2")
      stockCalibration(provider, path.join(project, "Calibration.ts"), out);
  }
  const project = path.join(out, "project");
  fs.mkdirSync(project);
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  const rows = silentAnyCases();
  const options = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    module: "ESNext",
    moduleResolution: "Bundler",
    target: "ES2024",
    lib: ["ES2024", "DOM"],
    types: [],
    incremental: false,
  };
  const config = {
    compilerOptions: options,
    vueCompilerOptions: {
      strictTemplates: true,
      checkUnknownProps: true,
      checkUnknownComponents: false,
    },
    include: ["*/src/Probe.vue", "*/component.ts"],
  };
  save(path.join(project, "tsconfig.json"), config);
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  const coreFile = path.join(provider, "node_modules/vize/generated/content-mapper-types.d.ts");
  const helpers = fs.readFileSync(
    path.join(provider, "node_modules/vize/content-mapper-helpers.d.ts"),
    "utf8",
  );
  assert.ok(helpers.includes('export type * from "./generated/content-mapper-types.js"'));
  const catalog = fs.readFileSync(coreFile, "utf8");
  const coreNames = [...catalog.matchAll(/^export type (__[A-Za-z_0-9]+)/gmu)].map((match) =>
    string(match[1]),
  );
  assert.equal(coreNames.length, 53);
  const expectedImport =
    "import type {\n" +
    coreNames.map((name) => "  " + name + ",\n").join("") +
    '} from "vize/content-mapper-helpers";';
  const emitted: ObjectValue[] = [];
  for (const row of rows) {
    const root = path.join(project, row.id),
      src = path.join(root, "src");
    fs.mkdirSync(src, { recursive: true });
    fs.writeFileSync(path.join(root, "component.ts"), row.component);
    fs.writeFileSync(path.join(src, "Probe.vue"), row.source);
    save(path.join(src, "tsconfig.json"), { compilerOptions: options, files: ["Probe.vue"] });
    const generated = transform(
      path.join(provider, "node_modules/vize/bin/vize"),
      src,
      out,
      "rpc-" + row.id,
      row.source,
      options,
      expectedImport,
    );
    const resolver = generated.text
      .split("\n")
      .find((line) => line.startsWith("type __VizeSlotHostSlots<C> = "));
    assert.ok(resolver && resolver.endsWith(" : any : any;"));
    const updatedResolver = resolver.slice(0, -"any;".length) + "unknown;";
    const patched = generated.text.replace(resolver, updatedResolver);
    const plain = generated.text.replace(
      'type __VizeSilentAny = {}["__vizeSilentAny"];',
      "type __VizeSilentAny = any;",
    );
    assert.notEqual(plain, generated.text);
    for (const [variant, text] of [
      ["original", generated.text],
      ["unknown-fallback", patched],
      ["plain-any-calibration", plain],
    ]) {
      assert.ok(variant && text);
      fs.writeFileSync(path.join(src, variant + ".tsx"), text);
    }
    emitted.push({
      id: row.id,
      family: row.family,
      stage: row.stage,
      sourceSha256: hash(row.source),
      componentSha256: hash(row.component),
      originalSha256: hash(generated.text),
      candidateSha256: hash(patched),
      exactResolverBefore: resolver,
      exactResolverAfter: updatedResolver,
      mappings: generated.mappings,
    });
  }
  const results: ObjectValue[] = [];
  const vue = run(
    out,
    "official-vue-tsc",
    process.execPath,
    [
      path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
      "--pretty",
      "false",
      "--project",
      "tsconfig.json",
    ],
    project,
  );
  results.push({
    role: "official-vue-tsc",
    diagnostics: diagnostics(vue.stdout, vue.status, project),
  });
  const stock = path.join(provider, "node_modules/typescript/lib/tsc.js");
  for (const variant of ["original", "unknown-fallback", "plain-any-calibration"]) {
    const file = path.join(project, "tsconfig." + variant + ".json");
    save(file, {
      compilerOptions: options,
      include: ["*/src/" + variant + ".tsx", "*/component.ts"],
    });
    for (const checker of [{ version: "6.0.3", binary: process.execPath }, ...checkers]) {
      const argv =
        checker.version === "6.0.3"
          ? [stock, "--pretty", "false", "--project", file]
          : ["--checkers", "1", "--pretty", "false", "--project", file];
      const observed = run(out, variant + "-" + checker.version, checker.binary, argv, project);
      results.push({
        role: variant,
        compiler: checker.version,
        image: identify(checker.binary),
        diagnostics: diagnostics(observed.stdout, observed.status, project),
      });
    }
  }
  for (const checker of checkers) {
    const observed = run(
      out,
      "public-cli-" + checker.version,
      path.join(provider, "node_modules/vize/bin/vize"),
      [
        "check",
        "--no-config",
        "--format",
        "json",
        "--show-virtual-ts",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        checker.binary,
      ],
      project,
    );
    const parsed = packet(observed.stdout, observed.status, project);
    results.push({
      role: "public-cli",
      compiler: checker.version,
      diagnostics: parsed.diagnostics,
    });
    const mapped = run(
      out,
      "public-mapper-" + checker.version,
      checker.binary,
      [
        "--runExternalCode",
        "--checkers",
        "1",
        "--pretty",
        "false",
        "--project",
        "tsconfig.mapper.json",
      ],
      project,
    );
    results.push({
      role: "public-mapper",
      compiler: checker.version,
      diagnostics: diagnostics(mapped.stdout, mapped.status, project),
    });
  }
  const report = {
    schema: "vize.slotSilentAnyAblation.v1",
    product: "public0.602.0",
    identities,
    native,
    nativeIdentity: identity,
    checkerImages: checkers.map((row) => ({ version: row.version, ...identify(row.binary) })),
    emitted,
    results,
    calibration: calibrated.map((row) => ({
      dialect: row.dialect,
      rows: row.rows,
      semantic: row.semantic,
      status: row.status,
      failure: row.failure,
    })),
    helpers: { core: identify(coreFile), entrySha256: hash(helpers) },
    timingUsable: false,
    sourceRepairQualified: false,
    original11018ConservationGate:
      "RED unchanged; this is independent helper semantic classification",
  };
  save(path.join(out, "summary.json"), report);
  fs.copyFileSync(path.join(provider, "package-lock.json"), path.join(out, "package-lock.json"));
  fs.copyFileSync(path.join(provider, "package.json"), path.join(out, "package.json"));
  assert.equal(
    object(JSON.parse(fs.readFileSync(req.resolve("vue/package.json"), "utf8"))).version,
    "3.5.42",
  );
}
await main();
