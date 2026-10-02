import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { compileFunction } from "node:vm";

const fixtures = JSON.parse(readFileSync(0, "utf8"));
const values = [Object.freeze({ marker: "prototype-value" }), null, 42];
for (const fixture of fixtures) {
  for (const value of values) {
    const other = Symbol("ordinary-property");
    const context = { ["__proto__"]: value, other };
    const original = compileFunction(`return ${fixture.original};`, ["__proto__", "other"])(
      value,
      other,
    );
    const emitted = compileFunction(`return ${fixture.emitted};`, ["_ctx"])(context);
    assert.equal(Object.hasOwn(original, "__proto__"), true, fixture.original);
    assert.equal(Object.hasOwn(emitted, "__proto__"), true, fixture.emitted);
    assert.equal(Object.getPrototypeOf(original), Object.prototype);
    assert.equal(Object.getPrototypeOf(emitted), Object.getPrototypeOf(original), fixture.emitted);
    assert.deepEqual(Reflect.ownKeys(emitted), Reflect.ownKeys(original));
    for (const key of Reflect.ownKeys(original)) {
      assert.deepEqual(
        Object.getOwnPropertyDescriptor(emitted, key),
        Object.getOwnPropertyDescriptor(original, key),
        fixture.emitted,
      );
    }
  }
}
process.stdout.write(
  JSON.stringify({ fixtures: fixtures.length, executions: fixtures.length * values.length }),
);
