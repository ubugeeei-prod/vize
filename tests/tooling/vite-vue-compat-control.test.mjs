import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { assertVizeVueCompatShim } from "./support/vite-vue-compat-control.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(
  root,
  "tests/_fixtures/differential/config/vue-config-8371/packed-vue-compat",
);
const sha = (body) => createHash("sha256").update(body).digest("hex");
function shim(
  onAccess = () => {
    throw new Error("The shape control must not invoke accessors");
  },
) {
  return {
    name: "vite:vue",
    api: {
      get options() {
        return onAccess();
      },
      get include() {
        return onAccess();
      },
      set include(value) {
        onAccess(value);
      },
      get exclude() {
        return onAccess();
      },
      set exclude(value) {
        onAccess(value);
      },
      version: "6.0.7",
    },
  };
}

void test("the real API-only shim geometry never invokes its compiler or filter accessors", () => {
  let accessed = 0;
  const actual = shim(() => {
    accessed++;
    throw new Error("unexpected invocation");
  });
  const host = {
    name: "vite:vue",
    transform() {
      throw new Error("host compiler ran");
    },
  };
  assertVizeVueCompatShim([{ name: "vite-plugin-vize" }, actual], host);
  assert.equal(accessed, 0);
  assert.throws(() => assertVizeVueCompatShim([actual, host], host));
});

void test("missing, duplicated and genuine compiler hooks cannot masquerade as the shim", () => {
  assert.throws(() => assertVizeVueCompatShim([], {}));
  assert.throws(() => assertVizeVueCompatShim([shim(), shim()], {}));
  for (const key of ["transform", "load", "config", "configResolved"]) {
    const own = shim();
    own[key] = () => {};
    assert.throws(() => assertVizeVueCompatShim([own], {}), key);
    const inherited = shim();
    Object.setPrototypeOf(inherited, { [key]() {} });
    assert.throws(() => assertVizeVueCompatShim([inherited], {}), `inherited ${key}`);
  }
  for (const owner of ["plugin", "api"]) {
    const extra = shim();
    (owner === "plugin" ? extra : extra.api)[Symbol("unknown")] = true;
    assert.throws(() => assertVizeVueCompatShim([extra], {}), owner);
  }
});

void test("every API field, descriptor and exact compatibility version is required", () => {
  for (const key of ["options", "include", "exclude", "version"]) {
    const missing = shim();
    delete missing.api[key];
    assert.throws(() => assertVizeVueCompatShim([missing], {}), key);
  }
  for (const key of ["options", "include", "exclude"]) {
    const data = shim();
    Object.defineProperty(data.api, key, { value: null, configurable: true });
    assert.throws(() => assertVizeVueCompatShim([data], {}), `data ${key}`);
    const getter = shim();
    Object.defineProperty(getter.api, key, { get: undefined });
    assert.throws(() => assertVizeVueCompatShim([getter], {}), `getter ${key}`);
    for (const flag of ["enumerable", "configurable"]) {
      const changed = shim();
      Object.defineProperty(changed.api, key, { [flag]: false });
      assert.throws(() => assertVizeVueCompatShim([changed], {}), `${flag} ${key}`);
    }
  }
  const optionsSetter = shim();
  Object.defineProperty(optionsSetter.api, "options", { set() {} });
  assert.throws(() => assertVizeVueCompatShim([optionsSetter], {}));
  for (const key of ["include", "exclude"]) {
    const missingSetter = shim();
    Object.defineProperty(missingSetter.api, key, { set: undefined });
    assert.throws(() => assertVizeVueCompatShim([missingSetter], {}), key);
  }
  const wrongVersion = shim();
  wrongVersion.api.version = "6.0.6";
  assert.throws(() => assertVizeVueCompatShim([wrongVersion], {}));
  const inheritedApi = shim();
  Object.setPrototypeOf(inheritedApi.api, { transform() {} });
  assert.throws(() => assertVizeVueCompatShim([inheritedApi], {}));
  for (const key of ["name", "api"]) {
    const accessor = shim();
    const value = accessor[key];
    Object.defineProperty(accessor, key, {
      get() {
        return value;
      },
    });
    assert.throws(() => assertVizeVueCompatShim([accessor], {}), key);
  }
});

void test("the finite test-only successor retains whole producers and every original build control", () => {
  const packetBody = fs.readFileSync(path.join(fixture, "source.json"));
  assert.equal(sha(packetBody), "fc588f115cdba252e9b045a3aa1f1f357feaf21364a25a6a104179ce0888ad15");
  const packet = JSON.parse(packetBody);
  assert.equal(packet.source, "01aef925a6e9ca8ff3624843a12d34428b87377b");
  assert.deepEqual(packet.credit, { compiled: false, native: false, performance: false });
  assert.equal(packet.sources.length, 4);
  const original = packet.sources.map((entry) => {
    const target = path.join(fixture, entry.archive);
    const status = fs.lstatSync(target);
    assert.ok(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111));
    const body = fs.readFileSync(target);
    assert.equal(body.length, entry.bytes);
    assert.equal(sha(body), entry.sha256);
    if (entry.path.startsWith("npm/"))
      assert.deepEqual(fs.readFileSync(path.join(root, entry.path)), body);
    return body;
  });
  assert.equal(packet.changes.length, 5);
  let expected = original[0].toString();
  for (const [before, after] of packet.changes) {
    assert.equal(expected.split(before).length, 2);
    expected = expected.replace(before, after);
  }
  assert.deepEqual(fs.readFileSync(path.join(root, packet.sources[0].path)), Buffer.from(expected));
  assert.equal(packet.failure.job, 114787033304);
  assert.equal(
    packet.failure.rawSHA256,
    "19811ccc66b1b014e477995cf12f9d12522bf4d0fad1ee6406fa1df31e8d69f8",
  );
  assert.equal(
    packet.failure.zipSHA256,
    "9af05e6231b349a4d5c823528f90bacb0bf199bb13e274d29659851493860d0e",
  );
});
