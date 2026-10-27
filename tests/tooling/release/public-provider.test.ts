import assert from "node:assert/strict";
import { test } from "node:test";
import { validatePublicProvider } from "../../../tools/support/release/public-provider.ts";

const version = "0.602.0";
const source = "92223735e9c41cefbfe584d7f364e4847569c92b";
const image = { bytes: 123, sha256: "a".repeat(64) };
const provider = { path: "extension/native/vize.node", ...image };
function receipt() {
  return { version, source, provider: { files: [provider] } };
}
test("a public native provider must match the exact release source and loaded image", () => {
  assert.equal(validatePublicProvider(receipt(), version, source, image).source, source);
  for (const packet of [
    { ...receipt(), version: "0.601.0" },
    { ...receipt(), source: "0".repeat(40) },
    { ...receipt(), provider: { files: [] } },
    { ...receipt(), provider: { files: [provider, provider] } },
    { ...receipt(), provider: { files: [{ ...provider, bytes: 124 }] } },
    { ...receipt(), provider: { files: [{ ...provider, sha256: "b".repeat(64) }] } },
  ]) {
    assert.throws(() => validatePublicProvider(packet, version, source, image));
  }
});
