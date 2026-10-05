import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";

export type Value =
  | { kind: "undefined" | "null" | "hole" }
  | { kind: "boolean" | "string"; value: boolean | string }
  | { kind: "number"; value: number | "NaN" | "Infinity" | "-Infinity" | "-0" }
  | { kind: "bigint"; value: string }
  | { kind: "array"; items: Value[] }
  | { kind: "object"; entries: Array<{ key: string; value: Value }> };
export type Scope = {
  values: Record<string, Value>;
  objects: Record<string, Record<string, Value>>;
  calls: Record<string, { operation: "add" } | { operation: "throw"; message: string }>;
};
export type Trace =
  | { kind: "read"; name: string }
  | { kind: "call"; name: string; args: Value[]; result: Value }
  | { kind: "call"; name: string; args: Value[]; throws: { name: string; message: string } };
export type Result =
  | { kind: "return"; value: Value }
  | { kind: "throw"; error: { name: string; message: string } };
export type Warning = { method: "error" | "warn" | "log"; args: Value[] };
export type Observation = {
  stage: "decoded-callback" | "original-L1-prepared-expression";
  input: string;
  tokens: unknown;
  expression: string | null;
  expressionRecordKeys: string[];
  getterSource: string | null;
  getterName: string | null;
  noop: boolean;
  result: Result | null;
  trace: Trace[];
  warnings: Warning[];
  error: null | { name: string; message: string; ownProperties: unknown[] };
};

const root = new URL("../../_fixtures/reference/vue1/", import.meta.url);
const receiptBytes = readFileSync(new URL("receipt.json", root));
const receipt = JSON.parse(receiptBytes.toString("utf8"));
const gzip = readFileSync(new URL("vue.common.cjs.gz", root));
const source = gunzipSync(gzip);
const license = readFileSync(new URL("LICENSE", root));
export const sha256 = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex");
const gitBlob = (bytes: Uint8Array) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

// The complete original distribution and license are checked before any VM.
assert.equal(receipt.version, "1.0.28");
assert.equal(receipt.repository, "vuejs/vue");
assert.equal(receipt.commit, "a8d6330d7e6b30c252aa753f99c7cb73bfc67a70");
assert.equal(receipt.path, "dist/vue.common.js");
assert.equal(gzip.length, 70081);
assert.equal(source.length, 258747);
assert.equal(sha256(gzip), "a2b8f6e41a4afe0c6a3b58f42bf435f96f0ac8e5618d6da8fa362e017f22fd03");
assert.equal(sha256(source), "a3aa1d2f33558689da6cafdcda85579c0da1cb4e11bd91b414dd1323d16d2cde");
assert.equal(gitBlob(source), "63ebe385546dec4073dacbe13878d3f6a1759053");
assert.equal(gitBlob(license), "f005cb1ff60599f4fb98874e1790696bd7ad3686");
assert.equal(receipt.gzipBytes, gzip.length);
assert.equal(receipt.sourceBytes, source.length);
assert.equal(receipt.gzipSha256, sha256(gzip));
assert.equal(receipt.sourceSha256, sha256(source));
assert.equal(receipt.gitBlob, gitBlob(source));
assert.equal(receipt.licenseBlob, gitBlob(license));

export const referenceIdentity = {
  version: receipt.version,
  commit: receipt.commit,
  gitBlob: gitBlob(source),
  sourceSha256: sha256(source),
  gzipSha256: sha256(gzip),
  licenseGitBlob: gitBlob(license),
  licenseSha256: sha256(license),
  receiptSha256: sha256(receiptBytes),
};

export function value(actual: unknown): Value {
  if (actual === undefined) return { kind: "undefined" };
  if (actual === null) return { kind: "null" };
  if (typeof actual === "boolean") return { kind: "boolean", value: actual };
  if (typeof actual === "string") return { kind: "string", value: actual };
  if (typeof actual === "bigint") return { kind: "bigint", value: actual.toString() };
  if (typeof actual === "number") {
    return {
      kind: "number",
      value: Number.isNaN(actual)
        ? "NaN"
        : actual === Infinity
          ? "Infinity"
          : actual === -Infinity
            ? "-Infinity"
            : Object.is(actual, -0)
              ? "-0"
              : actual,
    };
  }
  if (Array.isArray(actual)) {
    return {
      kind: "array",
      items: Array.from({ length: actual.length }, (_, index) =>
        Object.hasOwn(actual, index) ? value(actual[index]) : { kind: "hole" },
      ),
    };
  }
  assert.equal(typeof actual, "object", "fixture scopes return serializable whole values");
  return {
    kind: "object",
    entries: Object.keys(actual as object).map((key) => ({
      key,
      value: value((actual as Record<string, unknown>)[key]),
    })),
  };
}

