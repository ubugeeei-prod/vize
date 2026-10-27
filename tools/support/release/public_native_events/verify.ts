import assert from "node:assert/strict";
import { verifyPublicProvider } from "../public-provider.ts";
import { compileFunction } from "node:vm";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const [consumerArgument, expectedVersion, phase, receiptArgument, identityArgument] =
  process.argv.slice(2);
assert(consumerArgument && expectedVersion && receiptArgument && identityArgument);
assert(/^[0-9]+\.[0-9]+\.[0-9]+$/.test(expectedVersion));
assert(phase === "before" || phase === "after");
assert.equal(process.env.VIZE_NATIVE_BINDING_PATH, undefined, "public provider only");
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH, undefined, "public provider only");
for (const key of [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
]) {
  assert(!process.env[key], `public provider only: ${key}`);
}
const releaseIdentity: unknown = JSON.parse(readFileSync(identityArgument, "utf8"));
assert(releaseIdentity !== null && typeof releaseIdentity === "object");
assert("version" in releaseIdentity && releaseIdentity.version === expectedVersion);
assert("tag" in releaseIdentity && releaseIdentity.tag === `v${expectedVersion}`);
assert(
  "sourceSha" in releaseIdentity &&
    typeof releaseIdentity.sourceSha === "string" &&
    /^[0-9a-f]{40}$/.test(releaseIdentity.sourceSha),
);
assert(
  "requiredSource" in releaseIdentity &&
    releaseIdentity.requiredSource === "3fa5cf66c8f5d12e7d2f286df5050933cb0daf47",
);
assert(
  "containsRequiredSource" in releaseIdentity &&
    releaseIdentity.containsRequiredSource === (phase === "after"),
);
if (phase === "before")
  assert.equal(expectedVersion, "0.599.0", "the historical control version is frozen");
