/** Qualify the actual source CLI and Go7.1 hosted mapper against frozen authored fixtures. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createRequire } from "node:module";
import { identify, run, save, hash, wrapperSource } from "./canon-instance-directives-io.ts";
import { isRecord, packet } from "./canon-instance-directives-model.ts";
import { sourceSdk, nativeDiagnostics } from "./canon-component-casing-observe.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { invertUnknownSlotText, assertUnknownSlotPacket } from "./canon-slot-unknown-packet.ts";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code: number | null;
  message: string;
};
const order = (rows: Diagnostic[]) =>
  [...rows].sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
async function main(): Promise<void> {
  const [providerArg, binaryArg, fixturesArg, outArg] = process.argv.slice(2);
  assert.ok(providerArg && binaryArg && fixturesArg && outArg && process.argv.length === 6);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    fixtures = path.resolve(fixturesArg),
    out = path.resolve(outArg);
  fs.mkdirSync(out);
  const project = path.join(out, "project");
  fs.mkdirSync(project);
  const nativeReq = createRequire(path.join(provider, "package.json"));
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "NODE_OPTIONS",
    "NODE_PATH",
  ])
    assert.ok(!process.env[key]);
  const loaded = object(nativeReq("@vizejs/native"));
  const tokenFn = loaded.getNativeBuildToken;
  assert.ok(typeof tokenFn === "function");
  const token: unknown = Reflect.apply(tokenFn, loaded, []);
  assert.ok(typeof token === "string" && token.length > 0);
  const nativeFiles = Object.keys(nativeReq.cache).filter(
    (file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine."),
  );
  assert.equal(nativeFiles.length, 1);
  const nativeFile = nativeFiles[0];
  assert.ok(nativeFile);
  const identityOwner = object(nativeReq("@vizejs/native/native-binding-identity.js"));
  const identityFn = identityOwner.getBindingIdentity;
  assert.ok(typeof identityFn === "function");
  const nativeIdentity = object(Reflect.apply(identityFn, identityOwner, [loaded]));
  assert.equal(nativeIdentity.kind, "native");
  assert.equal(nativeIdentity.token, token);
  const nativeImage = identify(nativeFile),
    sourceImage = identify(binary);
  const versions = Object.fromEntries(
    ["vize", "@vizejs/native", "vue", "typescript", "typescript-native", "vue-tsc"].map((name) => {
      const metadata = object(
        JSON.parse(
          fs.readFileSync(path.join(provider, "node_modules", name, "package.json"), "utf8"),
        ),
      );
      return [name, metadata.version];
    }),
  );
  assert.deepEqual(versions, {
    vize: "0.602.0",
    "@vizejs/native": "0.602.0",
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "vue-tsc": "3.3.13",
  });
  const getter: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  const get = object(getter).default;
  assert.ok(typeof get === "function");
  const checkerValue: unknown = Reflect.apply(get, getter, []);
  const checker = string(checkerValue);
  const checkerImage = identify(checker),
    cli = path.join(provider, "node_modules/vize/bin/vize");
  const fixtureBytes = fs.readFileSync(path.join(fixtures, "cases.json"));
  assert.equal(
    hash(fixtureBytes),
    "3f0f0b29e3ed2ad7239347ea57839553a830538af72abcc77ddb25c05cf18a23",
  );
  const pack = object(JSON.parse(fixtureBytes.toString("utf8"))),
    rows = array(pack.cases).map(object);
  assert.equal(rows.length, 10);
  const expected: Diagnostic[] = [],
    sources = new Map<string, string>(),
    inputs = [];
  for (const row of rows) {
    const id = string(row.id);
    for (const input of array(row.inputs).map(object)) {
      const relative = string(input.file),
        file = path.join(project, id, relative),
        bytes = fs.readFileSync(path.join(fixtures, string(input.source)));
      assert.ok(!relative.split("/").includes("..") && !path.isAbsolute(relative));
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, bytes);
      inputs.push({ file: id + "/" + relative, bytes: bytes.length, sha256: hash(bytes) });
      if (relative === "src/Tiny.vue") sources.set(id, bytes.toString("utf8"));
    }
    for (const value of array(row.diagnostics)) {
      const d = object(value);
      assert.equal(d.severity, 1);
      assert.ok(
        typeof d.line === "number" && typeof d.column === "number" && typeof d.code === "number",
      );
      expected.push({
        file: id + "/" + string(d.file),
        line: d.line,
        column: d.column,
        code: d.code,
        message: string(d.message),
      });
    }
    const config = object(
      JSON.parse(fs.readFileSync(path.join(project, id, "tsconfig.json"), "utf8")),
    );
    save(path.join(project, id, "tsconfig.mapper.json"), {
      ...config,
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    });
  }
  const originalConfig = object(
    JSON.parse(fs.readFileSync(path.join(fixtures, "tsconfig.json.txt"), "utf8")),
  );
  const config = { ...originalConfig, include: ["*/src/**/*.vue", "*/component.ts"] };
  save(path.join(project, "tsconfig.json"), config);
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  const oracle = run(
    out,
    "vue-tsc",
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
  const official = nativeDiagnostics(oracle.stdout, oracle.status, project);
  assert.deepEqual(order(official), order(expected));
  const cliExpected = expected.map((row) => {
    const id = row.file.split("/")[0];
    assert.ok(id);
    const source = sources.get(id);
    assert.ok(source);
    const line = source.split(/\r\n|[\r\n\u2028\u2029]/u)[row.line - 1];
    assert.ok(line !== undefined);
    return { ...row, message: row.message + " (source: " + line.trim() + ")" };
  });
  const tool = path.join(out, "native-tool");
  fs.mkdirSync(tool);
  const wrapper = path.join(tool, "tsgo");
  fs.writeFileSync(wrapper, wrapperSource, { mode: 0o755 });
  const observations: ObjectValue[] = [],
    baselineVts = new Map<string, string>(),
    baselinePackets = new Map<string, unknown>();
  for (const role of ["published", "source"]) {
    if (role === "source") {
      fs.unlinkSync(path.join(project, "node_modules"));
      sourceSdk(project, provider, binary);
    }
    const capture = path.join(out, "native-" + role);
    fs.mkdirSync(capture);
    const env = {
      ...process.env,
      VIZE_DIRECTIVE_NATIVE: checker,
      VIZE_DIRECTIVE_NATIVE_CAPTURE: capture,
    };
    const executable = role === "source" ? binary : cli;
    const checked = run(
      out,
      role + "-cli",
      executable,
      [
        "check",
        "--no-config",
        "--format",
        "json",
        "--show-virtual-ts",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        wrapper,
      ],
      project,
      env,
    );
    const result = packet(checked.stdout, checked.status, project);
    const diagnosticRows = result.diagnostics.map(({ file, line, column, code, message }) => ({
      file,
      line,
      column,
      code,
      message,
    }));
    observations.push({
      role,
      route: "cli",
      diagnostics: diagnosticRows,
      nativeCalls: fs.readdirSync(capture),
      sdkManifest: object(
        JSON.parse(fs.readFileSync(path.join(project, "node_modules/vize/package.json"), "utf8")),
      ),
    });
    for (const [id, source] of sources) {
      const file = result.result.files.find(
        (file) =>
          path.relative(project, path.resolve(project, file.file)).replaceAll("\\", "/") ===
          id + "/src/Tiny.vue",
      );
      assert.ok(file && typeof file.virtualTs === "string" && file.virtualTs.length);
      if (role === "published") baselineVts.set(id, file.virtualTs);
      else
        assert.equal(
          invertUnknownSlotText(file.virtualTs).original,
          baselineVts.get(id),
          "the full CLI VTS changes only the qualified helper token and generated position comments",
        );
      const transaction = captureSlotPacket(
        executable,
        path.join(project, id),
        source,
        path.join(out, "rpc-" + role + "-" + id),
      );
      if (role === "published") baselinePackets.set(id, transaction.result);
      else assertUnknownSlotPacket(baselinePackets.get(id), transaction.result);
    }
    const mapped = run(
      out,
      role + "-mapper",
      wrapper,
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
      env,
    );
    const mappedRows = nativeDiagnostics(mapped.stdout, mapped.status, project);
    observations.push({
      role,
      route: "hosted-mapper",
      diagnostics: mappedRows,
      nativeCalls: fs.readdirSync(capture),
    });
    const actualExpected =
      role === "source"
        ? expected
        : expected.filter(
            (row) =>
              !["unknown", "unknown-union", "unicode-crlf-unknown"].includes(
                row.file.split("/")[0] ?? "",
              ),
          );
    const expectedCliRows =
      role === "source"
        ? cliExpected
        : cliExpected.filter(
            (row) =>
              !["unknown", "unknown-union", "unicode-crlf-unknown"].includes(
                row.file.split("/")[0] ?? "",
              ),
          );
    assert.deepEqual(order(diagnosticRows), order(expectedCliRows));
    assert.deepEqual(order(mappedRows), order(actualExpected));
  }
  assert.deepEqual(identify(binary), sourceImage);
  assert.deepEqual(identify(checker), checkerImage);
  assert.deepEqual(identify(nativeFile), nativeImage);
  const summary = {
    schema: "vize.unknownSlotSourceProof.v1",
    status: "pass",
    inputs,
    sourceImage,
    nativeImage,
    nativeIdentity,
    versions,
    token,
    checkerImage,
    official,
    observations,
    wholeCliVtsInverse: 10,
    wholeMapperPacketsInverse: 10,
    directEofTransactions: 20,
    sourceQualification:
      "actual normal source CLI and Go7.1 hosted mapper; no source NAPI build claimed",
    scope:
      "unknown and unknown-union payload diagnostics; declared-any component remains open under11027; no timing or external7.0 hosted-mapper qualification",
  };
  save(path.join(out, "summary.json"), summary);
}
await main();
