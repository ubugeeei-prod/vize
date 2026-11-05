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
export const NATIVE_OPEN_OWNER_SUCCESSOR_ID =
  "typechecker/protocol/native-document-open-owners-11586";
export const NATIVE_PROJECT_IMPORT_SUCCESSOR_ID =
  "typechecker/protocol/native-document-project-imports-11586";
type Profile = {
  id: string;
  source: string;
  tree: string;
  base: string;
  path: string;
  sha256: string;
};
const profiles: readonly Profile[] = [
  {
    id: NATIVE_IDENTITY_SUCCESSOR_ID,
    source: "e3c055bbbe68d4d03f38d116a8a803e96610327f",
    tree: "8f00424ff6be4b24eb2953f58e1c119506b6946d",
    base: "372e7db0513254cefab44efbc048f86d3c9a74d8",
    path: "tests/_fixtures/differential/typechecker/native-document-identity-11586/current-source.json.gz",
    sha256: "ab03fe7250b47fa09c5ce71ba68f292d92b68342a2ca4d3ffe90f46b8c91cb12",
  },
  {
    id: NATIVE_OPEN_OWNER_SUCCESSOR_ID,
    source: "d7f0daf8a1516afe27fcc79e665a94b3b3bf02e4",
    tree: "ecd9ff968a2d164a04b0060184220f68a88cc678",
    base: "e3c055bbbe68d4d03f38d116a8a803e96610327f",
    path: "tests/_fixtures/differential/typechecker/native-document-identity-11586/open-owner-source.json.gz",
    sha256: "582f0bb255321010869031568d814713b5d0829a4d67690eed13b7347ed72924",
  },
  {
    id: NATIVE_PROJECT_IMPORT_SUCCESSOR_ID,
    source: "622262a7448d637ca2f10888c168615c29c950f9",
    tree: "2c4e59b254e78e3b1b7578b9a463c38b3f3665ca",
    base: "d7f0daf8a1516afe27fcc79e665a94b3b3bf02e4",
    path: "tests/_fixtures/differential/typechecker/native-document-identity-11586/project-import-source.json.gz",
    sha256: "20bd121b07a9ad03a98d69e2ece9f217702425e79afbb29276cb0b5ef34d1eca",
  },
];
const projectImportPaths = [
  "crates/vize_canon/src/lsp_client/lifecycle/workspace/initial_project_tests.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/project_identity.rs",
] as const;
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
function sourceImages(value: unknown, paths: readonly string[]): Record<string, string> {
  const images = object(value);
  assert.deepEqual(Object.keys(images), paths);
  for (const encoded of Object.values(images)) assert.ok(typeof encoded === "string");
  return images as Record<string, string>;
}
function sourceImage(
  value: unknown,
  profile: Profile,
  basePaths: readonly string[],
  headPaths: readonly string[],
  read: Read,
) {
  const row = object(value);
  assert.deepEqual(
    Object.keys(row).sort(),
    ["id", "image", "nativeQualification", "source", "state", "targets", "tree"].sort(),
    "closed identity successor fields differ",
  );
  assert.equal(row.id, profile.id, "unregistered identity successor profile");
  assert.equal(row.state, "active");
  assert.equal(
    row.nativeQualification,
    false,
    "source registration cannot claim native qualification",
  );
  assert.deepEqual(row.targets, ["native-document-identity"], "identity successor target differs");
  assert.equal(row.source, profile.source, "unregistered identity successor source revision");
  assert.equal(row.tree, profile.tree, "unregistered identity successor source tree");
  assert.deepEqual(row.image, { path: profile.path, sha256: profile.sha256 });
  const archive = bytes(read, "head", profile.path);
  assert.equal(
    hash(archive),
    profile.sha256,
    `changed native identity successor image: ${profile.id}`,
  );
  const image = object(JSON.parse(gunzipSync(archive).toString("utf8")));
  assert.deepEqual(Object.keys(image).sort(), ["base", "baseImages", "headImages", "source"]);
  assert.equal(image.source, profile.source);
  assert.equal(image.base, profile.base);
  return {
    base: sourceImages(image.baseImages, basePaths),
    head: sourceImages(image.headImages, headPaths),
  };
}
function actualSources(read: Read, side: "base" | "head", images: Record<string, string>) {
  for (const [path, encoded] of Object.entries(images)) {
    assert.ok(
      bytes(read, side, path).equals(Buffer.from(encoded, "base64")),
      `changed ${side} native identity successor source: ${path}`,
    );
  }
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

/** Closed, contiguous source transitions authenticate registration, never runtime acceptance. */
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
    Array.isArray(current) && current.length >= 1 && current.length <= profiles.length,
    "one to three registered identity successors required",
  );
  const retained = Array.isArray(previous) ? previous.length : 0;
  assert.ok(retained <= current.length, "registered identity successor must remain unchanged");
  assert.deepEqual(
    current.slice(0, retained),
    previous ?? [],
    "registered identity successor must remain unchanged",
  );
  if (retained === current.length) return result;

  const after = nativeDocumentIdentityActor(read, originals);
  assert.ok(Array.isArray(originals) && originals.length === 1);
  const original = object(originals[0]);
  assert.ok(Array.isArray(original.sources));
  const originalPaths = original.sources.map((value) => {
    const path = object(value).path;
    assert.ok(typeof path === "string");
    return path;
  });
  const reviewPaths = [
    ...originalPaths,
    "crates/vize_canon/src/lsp_client/editor_lsp/client/tests.rs",
    "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
  ];
  const images = current.map((value: unknown, index: number) => {
    const profile = profiles[index];
    assert.ok(profile);
    const paths = index === 2 ? [...reviewPaths, ...projectImportPaths] : reviewPaths;
    return sourceImage(value, profile, index === 0 ? originalPaths : paths, paths, read);
  });
  for (let index = 1; index < images.length; index += 1) {
    const previousHead = images[index - 1]!.head;
    const entries = Object.entries(images[index]!.base);
    assert.deepEqual(
      Object.fromEntries(entries.filter(([path]) => Object.hasOwn(previousHead, path))),
      previousHead,
      "identity successor source chain is not contiguous",
    );
    actualSources(
      read,
      "base",
      Object.fromEntries(entries.filter(([path]) => !Object.hasOwn(previousHead, path))),
    );
  }
  actualSources(read, "base", images[retained]!.base);
  actualSources(read, "head", images.at(-1)!.head);
  for (const [path, selector] of consumers) {
    assertCompiledIdentityConsumer(bytes(read, "head", path).toString("utf8"), selector, null);
  }
  if (current.length === 3) {
    for (const [path, selectors] of [
      [
        "crates/vize_canon/src/lsp_client/session/logical.rs",
        [
          "an_independent_non_authored_owner_is_not_promoted_to_logical_authority",
          "a_missing_attached_snapshot_is_an_error_not_accepted_logical_authority",
        ],
      ],
      [
        projectImportPaths[0],
        ["native_first_materialized_editor_matches_existing_activation_and_delta"],
      ],
    ] as const) {
      for (const selector of selectors)
        assertCompiledIdentityConsumer(bytes(read, "head", path).toString("utf8"), selector, null);
    }
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
  result.ids.push(...profiles.slice(retained, current.length).map((profile) => profile.id));
  result.sources = new Set(Object.keys(images.at(-1)!.head));
  return result;
}
