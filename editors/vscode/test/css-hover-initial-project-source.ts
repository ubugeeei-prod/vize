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
export const residentMarkupTypedSourceSha256 =
  "584b9454b4f846de7515281df783974e6f96754f2f3d1c0481aadff0890421e8";
export const residentMarkupTypedEntry: SourceTreeEntry = Object.freeze({
  path: "crates/vize_maestro/src/server/handlers/typed.rs",
  mode: "100644",
  type: "blob",
  oid: "f5fdd7878687c044e9f6d948de3d874ebbeae431",
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
const residentMarkupDispatch = `        } else if let Some(hover) = HoverService::hover_resident_markup(&ctx) {
            Some(hover)
`;

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

// Authenticate the complete resident-markup dispatch body before removing only
// its early return. The result must be the complete reviewed initial-project body.
export function invertResidentMarkupSource(bytes: unknown): Buffer {
  assert.ok(Buffer.isBuffer(bytes), "complete committed resident-markup bytes required");
  assert.equal(hash(bytes), residentMarkupTypedSourceSha256, "unreviewed resident-markup bytes");
  assert.equal(blob(bytes), residentMarkupTypedEntry.oid, "unreviewed resident-markup blob");
  const pieces = bytes.toString("utf8").split(residentMarkupDispatch);
  assert.equal(pieces.length, 2, "foreign resident-markup source structure");
  const restored = Buffer.from(pieces.join(""));
  assert.equal(
    hash(restored),
    initialProjectTypedSourceSha256,
    "whole resident-markup inverse differs",
  );
  assert.equal(blob(restored), initialProjectTypedEntry.oid, "whole initial-project blob differs");
  return restored;
}
