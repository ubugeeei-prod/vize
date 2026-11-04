import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import {
  assertCompiledIdentityConsumer,
  assertNativeDocumentIdentityAbi,
  nativeDocumentIdentityActor,
} from "./native-document-identity-fixtures.ts";

type Read = (side: "base" | "head", path: string) => Buffer | null;
export const NATIVE_IDENTITY_SUCCESSOR_ID =
  "typechecker/protocol/native-document-identity-review-11586";
const imagePath =
  "tests/_fixtures/differential/typechecker/native-document-identity-11586/current-source.json.gz";
const imageSha = "ab03fe7250b47fa09c5ce71ba68f292d92b68342a2ca4d3ffe90f46b8c91cb12";
const source = "e3c055bbbe68d4d03f38d116a8a803e96610327f";
const tree = "8f00424ff6be4b24eb2953f58e1c119506b6946d";
const base = "372e7db0513254cefab44efbc048f86d3c9a74d8";
const hash = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown): Record<string, unknown> {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "identity successor requires an object",
  );
  return value as Record<string, unknown>;
}
function bytes(read: Read, side: "base" | "head", path: string): Buffer {
  const value = read(side, path);
  assert.ok(value, `missing ${side} native identity successor source: ${path}`);
  return value;
}
const consumers = [
  [
    "crates/vize_canon/src/lsp_client/editor_lsp/client/tests.rs",
    "hierarchy_items_preserve_unmapped_foreign_documents",
  ],
  [
    "crates/vize_canon/src/lsp_client/editor_lsp/file_rename.rs",
    "materialized_renames_share_one_native_namespace_and_keep_each_caller_context",
  ],
  [
    "crates/vize_canon/src/lsp_client/session/tests.rs",
    "moving_one_spelling_preserves_the_other_native_identity_owner",
  ],
  [
    "crates/vize_canon/src/lsp_client/session/tests.rs",
    "releasing_either_normalized_owner_keeps_the_other_until_the_last_close",
  ],
  [
    "crates/vize_canon/src/lsp_client/utils.rs",
    "converging_workspace_edit_keys_keep_every_edit_in_each_original_order",
  ],
  [
    "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
    "closed_foreign_targets_keep_the_import_path_before_an_open_owner_takes_precedence",
  ],
] as const;

/** One closed source transition authenticates registration, never runtime acceptance. */
export function newNativeIdentitySuccessor(
  previous: unknown,
  current: unknown,
  originals: unknown,
  read: Read,
) {
  const result = { ids: [] as string[], sources: new Set<string>(), historyRead: read };
  assert.ok(previous === undefined || Array.isArray(previous));
  if (current === undefined) {
    assert.ok(
      previous === undefined || previous.length === 0,
      "registered identity successor must remain unchanged",
    );
    return result;
  }
  assert.ok(
    Array.isArray(current) && current.length === 1,
    "one registered identity successor required",
  );
  if (Array.isArray(previous) && previous.length) {
    assert.deepEqual(current, previous, "registered identity successor must remain unchanged");
    return result;
  }
  const row = object(current[0]);
  assert.deepEqual(
    Object.keys(row).sort(),
    ["id", "image", "nativeQualification", "source", "state", "targets", "tree"].sort(),
    "closed identity successor fields differ",
  );
  assert.equal(row.id, NATIVE_IDENTITY_SUCCESSOR_ID);
  assert.equal(row.state, "active");
  assert.equal(row.nativeQualification, false);
  assert.deepEqual(row.targets, ["native-document-identity"]);
  assert.equal(row.source, source, "unregistered identity successor source revision");
  assert.equal(row.tree, tree, "unregistered identity successor source tree");
  assert.deepEqual(row.image, { path: imagePath, sha256: imageSha });

  const after = nativeDocumentIdentityActor(read, originals);
  assert.ok(Array.isArray(originals) && originals.length === 1);
  const original = object(originals[0]);
  assert.ok(Array.isArray(original.sources));
  const originalPaths = original.sources.map((value) => {
    const path = object(value).path;
    assert.ok(typeof path === "string");
    return path;
  });
  const currentPaths = [
    ...originalPaths,
    "crates/vize_canon/src/lsp_client/editor_lsp/client/tests.rs",
    "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
  ];
  const archive = bytes(read, "head", imagePath);
  assert.equal(hash(archive), imageSha, "changed native identity successor image");
  const image = object(JSON.parse(gunzipSync(archive).toString("utf8")));
  assert.deepEqual(Object.keys(image).sort(), ["base", "baseImages", "headImages", "source"]);
  assert.equal(image.source, source);
  assert.equal(image.base, base);
  const baseImages = object(image.baseImages);
  const headImages = object(image.headImages);
  assert.deepEqual(Object.keys(baseImages), originalPaths);
  assert.deepEqual(Object.keys(headImages), currentPaths);
  for (const [side, images] of [
    ["base", baseImages],
    ["head", headImages],
  ] as const) {
    for (const [path, encoded] of Object.entries(images)) {
      assert.ok(typeof encoded === "string");
      assert.ok(
        bytes(read, side, path).equals(Buffer.from(encoded, "base64")),
        `changed ${side} native identity successor source: ${path}`,
      );
    }
  }
  for (const [path, selector] of consumers) {
    assertCompiledIdentityConsumer(bytes(read, "head", path).toString("utf8"), selector, null);
  }
  // Historical replay never substitutes for actual current ABI/producer custody.
  assertNativeDocumentIdentityAbi(read, original.legacyProtocol);
  const beforeArchive = object(original.sourceArchive);
  assert.ok(typeof beforeArchive.path === "string");
  const beforeBytes = bytes(read, "head", beforeArchive.path);
  assert.equal(
    hash(beforeBytes),
    beforeArchive.sha256,
    "changed original identity preimage archive",
  );
  const beforeImages = object(JSON.parse(gunzipSync(beforeBytes).toString("utf8")));
  assert.deepEqual(Object.keys(beforeImages), originalPaths);
  const before = new Map<string, Buffer>();
  for (const value of original.sources) {
    const entry = object(value);
    const path = entry.path as string;
    const encoded = beforeImages[path];
    assert.ok(typeof encoded === "string");
    const image = Buffer.from(encoded, "base64");
    assert.equal(hash(image), entry.before, `changed original identity preimage: ${path}`);
    before.set(path, image);
  }
  result.historyRead = (side, path) =>
    (side === "base" ? before : after).get(path) ?? read(side, path);
  result.ids.push(NATIVE_IDENTITY_SUCCESSOR_ID);
  result.sources = new Set(currentPaths);
  return result;
}
