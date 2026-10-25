import assert from "node:assert/strict";

function plain(value) {
  if (value === undefined) return { kind: "undefined" };
  if (value === null || typeof value !== "object") {
    assert.notEqual(typeof value, "function");
    assert.notEqual(typeof value, "symbol");
    return value;
  }
  if (Array.isArray(value)) return value.map(plain);
  return Object.fromEntries(
    Reflect.ownKeys(value).map((key) => {
      assert.equal(typeof key, "string");
      return [key, plain(value[key])];
    }),
  );
}

// Observe every own VNode field and descriptor in this native-element family.
// Only the context field has process-local identity: it must be the very same
// owning instance, rather than an arbitrary object or a dropped field.
export function vnode(node, owner) {
  return Object.fromEntries(
    Reflect.ownKeys(node).map((key) => {
      assert.equal(typeof key, "string");
      const descriptor = Object.getOwnPropertyDescriptor(node, key);
      assert.ok(descriptor);
      assert.equal(typeof descriptor.get, "undefined");
      assert.equal(typeof descriptor.set, "undefined");
      let value;
      if (key === "context" && node[key] !== undefined) {
        assert.equal(node[key], owner);
        value = { kind: "actual-owning-Vue-instance" };
      } else if (key === "children" && node[key] !== undefined) {
        value = node[key].map((child) => vnode(child, owner));
      } else {
        value = plain(node[key]);
      }
      return [
        key,
        {
          value,
          enumerable: descriptor.enumerable,
          configurable: descriptor.configurable,
          writable: descriptor.writable,
        },
      ];
    }),
  );
}