const directory = path.dirname(fileURLToPath(import.meta.url));
const consumer = realpathSync(consumerArgument);
const receiptPath = path.resolve(receiptArgument);
const referencePath = path.join(
  directory,
  "official-vue-3.5.44-prefix-identifiers-reference.json.txt",
);
const require = createRequire(path.join(consumer, "package.json"));
const native = require("@vizejs/native");
const compiler = require("@vue/compiler-dom");
const vue = require("vue");
assert.equal(require("@vizejs/native/package.json").version, expectedVersion);
assert.equal(require("@vue/compiler-dom/package.json").version, "3.5.44");
assert.equal(vue.version, "3.5.44");
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const options = {
  mode: "function",
  prefixIdentifiers: true,
  hoistStatic: false,
  cacheHandlers: false,
};
const cases = [
  { id: "original-native-if", source: '<div v-if="ok" @fooBar="pingA">x</div>' },
  {
    id: "original-native-for-keyed",
    source: '<div><div v-for="i in 3" :key="i" @fooBar="pingA">a</div></div>',
  },
  { id: "original-component-if", source: '<Comp v-if="ok" @fooBar="pingA" />' },
  { id: "native-plain-control", source: '<div @fooBar="pingA">x</div>' },
  { id: "native-if-lowercase-control", source: '<div v-if="ok" @foobar="pingA">x</div>' },
  {
    id: "native-for-lowercase-control",
    source: '<div><div v-for="i in 3" :key="i" @foobar="pingA">a</div></div>',
  },
  {
    id: "native-if-explicit-key",
    source: '<div v-if="ok" key="native" @fooBar="pingA">x</div>',
  },
  {
    id: "component-for-control",
    source: '<div><Comp v-for="i in 3" :key="i" @fooBar="pingA" /></div>',
  },
  {
    id: "native-if-else",
    source: '<div v-if="ok" @fooBar="pingA">a</div><div v-else @fooBar="pingB">b</div>',
  },
  {
    id: "native-for-unkeyed",
    source: '<div><div v-for="i in 3" @fooBar="pingA">a</div></div>',
  },
] as const;
const states = [
  { id: "branch-on", ok: true },
  { id: "branch-off", ok: false },
] as const;
const component = { name: "Comp", render() {} };
const runtime = {
  ...vue,
  resolveComponent: (name: string) => {
    assert.equal(name, "Comp");
    return component;
  },
};
type NodeShape = {
  type: unknown;
  key: unknown;
  ref: unknown;
  props: Record<string, unknown> | null;
  children: unknown;
  patchFlag: number;
  dynamicProps: unknown;
  shapeFlag: number;
};
function context(ok: boolean) {
  return { ok, pingA: function pingA(this: void) {}, pingB: function pingB(this: void) {} };
}
function view(value: unknown, ctx: ReturnType<typeof context>): unknown {
  if (typeof value === "function") {
    if (value === ctx.pingA) return { handler: "pingA" };
    if (value === ctx.pingB) return { handler: "pingB" };
    throw new Error("unrecognized function in the complete VNode packet");
  }
  if (Array.isArray(value)) return value.map((entry) => view(entry, ctx));
  if (vue.isVNode(value)) {
    const node = value as NodeShape;
    const type =
      node.type === vue.Fragment
        ? "Fragment"
        : node.type === vue.Comment
          ? "Comment"
          : node.type === vue.Text
            ? "Text"
            : node.type === component
              ? "component:Comp"
              : node.type;
    assert.equal(typeof type, "string");
    return {
      type,
      key: node.key,
      ref: view(node.ref, ctx),
      props: view(node.props, ctx),
      children: view(node.children, ctx),
      patchFlag: node.patchFlag,
      dynamicProps: node.dynamicProps,
      shapeFlag: node.shapeFlag,
    };
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
        .map(([key, entry]) => [key, view(entry, ctx)]),
    );
  }
  return value;
}
function execute(code: string, preamble: string | null) {
  return states.map((state) => {
    const ctx = context(state.ok);
    const render =
      preamble === null
        ? compileFunction(code, ["Vue"])(runtime)
        : compileFunction(`${preamble}\n${code}\nreturn render;`, ["Vue", "ok", "pingA", "pingB"])(
            runtime,
            ctx.ok,
            ctx.pingA,
            ctx.pingB,
          );
    return { state, vnode: view(render(ctx, [], {}, {}, {}, {}), ctx) };
  });
}
const inputSha256 = sha256(JSON.stringify({ cases, states, options }));
const generatedReference = {
  schema: "vize-public-uppercase-event-oracle-v1",
  compiler: "@vue/compiler-dom@3.5.44",
  runtime: "vue@3.5.44",
  inputSha256,
  cases: cases.map((row) => {
    const warnings: unknown[] = [];
    const result = compiler.compile(row.source, {
      ...options,
      onWarn: (warning: unknown) => warnings.push(warning),
    });
    return { ...row, code: result.code, warnings, observations: execute(result.code, null) };
  }),
};
const referenceBytes = readFileSync(referencePath);
assert.equal(inputSha256, "d614ba281249549cbc92c061defd21a9aeee0b724aa644b16e7df35b7a205cce");
assert.equal(
  sha256(referenceBytes),
  "c8752cefa3ebd5ccfdbe9477804fe8fc4bc03060507efdef4bb7961f8df97437",
);
const reference = JSON.parse(referenceBytes.toString());
assert.deepEqual(
  generatedReference,
  reference,
  "the pinned independent compiler and all oracle packets remain identical",
);
const results = cases.map((row, index) => {
  const result = native.compile(row.source, options);
  const observations = execute(result.code, result.preamble);
  let difference: string | null = null;
  try {
    assert.deepEqual(observations, reference.cases[index].observations);
  } catch (error) {
    difference = String(error);
  }
  return {
    ...row,
    completeCompileResult: result,
    observations,
    matchesOfficialOracle: difference === null,
    difference,
  };
});
const nativeImages = Object.keys(require.cache)
  .filter((name) => name.endsWith(".node"))
  .map((name) => {
    assert(
      realpathSync(name).startsWith(consumer + path.sep),
      "native binary comes from this clean installed consumer",
    );
    const packagePath = path.join(path.dirname(name), "package.json");
    const packageMetadata = JSON.parse(readFileSync(packagePath, "utf8"));
    assert.equal(packageMetadata.version, expectedVersion);
    assert.match(packageMetadata.name, /^@vizejs\/native-/);
    const lock = JSON.parse(readFileSync(path.join(consumer, "package-lock.json"), "utf8"));
    const packageKey = `node_modules/${packageMetadata.name}`;
    // The original macOS before installation used npm --prefix and retained
    // duplicate absolute-prefix lock keys. Keep those original bytes intact.
    const lockKeys = Object.keys(lock.packages).filter(
      (key) => key === packageKey || (phase === "before" && key.endsWith(`/${packageKey}`)),
    );
    assert(lockKeys.length > 0, "the loaded public target is present in the lock");
    const locked = lock.packages[lockKeys[0]];
    for (const key of lockKeys) {
      for (const field of ["version", "integrity", "resolved"]) {
        assert.equal(lock.packages[key][field], locked[field], "all historical lock aliases agree");
      }
    }
    assert.equal(locked.version, expectedVersion);
    assert.match(locked.integrity, /^sha512-/);
    assert.match(locked.resolved, /^https:\/\/registry\.npmjs\.org\//);
    const bytes = readFileSync(name);
    return {
      path: name,
      bytes: bytes.length,
      sha256: sha256(bytes),
      package: packageMetadata.name,
      version: packageMetadata.version,
      lockKeys,
      archiveIntegrity: locked.integrity,
      archiveUrl: locked.resolved,
    };
  });
assert.equal(nativeImages.length, 1);
const image = nativeImages[0];
assert.ok(image);
const publicProvider =
  phase === "after"
    ? await verifyPublicProvider(expectedVersion, releaseIdentity.sourceSha, image)
    : null;
const failed = results.filter((row) => !row.matchesOfficialOracle).map((row) => row.id);
const receipt = {
  schema: "vize-public-uppercase-event-acceptance-v1",
  phase,
  expectedVersion,
  inputSha256,
  referenceSha256: sha256(referenceBytes),
  scriptSha256: sha256(readFileSync(fileURLToPath(import.meta.url))),
  releaseIdentity,
  consumer,
  packageLockSha256: sha256(readFileSync(path.join(consumer, "package-lock.json"))),
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  nativeImages,
  publicProvider,
  nativeBuildToken: native.getNativeBuildToken(),
  results,
  failed,
  accepted: phase === "after" && failed.length === 0,
};
writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n", { flag: "wx" });
if (phase === "before") {
  const historicalBytes = readFileSync(path.join(directory, "historical-public-0.599.0.json.txt"));
  assert.equal(
    sha256(historicalBytes),
    "aa05a1b783459b300d83e01e493b3f1fe5e6020832171e856ddf2fd967f1ce47",
  );
  const historical = JSON.parse(historicalBytes.toString("utf8"));
  assert.equal(historical.inputSha256, inputSha256);
  assert.equal(historical.referenceSha256, sha256(referenceBytes));
  assert.deepEqual(
    results.map(({ id, observations }) => ({ id, observations })),
    historical.results.map((row: { id: string; observations: unknown }) => ({
      id: row.id,
      observations: row.observations,
    })),
    "all twenty historical case/state observations remain byte-for-byte equivalent",
  );
  assert.deepEqual(
    failed,
    [
      "original-native-if",
      "original-native-for-keyed",
      "native-if-explicit-key",
      "native-if-else",
      "native-for-unkeyed",
    ],
    "exact historical failures and all neighboring controls remain frozen",
  );
} else
  assert.deepEqual(failed, [], "the public release must satisfy every pinned independent packet");
console.log(
  JSON.stringify({
    phase,
    version: expectedVersion,
    inputSha256,
    referenceSha256: receipt.referenceSha256,
    cases: results.length,
    observations: results.length * states.length,
    failed,
    accepted: receipt.accepted,
    receiptPath,
  }),
);
