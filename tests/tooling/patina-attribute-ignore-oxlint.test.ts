import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  attributeHyphenationSchema,
  getAttributeHyphenationOptions,
} from "../../npm/oxlint/src/attribute-options.ts";
const require = createRequire(import.meta.url);
const Ajv: typeof import("ajv").default = require("ajv");

test("actual Oxlint tuple schema and projector preserve literal ignore boundaries", () => {
  const validate = new Ajv().compile({
    type: "array",
    items: attributeHyphenationSchema(),
    maxItems: 2,
  });
  for (const options of [
    ["always"],
    ["never", { ignore: ["active"] }],
    [{}],
    [{ ignore: [] }],
    [{ ignore: ["\u0085", "name:exit\n"] }],
    [{ style: "never", ignore: ["active", "Active"] }],
    [{ ignoreTags: [] }],
  ]) {
    assert.equal(validate(options), true, JSON.stringify(options));
    const before = JSON.stringify(options);
    assert(getAttributeHyphenationOptions(options));
    assert.equal(JSON.stringify(options), before);
  }
  for (const options of [
    ["sometimes"],
    [{ style: null }],
    [{ ignore: null }],
    [{ ignore: [1] }],
    [{ ignore: [""] }],
    [{ ignore: ["\uFEFF"] }],
    [{ ignore: ["name:exit"] }],
    [{ ignore: ["active", "active"] }],
    ["always", { style: "never" }],
    ["always", {}, {}],
  ]) {
    assert.equal(validate(options), false, JSON.stringify(options));
    assert.equal(getAttributeHyphenationOptions(options), undefined, JSON.stringify(options));
  }
  assert.throws(
    () => getAttributeHyphenationOptions([{}, {}]),
    /canonical object or a style\/ignore tuple/,
  );
});