function materialize(entry: Value): unknown {
  switch (entry.kind) {
    case "undefined":
      return undefined;
    case "null":
      return null;
    case "boolean":
    case "string":
      return entry.value;
    case "number":
      return entry.value === "-0" ? -0 : Number(entry.value);
    default:
      throw new Error(`Unexpected fixed scope value: ${entry.kind}`);
  }
}

function observedScope(declaration: Scope, trace: Trace[]) {
  const scope = Object.create(null) as Record<string, unknown>;
  const read = (target: object, key: string, name: string, actual: unknown) => {
    Object.defineProperty(target, key, {
      enumerable: true,
      get() {
        trace.push({ kind: "read", name });
        return actual;
      },
    });
  };
  for (const [name, entry] of Object.entries(declaration.values))
    read(scope, name, name, materialize(entry));
  for (const [name, entries] of Object.entries(declaration.objects)) {
    const object = Object.create(null);
    for (const [key, entry] of Object.entries(entries))
      read(object, key, `${name}.${key}`, materialize(entry));
    read(scope, name, name, object);
  }
  for (const [name, implementation] of Object.entries(declaration.calls)) {
    read(scope, name, name, (...args: unknown[]) => {
      if (implementation.operation === "throw") {
        const error = new Error(implementation.message);
        trace.push({
          kind: "call",
          name,
          args: args.map(value),
          throws: { name: error.name, message: error.message },
        });
        throw error;
      }
      const result = args.reduce<number>((sum, entry) => sum + Number(entry), 0);
      trace.push({ kind: "call", name, args: args.map(value), result: value(result) });
      return result;
    });
  }
  return scope;
}

function fullError(error: unknown) {
  assert.ok(error !== null && typeof error === "object");
  const actual = error as { name: string; message: string };
  return {
    name: actual.name,
    message: actual.message,
    ownProperties: Reflect.ownKeys(error).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(error, key)!;
      return {
        key: String(key),
        enumerable: descriptor.enumerable,
        configurable: descriptor.configurable,
        kind: "value" in descriptor ? "data" : "accessor",
        writable: "value" in descriptor ? descriptor.writable : null,
        getterSource: descriptor.get?.toString() ?? null,
        setterSource: descriptor.set?.toString() ?? null,
        value: value("value" in descriptor ? descriptor.value : Reflect.get(error, key)),
      };
    }),
  };
}

export function observe(
  input: string,
  environment: "test" | "production",
  declaration: Scope,
  stage: Observation["stage"] = "decoded-callback",
): Observation {
  const warnings: Warning[] = [];
  const trace: Trace[] = [];
  const module = { exports: {} };
  const console = Object.fromEntries(
    (["error", "warn", "log"] as const).map((method) => [
      method,
      (...args: unknown[]) => warnings.push({ method, args: args.map(value) }),
    ]),
  );
  // A fresh VM for every original/current input and each environment prevents
  // expression/text cache hits from suppressing genuine development warnings.
  vm.runInNewContext(
    source.toString("utf8"),
    {
      module,
      exports: module.exports,
      process: { env: { NODE_ENV: environment } },
      console,
      setTimeout: () => 0,
    },
    { timeout: 1000, filename: "actual-pinned-vue-1.0.28.cjs" },
  );
  const Vue = module.exports as {
    version: string;
    parsers: {
      text: { parseText(input: string): unknown; tokensToExp(tokens: unknown): string };
      expression: {
        parseExpression(
          input: string,
          needSet: boolean,
        ): {
          exp: string;
          get: (scope: object) => unknown;
        };
      };
    };
  };
  assert.equal(Vue.version, "1.0.28");
  const observation: Observation = {
    stage,
    input,
    tokens: null,
    expression: null,
    expressionRecordKeys: [],
    getterSource: null,
    getterName: null,
    noop: false,
    result: null,
    trace,
    warnings,
    error: null,
  };
  try {
    if (stage === "decoded-callback") {
      const tokens = Vue.parsers.text.parseText(input);
      observation.tokens = tokens === null ? null : JSON.parse(JSON.stringify(tokens));
      if (tokens === null) return observation;
      observation.expression = Vue.parsers.text.tokensToExp(tokens);
    } else {
      // This is the actual fresh Rust callback's prepared expression, not a
      // synthesized framing input and not a claim about browser HTML decoding.
      observation.expression = input;
    }
    const parsed = Vue.parsers.expression.parseExpression(observation.expression, false);
    observation.expressionRecordKeys = Object.keys(parsed);
    observation.getterSource = parsed.get.toString();
    observation.getterName = parsed.get.name;
    observation.noop =
      parsed.get.name === "noop" && observation.getterSource === "function noop() {}";
    observation.result = {
      kind: "return",
      value: value(parsed.get(observedScope(declaration, trace))),
    };
  } catch (error) {
    observation.error = fullError(error);
    observation.result = {
      kind: "throw",
      error: {
        name: observation.error.name,
        message: observation.error.message,
      },
    };
  }
  return observation;
}
