/** Independent public Vue/TS6/VueTSC contract oracle; no Rust source build or repair qualification. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { run, save, identify, hash } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue, packet } from "./canon-instance-directives-model.ts";
type Case = { id: string; component: string; source: string; requiresAuthoredError: boolean };
function cases(value: unknown): value is Case[] {
  return (
    Array.isArray(value) &&
    value.every(
      (row: unknown) =>
        isRecord(row) &&
        typeof row.id === "string" &&
        typeof row.component === "string" &&
        typeof row.source === "string" &&
        typeof row.requiresAuthoredError === "boolean",
    )
  );
}
function diagnostics(bytes: Buffer, status: number | null, project: string) {
  const source = new TextDecoder("utf8", { fatal: true }).decode(bytes),
    result = [];
  for (const match of source.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)) {
    const [, file, line, column, code, message] = match;
    assert.ok(file && line && column && code && message !== undefined);
    result.push({
      file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
      line: +line,
      column: +column,
      code: +code,
      message,
    });
  }
  assert.equal(result.length, (source.match(/error TS\d+:/gu) ?? []).length);
  assert.equal(status, result.length ? 2 : 0);
  return result;
}
async function main() {
  const [providerArg, driverArg, outArg] = process.argv.slice(2);
  assert.ok(providerArg && driverArg && outArg);
  const provider = path.resolve(providerArg),
    driver = path.resolve(driverArg),
    out = path.resolve(outArg);
  fs.mkdirSync(out);
  const fixtureFile = path.join(driver, "tools/benchmarks/scripts/canon-slot-constructor-cases.ts");
  const fixtureBytes = fs.readFileSync(fixtureFile);
  const fixtureBlob = createHash("sha1")
    .update("blob " + fixtureBytes.length + "\0")
    .update(fixtureBytes)
    .digest("hex");
  assert.equal(fixtureBlob, "d34ce4bf37ea26ac8a9120c8bd6f66c79346d86f");
  const imported: unknown = await import(pathToFileURL(fixtureFile).href);
  assert.ok(isRecord(imported) && typeof imported.slotCases === "function");
  const value: unknown = Reflect.apply(imported.slotCases, imported, []);
  assert.ok(cases(value));
  const rows = [...value];
  const zero = rows.find((row) => row.id === "zero");
  assert.ok(zero);
  const publicInternal = zero.component.replace(
    "props: { value: { type: Number, required: true },",
    "props: { internal: { type: String, required: true }, value: { type: Number, required: true },",
  );
  assert.notEqual(publicInternal, zero.component);
  rows.push({
    id: "required-public-internal",
    component: publicInternal,
    source: zero.source,
    requiresAuthoredError: true,
  });
  const nativeComponent = zero.component.replace(
    "export declare const WitnessChild: { new(): Instance };",
    "export const WitnessChild = VueChild;",
  );
  assert.notEqual(nativeComponent, zero.component);
  for (const [id, binding, tag] of [
    ["pascal", "WitnessChild", "WitnessChild"],
    ["camel", "witnessChild", "witnessChild"],
    ["kebab", "witnessChild", "witness-child"],
  ]) {
    assert.ok(id && binding && tag);
    for (const stage of ["valid", "invalid"]) {
      let source = zero.source
        .replace("import WitnessChild from", "import " + binding + " from")
        .replaceAll("<WitnessChild", "<" + tag)
        .replaceAll("</WitnessChild", "</" + tag);
      if (stage === "invalid") source = source.replace(':value="1"', ":value=\"'bad'\"");
      rows.push({
        id: "casing-" + id + "-" + stage,
        component: nativeComponent,
        source,
        requiresAuthoredError: stage === "invalid",
      });
    }
  }
  const project = path.join(out, "project");
  fs.mkdirSync(project);
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  const manifest = [];
  for (const row of rows) {
    for (const [file, source] of [
      ["component.ts", row.component],
      ["src/Tiny.vue", row.source],
    ]) {
      assert.ok(file && source);
      const relative = path.join(row.id, file),
        absolute = path.join(project, relative),
        capture = path.join(out, "inputs", relative);
      fs.mkdirSync(path.dirname(absolute), { recursive: true });
      fs.mkdirSync(path.dirname(capture), { recursive: true });
      fs.writeFileSync(absolute, source);
      fs.writeFileSync(capture, source);
      manifest.push({ file: relative, bytes: Buffer.byteLength(source), sha256: hash(source) });
    }
  }
  const config = {
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ES2024",
      lib: ["ES2024", "DOM"],
      types: [],
      incremental: false,
    },
    vueCompilerOptions: {
      strictTemplates: true,
      checkUnknownProps: true,
      checkUnknownComponents: false,
    },
    include: ["**/*.vue", "**/component.ts"],
  };
  save(path.join(project, "tsconfig.json"), config);
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  const hSource = ["import { h, type Component } from 'vue';"];
  const hOwners: { id: string; start: number; end: number }[] = [];
  for (const [index, row] of rows.entries()) {
    if (
      ![
        "zero",
        "constructor-public-props-discrepancy",
        "constructor-public-type-discrepancy",
        "required-public-internal",
      ].includes(row.id)
    )
      continue;
    const firstLine = hSource.length + 1;
    hSource.push(
      "import C" + index + " from './" + row.id + "/component';",
      "const component" + index + ":Component=C" + index + ";",
      "const props" + index + ":InstanceType<typeof C" + index + ">['$props']={value:1};",
      "h(C" + index + ", props" + index + ");",
      "void component" + index + ";",
    );
    hOwners.push({ id: row.id, start: firstLine, end: hSource.length });
  }
  fs.writeFileSync(path.join(project, "authored_h.ts"), hSource.join("\n") + "\n");
  save(path.join(project, "tsconfig.h.json"), {
    ...config,
    include: ["authored_h.ts", "**/component.ts"],
  });
  const getter: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  assert.ok(isRecord(getter) && typeof getter.default === "function");
  const checker: unknown = Reflect.apply(getter.default, getter, []);
  assert.ok(typeof checker === "string");
  const identityFiles = [
    "package-lock.json",
    ...[
      "vize",
      "@vizejs/native",
      "@vizejs/native-linux-x64-gnu",
      "vue",
      "typescript",
      "typescript-native",
      "vue-tsc",
    ].map((name) => "node_modules/" + name + "/package.json"),
    "node_modules/vize/content-mapper-helpers.d.ts",
    "node_modules/vize/generated/content-mapper-types.d.ts",
    "node_modules/@vizejs/native-linux-x64-gnu/vize-vitrine.linux-x64-gnu.node",
  ];
  const identities = identityFiles.map((file) => ({
    file,
    ...identify(path.join(provider, file)),
  }));
  const versions: Record<string, unknown> = {};
  for (const name of ["vize", "vue", "typescript", "typescript-native", "vue-tsc"]) {
    const p = jsonValue(
      fs.readFileSync(path.join(provider, "node_modules", name, "package.json"), "utf8"),
    );
    assert.ok(isRecord(p));
    versions[name] = p.version;
  }
  assert.deepEqual(versions, {
    vize: "0.602.0",
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "vue-tsc": "3.3.13",
  });
  const stockFile = path.join(provider, "node_modules/typescript/lib/tsc.js"),
    vueFile = path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
    publicCli = path.join(provider, "node_modules/vize/bin/vize");
  const vueRequire = createRequire(vueFile),
    vueTs = vueRequire.resolve("typescript/package.json");
  const compilerManifest = jsonValue(fs.readFileSync(vueTs, "utf8"));
  assert.ok(isRecord(compilerManifest) && compilerManifest.version === "6.0.3");
  assert.equal(
    fs.realpathSync(vueTs),
    fs.realpathSync(path.join(provider, "node_modules/typescript/package.json")),
  );
  for (const file of [
    "authored_h.ts",
    "tsconfig.json",
    "tsconfig.mapper.json",
    "tsconfig.h.json",
  ]) {
    const bytes = fs.readFileSync(path.join(project, file));
    fs.writeFileSync(path.join(out, "inputs", file), bytes);
    manifest.push({ file, bytes: bytes.length, sha256: hash(bytes) });
  }
  const h = run(
    out,
    "stock-h",
    process.execPath,
    [stockFile, "--pretty", "false", "--project", "tsconfig.h.json"],
    project,
  );
  const hDiagnostics = diagnostics(h.stdout, h.status, project);
  const official = run(
    out,
    "official-vue-tsc",
    process.execPath,
    [vueFile, "--pretty", "false", "--project", "tsconfig.json"],
    project,
  );
  const officialDiagnostics = diagnostics(official.stdout, official.status, project);
  const mapper = run(
    out,
    "published-mapper",
    checker,
    [
      "--runExternalCode",
      "--checkers",
      "1",
      "--noEmit",
      "--pretty",
      "false",
      "--project",
      "tsconfig.mapper.json",
    ],
    project,
  );
  const mapperDiagnostics = diagnostics(mapper.stdout, mapper.status, project);
  const cli = run(
    out,
    "published-cli",
    publicCli,
    [
      "check",
      "--no-config",
      "--format",
      "json",
      "--show-virtual-ts",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      checker,
    ],
    project,
  );
  const cliPacket = packet(cli.stdout, cli.status, project);
  const byCase = rows.map((row) => ({
    id: row.id,
    requiresAuthoredError: row.requiresAuthoredError,
    official: officialDiagnostics.filter((d) => d.file.startsWith(row.id + "/")),
    mapper: mapperDiagnostics.filter((d) => d.file.startsWith(row.id + "/")),
    cli: cliPacket.diagnostics.filter((d) => d.file.startsWith(row.id + "/")),
  }));
  for (const entry of identities)
    assert.deepEqual(identify(path.join(provider, entry.file)), {
      path: entry.path,
      bytes: entry.bytes,
      sha256: entry.sha256,
    });
  const summary = {
    schema: 1,
    role: "independent semantic classification, not repair or native-phase performance qualification",
    fixtureDriver: {
      source: "190257dbe52a7d4942be4496403a55b32c24426f",
      blob: fixtureBlob,
      identity: identify(fixtureFile),
    },
    versions,
    identities,
    checker: identify(checker),
    stock: identify(stockFile),
    vueTsc: identify(vueFile),
    vueTscTypeScript: identify(vueTs),
    physicalVue: fs.realpathSync(path.join(project, "node_modules/vue/package.json")),
    manifest,
    hDiagnostics,
    hOwners,
    hSource: hSource.join("\n") + "\n",
    byCase,
    timingUsable: false,
    claimedRepair: false,
    previousUnsafeConservationGate: "failed; not waived or repinned",
  };
  save(path.join(out, "summary.json"), summary);
  const missing = byCase.filter(
    (row) =>
      row.requiresAuthoredError && (!row.official.length || !row.cli.length || !row.mapper.length),
  );
  save(path.join(out, "missing-invalid-probes.json"), missing);
  assert.equal(
    missing.length,
    0,
    "intended invalid probes must be independently diagnosed, not accepted from zero output",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
