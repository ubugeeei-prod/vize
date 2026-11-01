import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  isVOnEventHyphenationOptions,
  vOnEventHyphenationSchema,
} from "../../npm/oxlint/src/v-on-event-options.ts";

const require = createRequire(import.meta.url);
const Ajv: typeof import("ajv").default = require("ajv");

test("Oxlint's actual option schema and projector guard preserve upstream string boundaries", () => {
  const validate = new Ajv().compile(vOnEventHyphenationSchema());
  for (const options of [
    {},
    { style: "never" },
    { ignore: [] },
    { ignore: ["update:"] },
    { ignore: ["row", "Row"] },
    { ignore: ["\u0085"] },
    { ignore: ["row:exit\n"] },
    { autofix: false },
    { autofix: true },
  ]) {
    assert.equal(validate(options), true, JSON.stringify(options));
    assert.equal(isVOnEventHyphenationOptions(options), true, JSON.stringify(options));
  }
  for (const options of [
    null,
    [],
    { style: null },
    { style: "sometimes" },
    { ignore: null },
    { ignore: "row" },
    { ignore: [0] },
    { ignore: [""] },
    { ignore: ["\uFEFF"] },
    { ignore: ["\n\t "] },
    { ignore: ["row:exit"] },
    { ignore: ["row", "row"] },
    { autofix: "true" },
    { autofix: null },
    { ignoreTags: [] },
  ]) {
    assert.equal(validate(options), false, JSON.stringify(options));
    assert.equal(isVOnEventHyphenationOptions(options), false, JSON.stringify(options));
  }
  for (const style of ["always", "never"]) assert.equal(validate(style), true);
  for (const style of ["sometimes", 1]) assert.equal(validate(style), false);
});
