// The current Rust operator embeds this typed parser; immutable legacy H does
// not need to contain this new tool or understand the overlap CLI protocol.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { isMap, isScalar, parseDocument } from "yaml";

const document = parseDocument(readFileSync(0, "utf8"), { uniqueKeys: true });
assert.deepEqual(document.errors, [], "ambiguous release workflow YAML");
assert.deepEqual(document.warnings, [], "unsupported release workflow YAML");
const inputs = document.getIn(["on", "workflow_dispatch", "inputs"], true);
assert.ok(isMap(inputs), "declared release dispatch input map required");
const schemas = {
  tag_name: { required: true, type: "string" },
  release_pr: { required: false, default: "", type: "string" },
  expected_sha: { required: false, default: "", type: "string" },
  pinned: { required: false, default: false, type: "boolean" },
};
const keys = inputs.items.map((pair) => {
  assert.ok(isScalar(pair.key) && typeof pair.key.value === "string", "literal input key required");
  return pair.key.value;
});
assert.equal(new Set(keys).size, keys.length, "duplicate release dispatch input");
for (const key of keys)
  assert.ok(
    Object.hasOwn(schemas, key) || key === "qualification_deadline" || key === "crate_set",
    `unknown release dispatch input: ${key}`,
  );
for (const [key, schema] of Object.entries(schemas)) {
  const input: unknown = inputs.get(key, true);
  assert.ok(isMap(input), `declared ${key} input required`);
  const value: Record<string, unknown> = input.toJSON();
  assert.equal(typeof value.description, "string", `${key} description required`);
  delete value.description;
  assert.deepEqual(value, schema, `unsupported ${key} declaration`);
}
const crate = inputs.get("crate_set", true);
if (crate !== undefined) {
  assert.ok(isMap(crate), "literal crate_set input required");
  const value = crate.toJSON();
  assert.equal(typeof value.description, "string");
  assert.equal(value.type, "choice");
  assert.equal(value.required, true);
  assert.ok(Array.isArray(value.options) && value.options.length > 0);
  for (const entry of value.options) {
    assert.equal(typeof entry, "string");
    assert.ok(entry.length > 0);
  }
  assert.ok(value.options.includes(value.default));
  assert.deepEqual(Object.keys(value).sort(), [
    "default",
    "description",
    "options",
    "required",
    "type",
  ]);
}
const deadline = inputs.get("qualification_deadline", true);
if (deadline === undefined) process.stdout.write("legacy\n");
else {
  assert.ok(isMap(deadline), "literal qualification_deadline declaration required");
  const value = deadline.toJSON();
  assert.equal(typeof value.description, "string");
  delete value.description;
  assert.deepEqual(
    value,
    { required: false, default: "", type: "string" },
    "unsupported qualification_deadline declaration",
  );
  process.stdout.write("overlap\n");
}
