import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { nativeLoaderProgram } from "../../../tools/support/release/human_reports_11252_11253_public/native-loader-program.ts";

const addedExports =
  "module.exports.typeProjectInputs = nativeBinding.typeProjectInputs;\n" +
  "module.exports.typeProjectOwners = nativeBinding.typeProjectOwners;\n";
const sha256 = (body: Buffer) => createHash("sha256").update(body).digest("hex");

/** Keep the original authenticated loader law through exactly two API reexports. */
export function originalNativeLoaderModule(name: string, body: Buffer): Buffer {
  const reference = nativeLoaderProgram.modules.find((module) => module.name === name);
  assert.ok(reference, "unknown native loader module");
  if (body.length === reference.bytes && sha256(body) === reference.sha256) return body;
  assert.equal(name, "index.js", "only the public project API facade changed");
  const text = body.toString("utf8");
  assert.deepEqual(Buffer.from(text), body);
  assert.equal(text.split(addedExports).length, 2, "exact project API export addition");
  const original = Buffer.from(text.replace(addedExports, ""));
  assert.equal(original.length, reference.bytes, "complete original native loader length");
  assert.equal(sha256(original), reference.sha256, "complete original native loader program");
  return original;
}
