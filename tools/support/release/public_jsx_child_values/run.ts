/** Run unchanged original observers against the separately authenticated public native image. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { request, type Custody } from "./contract.ts";

type Image = { bytes: number; sha256: string };
type Authority = {
  version: string;
  candidate: string;
  producerRun: string;
  nativeIdentity: { kind: "native"; token: string };
  entrypoint: Image;
  packageJson: Image;
  binary: Image & { package: string };
};
type CompileResult = {
  code: string;
  map?: string;
  errors: string[];
  warnings: string[];
  scopedStyles: unknown[];
};
type Native = {
  compileJsx(source: string, options: object): CompileResult;
  getNativeBuildToken(): string;
};
const [rootArg] = process.argv.slice(2);
assert(rootArg, "the completed independent bootstrap root is required");
const root = fs.realpathSync(rootArg);
assert(root.startsWith(fs.realpathSync(process.env.RUNNER_TEMP!) + path.sep));
const consumer = fs.realpathSync(path.join(root, "consumer"));
const oracle = fs.realpathSync(path.join(root, "oracle"));
const authorityArg = path.join(root, "proof/authority.json");
const checkout = fs.realpathSync(process.cwd());
const out = path.join(root, "runtime");
assert(!fs.existsSync(out), "output must be fresh; never inherit a previous successful capture");
fs.mkdirSync(out, { recursive: true });
const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const image = (file: string): Image => {
  const bytes = fs.readFileSync(file);
  return { bytes: bytes.length, sha256: sha256(bytes) };
};
const read = (file: string): unknown => JSON.parse(fs.readFileSync(file, "utf8"));
const write = (name: string, value: unknown): void =>
  fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n");
const authorityBytes = fs.readFileSync(authorityArg);
const authority = JSON.parse(authorityBytes.toString()) as Authority;
request(authority.version, authority.candidate, authority.producerRun, process.env);
const bootstrap = read(path.join(root, "proof/bootstrap-complete.json")) as {
  version: string;
  candidate: string;
  producerRun: string;
  authority: Image;
  checkout: string;
  custody: Custody;
};
assert.deepEqual(image(authorityArg), bootstrap.authority);
assert.equal(bootstrap.version, authority.version);
assert.equal(bootstrap.candidate, authority.candidate);
assert.equal(bootstrap.producerRun, authority.producerRun);
assert.equal(bootstrap.checkout, checkout);
const custody = read(fileURLToPath(new URL("./source-custody.json", import.meta.url))) as Custody;
assert.deepEqual(bootstrap.custody, custody);
assert.match(authority.version, /^0\.\d+\.0$/);
assert.match(authority.candidate, /^[0-9a-f]{40}$/);
assert.deepEqual(Object.keys(authority.nativeIdentity).sort(), ["kind", "token"]);
assert.equal(authority.nativeIdentity.kind, "native");
assert.match(authority.nativeIdentity.token, /^[0-9a-f]{32}$/);
assert.match(authority.binary.package, /^@vizejs\/native-(?:darwin|linux|win32)-/);
for (const entry of [authority.entrypoint, authority.packageJson, authority.binary]) {
  assert(Number.isSafeInteger(entry.bytes) && entry.bytes > 0);
  assert.match(entry.sha256, /^[0-9a-f]{64}$/);
}
for (const key of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "NODE_OPTIONS",
  "NODE_PATH",
])
  assert(!process.env[key], `${key} must not override the independent public provider`);
const merge = custody.merge;
const ancestry = spawnSync(
  "git",
  ["--no-replace-objects", "merge-base", "--is-ancestor", merge, authority.candidate],
  { cwd: checkout, encoding: "utf8", timeout: 30000 },
);
write("candidate-ancestry.json", {
  merge,
  candidate: authority.candidate,
  status: ancestry.status,
  signal: ancestry.signal,
  error: ancestry.error?.message,
  stdout: ancestry.stdout,
  stderr: ancestry.stderr,
});
assert.equal(ancestry.status, 0, "release candidate must actually contain the merged repair");

// Exact existing observers/inputs from M. Nothing below rewrites their source or expectations.
const pins = custody.pins;
const checkPins = (): void => {
  for (const base of [checkout, oracle])
    for (const [name, expected] of Object.entries(pins))
      assert.equal(image(path.join(base, name)).sha256, expected, name);
};
checkPins();
write("source-custody.json", { merge, pins });
const require = createRequire(path.join(consumer, "package.json"));
const withinConsumer = (file: string): string => {
  const resolved = fs.realpathSync(file);
  assert(resolved.startsWith(path.join(consumer, "node_modules") + path.sep), resolved);
  return resolved;
};
const entrypoint = withinConsumer(require.resolve("@vizejs/native"));
const packageJson = path.join(path.dirname(entrypoint), "package.json");
assert.deepEqual(image(entrypoint), authority.entrypoint);
assert.deepEqual(image(packageJson), authority.packageJson);
const metadata = read(packageJson) as { name: string; version: string };
assert.equal(metadata.name, "@vizejs/native");
assert.equal(metadata.version, authority.version);
const native = require("@vizejs/native") as Native;
const identity = (
  require("@vizejs/native/native-binding-identity.js") as {
    getBindingIdentity(binding: Native): unknown;
  }
).getBindingIdentity(native);
assert.deepEqual(identity, authority.nativeIdentity);
assert.equal(native.getNativeBuildToken(), authority.nativeIdentity.token);
const binary = withinConsumer(require.resolve(authority.binary.package));
assert(binary.endsWith(".node"), "the authority must name the actual native image");
assert.deepEqual(image(binary), { bytes: authority.binary.bytes, sha256: authority.binary.sha256 });
const binaryMetadata = read(path.join(path.dirname(binary), "package.json")) as {
  name: string;
  version: string;
};
assert.equal(binaryMetadata.name, authority.binary.package);
assert.equal(binaryMetadata.version, authority.version);
const loaded = Object.keys(require.cache).filter(
  (file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine."),
);
assert.equal(loaded.length, 1);
assert.equal(fs.realpathSync(loaded[0]!), binary);
write("public-native-identity.json", {
  authority,
  authoritySha256: sha256(authorityBytes),
  consumer,
  entrypoint,
  packageJson,
  binary,
  metadata,
  binaryMetadata,
  identity,
});

const original = read(
  path.join(checkout, "tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json"),
) as { source: string };
const shadowedSource =
  "const Node=1;const document=1;const Array=1;const String=1;" + original.source;
const compile = (source: string, jsxMode: "vapor" | "vdom"): CompileResult =>
  native.compileJsx(source, {
    filename: "App.jsx",
    lang: "jsx",
    jsxMode,
    jsxCompat: "native",
    sourceMap: false,
    ssr: false,
  });
const results = {
  vapor: compile(original.source, "vapor"),
  vdom: compile(original.source, "vdom"),
  shadowedVapor: compile(shadowedSource, "vapor"),
};
write("complete-compile-results.json", {
  original,
  shadowedSource,
  results,
  ownKeys: Object.fromEntries(
    Object.entries(results).map(([name, result]) => [name, Object.getOwnPropertyNames(result)]),
  ),
  undefinedKeys: Object.fromEntries(
    Object.entries(results).map(([name, result]) => [
      name,
      Object.keys(result).filter((key) => result[key as keyof CompileResult] === undefined),
    ]),
  ),
});
for (const result of Object.values(results)) {
  assert.equal(typeof result.code, "string");
  assert(result.code.length > 0);
  assert.deepEqual(result.errors, []);
  assert.deepEqual(result.warnings, []);
  // NAPI's actual declaration is map?: string; unlike the Rust DTO, None is undefined.
  assert.equal(result.map, undefined);
  assert.deepEqual(result.scopedStyles, []);
}
const module = (result: CompileResult) => ({ code: result.code, functionName: "App" });
const packet = {
  original,
  vapor: module(results.vapor),
  vdom: module(results.vdom),
  shadowedSource,
  shadowedVapor: module(results.shadowedVapor),
};
write("child-modules.json", packet);
const run = (name: string, script: string, args: string[], input?: string): number | null => {
  const result = spawnSync(process.execPath, [path.join(oracle, script), ...args], {
    cwd: oracle,
    input,
    encoding: "utf8",
    timeout: 180000,
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, VIZE_JSX_CHILD_TRACE_CAPTURE: path.join(out, "child-runtime.json") },
  });
  fs.writeFileSync(path.join(out, `${name}.stdout`), result.stdout ?? "");
  fs.writeFileSync(path.join(out, `${name}.stderr`), result.stderr ?? "");
  write(`${name}.process.json`, {
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    script,
    args,
  });
  return result.status;
};
const nodeStatus = run(
  "node",
  "tests/tooling/jsx-vapor-oracle/child-values.ts",
  [],
  JSON.stringify(packet),
);
const browserStatus = run("browser", "tests/tooling/jsx-vapor-oracle/child-browser.ts", [
  path.join(out, "child-modules.json"),
  path.join(out, "child-browser.json"),
]);
checkPins();
assert.deepEqual(image(entrypoint), authority.entrypoint);
assert.deepEqual(image(packageJson), authority.packageJson);
assert.deepEqual(image(binary), { bytes: authority.binary.bytes, sha256: authority.binary.sha256 });
assert.equal(native.getNativeBuildToken(), authority.nativeIdentity.token);
assert.equal(nodeStatus, 0, "retain the complete failing raw Node routes if qualification fails");
assert.equal(browserStatus, 0, "retain the complete failing genuine Chromium receipt");
const runtime = read(path.join(out, "child-runtime.json")) as {
  routes: Record<string, unknown>;
  receipt: {
    qualifiedVaporPhases: number;
    referenceConflictPhases: number;
    wholeCorpusQualified: boolean;
    issue6890Complete: boolean;
  };
};
assert.deepEqual(
  Object.keys(runtime.routes).sort(),
  [
    "public-vapor",
    "public-babel",
    "source-vapor",
    "source-vdom",
    "node-control-public",
    "node-control-source",
    "shadowed-public",
    "shadowed-source",
  ].sort(),
);
assert.equal(runtime.receipt.qualifiedVaporPhases, 9);
assert.equal(runtime.receipt.referenceConflictPhases, 1);
assert.equal(runtime.receipt.wholeCorpusQualified, false);
assert.equal(runtime.receipt.issue6890Complete, false);
const browser = read(path.join(out, "child-browser.json")) as {
  routes: Record<string, unknown>;
  qualifiedVaporPhases: number;
  referenceConflictPhases: number;
  wholeCorpusQualified: boolean;
  issue6890Complete: boolean;
};
assert.deepEqual(
  Object.keys(browser.routes).sort(),
  ["public", "public-shadow", "source", "source-shadow", "before", "before-shadow"].sort(),
);
assert.equal(browser.qualifiedVaporPhases, 9);
assert.equal(browser.referenceConflictPhases, 1);
assert.equal(browser.wholeCorpusQualified, false);
assert.equal(browser.issue6890Complete, false);
write("public-child-acceptance.json", {
  status: "pass",
  version: authority.version,
  candidate: authority.candidate,
  merge,
  authoritySha256: sha256(authorityBytes),
  nativeIdentity: identity,
  completeNativeRoutes: 8,
  browserRoutes: 6,
  qualifiedVaporPhases: 9,
  referenceConflictPhases: 1,
  nativeVdomDebtRetained: true,
  wholeCorpusQualified: false,
  issue6890Complete: false,
  scope:
    "authenticated installed-public native compiler; unchanged Vue3.6.0-rc.10 Vapor reference environment; nine cells only",
  files: Object.fromEntries(fs.readdirSync(out).map((name) => [name, image(path.join(out, name))])),
});
