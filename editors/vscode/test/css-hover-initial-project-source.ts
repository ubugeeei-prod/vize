import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type { SourceTreeEntry } from "./css-hover-foreground-source.ts";

export const initialProjectTypedSourceSha256 =
  "8b7f11857234b5e2b1ad41bd8777993e60303589a5d64eb1b393ef307a27449e";
export const foregroundTypedSourceSha256 =
  "752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac";
export const initialProjectTypedEntry: SourceTreeEntry = Object.freeze({
  path: "crates/vize_maestro/src/server/handlers/typed.rs",
  mode: "100644",
  type: "blob",
  oid: "80ba6f52a001082d23f117172e5045acb99dcaa8",
});
const foregroundBlob = "a4b1e81960029f488921f917e46589f1d7655da9";
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array): string =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

const context = "        let mut ctx = ctx.with_foreground_preparation(preparation);\n";
const oldContext = "        let ctx = ctx.with_foreground_preparation(preparation);\n";
const nativeVue = `            let stamp = self.state.corsa_request_stamp();
            let (corsa_bridge, initial) = self
                .state
                .get_corsa_bridge_for_canonical_source(ctx.uri, &ctx.content)
                .await;
            ctx.attach_initial_canonical_document(corsa_bridge.as_ref(), initial, stamp);
`;
const oldNativeVue = "            let corsa_bridge = self.state.get_corsa_bridge().await;\n";

function replaceExactly(source: string, selected: string, replacement: string): string {
  const pieces = source.split(selected);
  assert.equal(pieces.length, 3, "foreign initial-project source structure");
  return pieces.join(replacement);
}

// The entire new body must match before this finite inverse runs. Only the two
// native Vue bridge blocks and their two mutable contexts are restored. CSS,
// static members, JSX, authored URI/content and preparation order are not erased.
export function invertInitialProjectSource(bytes: unknown): Buffer {
  assert.ok(Buffer.isBuffer(bytes), "complete committed initial-project bytes required");
  assert.equal(hash(bytes), initialProjectTypedSourceSha256, "unreviewed initial-project bytes");
  assert.equal(blob(bytes), initialProjectTypedEntry.oid, "unreviewed initial-project blob");
  let original = replaceExactly(bytes.toString("utf8"), context, oldContext);
  original = replaceExactly(original, nativeVue, oldNativeVue);
  const restored = Buffer.from(original);
  assert.equal(
    hash(restored),
    foregroundTypedSourceSha256,
    "whole initial-project inverse differs",
  );
  assert.equal(blob(restored), foregroundBlob, "whole foreground typed blob differs");
  return restored;
}
