import assert from "node:assert/strict";

/** Accept the existing API-only Vize shim without invoking its lazy accessors. */
export function assertVizeVueCompatShim(plugins, forbiddenHost) {
  assert.ok(!plugins.includes(forbiddenHost), "The authored host compiler must be removed");
  const matches = plugins.filter((plugin) => plugin.name === "vite:vue");
  assert.equal(matches.length, 1, "Exactly one Vize compatibility shim is required");
  const shim = matches[0];
  assert.equal(Object.getPrototypeOf(shim), Object.prototype);
  assert.deepEqual(Reflect.ownKeys(shim), ["name", "api"]);
  for (const name of ["name", "api"]) {
    const descriptor = Object.getOwnPropertyDescriptor(shim, name);
    assert.deepEqual(Reflect.ownKeys(descriptor), [
      "value",
      "writable",
      "enumerable",
      "configurable",
    ]);
    assert.equal(descriptor.writable, true);
    assert.equal(descriptor.enumerable, true);
    assert.equal(descriptor.configurable, true);
  }
  const api = Object.getOwnPropertyDescriptor(shim, "api").value;
  assert.equal(Object.getPrototypeOf(api), Object.prototype);
  assert.deepEqual(Reflect.ownKeys(api), ["options", "include", "exclude", "version"]);
  for (const name of ["options", "include", "exclude"]) {
    const descriptor = Object.getOwnPropertyDescriptor(api, name);
    assert.deepEqual(Reflect.ownKeys(descriptor), ["get", "set", "enumerable", "configurable"]);
    assert.equal(typeof descriptor.get, "function");
    assert.equal(typeof descriptor.set, name === "options" ? "undefined" : "function");
    assert.equal(descriptor.enumerable, true);
    assert.equal(descriptor.configurable, true);
  }
  assert.deepEqual(Object.getOwnPropertyDescriptor(api, "version"), {
    value: "6.0.7",
    writable: true,
    enumerable: true,
    configurable: true,
  });
}
