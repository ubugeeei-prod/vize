import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import Ajv2020 from "ajv/dist/2020.js";

const repositoryRoot = fileURLToPath(new URL("../..", import.meta.url));
const schema = JSON.parse(
  readFileSync(
    `${repositoryRoot}/crates/vize_marquette/schema/test-run-evidence.schema.json`,
    "utf8",
  ),
);

test("test-run schema satisfies its declared dialect", () => {
  const validator = new Ajv2020({
    strict: true,
    validateFormats: false,
  });

  assert.equal(validator.validateSchema(schema), true, validator.errorsText());
  assert.doesNotThrow(() => validator.compile(schema));
});

test("dialect validation rejects malformed schemas", () => {
  const validator = new Ajv2020({
    strict: true,
    validateFormats: false,
  });
  const invalidSchema = {
    ...schema,
    type: 42,
  };

  assert.equal(validator.validateSchema(invalidSchema), false);
});
