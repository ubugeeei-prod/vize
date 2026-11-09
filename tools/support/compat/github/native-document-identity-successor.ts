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
const currentSourceTransitions = [
  {
    // The physical containment check is additive; recover the complete prior
    // URI producer without granting new native qualification to this reader.
    path: "crates/vize_canon/src/lsp_client/session_paths.rs",
    before: "pub(super) fn build_materialized_session_document_uri(",
    after: `pub(super) fn is_within_physical_project_namespace(path: &Path, root: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|physical| is_within_project_namespace(&physical, root))
}

pub(super) fn build_materialized_session_document_uri(`,
  },
  {
    // The tracked logical delta replaced the full document mirror after the
    // project-import image. Materialized identities still use the full mirror.
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before: `        let documents = self
            .editor_lsp_documents_dirty
            .then(|| self.editor_documents());
        let session = self
            .editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
        if let Some(documents) = documents {
            session.synchronize(&documents)?;`,
    after: `        let dirty = self.editor_lsp_documents_dirty || self.document_texts.is_dirty();
        let documents =
            (dirty && self.materialized_project_session).then(|| self.editor_documents());
        let session = self
            .editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
        if dirty {
            if let Some(documents) = documents {
                // Materialized identities are projected by editor_documents;
                // their topology is not certified by the logical source delta.
                session.synchronize(&documents)?;
                self.document_texts.acknowledge();
            } else {
                session.synchronize_tracked(&mut self.document_texts)?;
            }`,
  },
  {
    // Shared retained texts (#12340) retype the editor document maps.
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "use super::{CorsaProjectClient, EditorLspSession, retry::retry_transient_editor_request};\n",
    after:
      "use super::{CorsaProjectClient, EditorLspSession, retry::retry_transient_editor_request};\nuse crate::lsp_client::shared_text::DocumentTexts;\n",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "        document_uri: &str,\n        documents: &FxHashMap<String, String>,\n    ) -> Result<DocumentDiagnosticReportResult, String> {",
    after:
      "        document_uri: &str,\n        documents: &DocumentTexts,\n    ) -> Result<DocumentDiagnosticReportResult, String> {",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "                let mut documents = self.document_texts.clone();\n                documents.insert(uri.into(), text.into());\n                return self.request_with_editor_lsp_documents_recovery(&documents, |session| {",
    after:
      "                let mut documents = self.document_texts.clone();\n                documents.insert(uri.into(), String::from(text).into());\n                return self.request_with_editor_lsp_documents_recovery(&documents, |session| {",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "        &mut self,\n        documents: &FxHashMap<String, String>,\n        mut request: impl FnMut(&mut EditorLspSession) -> Result<T, String>,",
    after:
      "        &mut self,\n        documents: &DocumentTexts,\n        mut request: impl FnMut(&mut EditorLspSession) -> Result<T, String>,",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "        &mut self,\n        documents: &FxHashMap<String, String>,\n    ) -> Result<&mut EditorLspSession, String> {",
    after:
      "        &mut self,\n        documents: &DocumentTexts,\n    ) -> Result<&mut EditorLspSession, String> {",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "    /// Every retained document under the same identity used by native queries.\n    pub(in crate::lsp_client) fn editor_documents(&mut self) -> FxHashMap<String, String> {\n        let retained = self",
    after:
      "    /// Every retained document under the same identity used by native queries.\n    pub(in crate::lsp_client) fn editor_documents(&mut self) -> DocumentTexts {\n        let retained = self",
  },
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    before:
      "\nfn document_maps_equal(lhs: &FxHashMap<String, String>, rhs: &FxHashMap<String, String>) -> bool {\n    lhs.len() == rhs.len()",
    after:
      "\nfn document_maps_equal(lhs: &DocumentTexts, rhs: &DocumentTexts) -> bool {\n    lhs.len() == rhs.len()",
  },
  {
    // Checked access retains the exact open-URI owner selected by binary_search.
    path: "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
    before: "            return Some(self.open_uris[index].clone());",
    after: "            return self.open_uris.get(index).cloned();",
  },
  {
    // Shared retained texts (#12340) retype this reader's document map.
    path: "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
    before:
      '        let mut capture = capture::Capture::new();\n        let matched = self.document_texts.get(uri).map(String::as_str) == Some(source);\n        capture.record("clientSourceAdmission", || {',
    after:
      '        let mut capture = capture::Capture::new();\n        let matched = self\n            .document_texts\n            .get(uri)\n            .map(super::super::shared_text::RetainedText::as_str)\n            == Some(source);\n        capture.record("clientSourceAdmission", || {',
  },
  {
    // Shared retained texts (#12340) retype this reader's document map.
    path: "crates/vize_canon/src/lsp_client/materialized_refresh.rs",
    before:
      '                uri.clone(),\n                vize_l0::String::from("export const value = 1;"),\n            )]);',
    after:
      '                uri.clone(),\n                crate::lsp_client::shared_text::RetainedText::from("export const value = 1;"),\n            )]);',
  },
  {
    // Shared retained texts (#12340) retype this reader's document map.
    path: "crates/vize_canon/src/lsp_client/session/logical.rs",
    before:
      "use crate::file_uri::file_uri_to_path;\nuse std::path::Component;\nuse vize_l0::{FxHashMap, String, cstr};\n",
    after:
      "use crate::file_uri::file_uri_to_path;\nuse crate::lsp_client::shared_text::DocumentTexts;\nuse std::path::Component;\nuse vize_l0::{String, cstr};\n",
  },
  {
    path: "crates/vize_canon/src/lsp_client/session/logical.rs",
    before:
      "        &self,\n        documents: &FxHashMap<String, String>,\n    ) -> Result<(), String> {",
    after: "        &self,\n        documents: &DocumentTexts,\n    ) -> Result<(), String> {",
  },
] as const;

/** Replay the historical tests only after authenticating both whole source directions.
 * This reader grants no new source registration or native runtime qualification.
 */
export function historicalNativeIdentityCurrentSources(read: Read): Read {
  let headImages: Record<string, unknown> | undefined;
  return (side, path) => {
    const current = read(side, path);
    // Later reviewed edits of one source extend its first transition in order.
    const transitions = currentSourceTransitions.filter((entry) => entry.path === path);
    if (side !== "head" || !current || transitions.length === 0) return current;
    if (!headImages) {
      const profile = profiles[2]!;
      const archive = bytes(read, "head", profile.path);
      assert.equal(
        hash(archive),
        profile.sha256,
        `changed native identity successor image: ${profile.id}`,
      );
      const image = object(JSON.parse(gunzipSync(archive).toString("utf8")));
      assert.equal(image.source, profile.source);
      assert.equal(image.base, profile.base);
      headImages = object(image.headImages);
    }
    const encoded = headImages[path];
    assert.ok(typeof encoded === "string");
    const original = Buffer.from(encoded, "base64");
    let expected = original.toString("utf8");
    for (const transition of transitions) {
      assert.equal(expected.split(transition.before).length, 2);
      assert.equal(expected.split(transition.after).length, 1);
      expected = expected.replace(transition.before, () => transition.after);
    }
    // Unknown bytes stay actual bytes, so the unchanged source gate refuses them.
    if (!current.equals(Buffer.from(expected))) return current;
    let after = current.toString("utf8");
    for (const transition of [...transitions].reverse()) {
      assert.equal(after.split(transition.after).length, 2);
      assert.equal(
        after.split(transition.before).length,
        transition.after.split(transition.before).length,
      );
      after = after.replace(transition.after, () => transition.before);
    }
    assert.deepEqual(Buffer.from(after), original);
    return original;
  };
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
