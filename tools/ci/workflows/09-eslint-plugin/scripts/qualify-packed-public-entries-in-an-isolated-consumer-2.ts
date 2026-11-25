import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const source = JSON.parse(
  readFileSync(`${process.env.VIZE_ESLINT_EVIDENCE}/consumer.json`, "utf8"),
);
const packed = JSON.parse(
  readFileSync(`${process.env.VIZE_ESLINT_EVIDENCE}/packed-consumer.json`, "utf8"),
);
assert.equal(packed.nativeBuildToken, source.nativeBuildToken);
assert.deepEqual(packed.nativeImages, source.nativeImages);
