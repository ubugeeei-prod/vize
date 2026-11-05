import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import {
  historicalNativeIdentityCurrentSources,
  newNativeIdentitySuccessor,
  NATIVE_IDENTITY_SUCCESSOR_ID,
  NATIVE_OPEN_OWNER_SUCCESSOR_ID,
  NATIVE_PROJECT_IMPORT_SUCCESSOR_ID,
} from "../../tools/support/compat/github/native-document-identity-successor.ts";
import { newNativeDocumentIdentityCases } from "../../tools/support/compat/github/native-document-identity-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const base = "372e7db0513254cefab44efbc048f86d3c9a74d8";
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const originals = manifest.nativeDocumentIdentityCases;
const rows = manifest.nativeDocumentIdentitySuccessors;
const firstImage = JSON.parse(gunzipSync(fs.readFileSync(rows[0].image.path)).toString("utf8")) as {
  headImages: Record<string, string>;
};
type Read = (side: "base" | "head", path: string) => Buffer | null;
const baseCache = new Map<string, Buffer | null>();
const actualRead: Read = (side, path) => {
  if (side === "base" && baseCache.has(path)) return baseCache.get(path)!;
  try {
    if (side === "base") {
      const value = execFileSync("git", ["show", `${base}:${path}`], {
        stdio: ["ignore", "pipe", "ignore"],
      });
      baseCache.set(path, value);
      return value;
    }
    return fs.readFileSync(path);
  } catch {
    if (side === "base") baseCache.set(path, null);
    return null;
  }
};
const read = historicalNativeIdentityCurrentSources(actualRead);
const validate = (reader: Read = read, previous?: unknown, current: unknown = rows) =>
  newNativeIdentitySuccessor(previous, current, originals, reader);
const changed =
  (world: "base" | "head", target: string): Read =>
  (side, path) =>
    side === world && path === target
      ? Buffer.from("unregistered source mutation")
      : read(side, path);

test("current source custody inverts only the complete reviewed successors", () => {
  const image = JSON.parse(gunzipSync(fs.readFileSync(rows[2].image.path)).toString("utf8")) as {
    headImages: Record<string, string>;
  };
  const paths = [
    "crates/vize_canon/src/lsp_client/session_paths.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
  ];
  for (const path of paths) {
    const original = Buffer.from(image.headImages[path]!, "base64");
    assert.notDeepEqual(actualRead("head", path), original);
    assert.deepEqual(read("head", path), original);
    assert.deepEqual(read("base", path), actualRead("base", path));
  }
  const unrelated = "crates/vize_canon/src/lsp_client/session.rs";
  assert.deepEqual(read("head", unrelated), actualRead("head", unrelated));
  assert.throws(() => validate(actualRead), /changed head native identity successor source:/);
});

test("whole source custody leaves missing, duplicated, partial and unrelated edits unregistered", () => {
  for (const [path, currentEdit, partialEdit] of [
    [
      "crates/vize_canon/src/lsp_client/session_paths.rs",
      "        .is_ok_and(|physical| is_within_project_namespace(&physical, root))",
      "        .is_ok_and(|physical| is_within_project_namespace(root, &physical))",
    ],
    [
      "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
      "                session.synchronize_tracked(&mut self.document_texts)?;",
      "                session.synchronize_tracked(&self.document_texts)?;",
    ],
    [
      "crates/vize_maestro/src/ide/corsa_support/authored_uri.rs",
      "            return self.open_uris.get(index).cloned();",
      "            return self.open_uris.get(0).cloned();",
    ],
  ] as const) {
    const current = actualRead("head", path)!.toString("utf8");
    assert.equal(current.split(currentEdit).length, 2);
    for (const mutant of [
      `// unregistered prefix\n${current}`,
      `${current}\n// unregistered suffix\n`,
      current.replace(currentEdit, ""),
      current.replace(currentEdit, `${currentEdit}\n${currentEdit}`),
      current.replace(currentEdit, partialEdit),
      current.replace("use ", "// unrelated change\nuse "),
    ]) {
      assert.notEqual(mutant, current);
      const body = Buffer.from(mutant);
      const mutated = historicalNativeIdentityCurrentSources((side, file) =>
        side === "head" && file === path ? body : actualRead(side, file),
      );
      assert.deepEqual(mutated("head", path), body);
      assert.throws(
        () => validate(mutated),
        new RegExp(`changed head native identity successor source: ${path}`),
      );
    }
    const missing = historicalNativeIdentityCurrentSources((side, file) =>
      side === "head" && file === path ? null : actualRead(side, file),
    );
    assert.equal(missing("head", path), null);
    assert.throws(
      () => validate(missing),
      new RegExp(`missing head native identity successor source: ${path}`),
    );
  }
});

test("current source replay requires the complete unchanged project-import image", () => {
  const path = "crates/vize_canon/src/lsp_client/editor_lsp/client.rs";
  for (const body of [null, Buffer.from("unregistered archive")]) {
    const read = historicalNativeIdentityCurrentSources((side, file) =>
      side === "head" && file === rows[2].image.path ? body : actualRead(side, file),
    );
    assert.throws(
      () => read("head", path),
      body === null
        ? /missing head native identity successor source:/
        : /changed native identity successor image:/,
    );
  }
});

test("the corrective transition authenticates actual source bytes and preserves the original actor", () => {
  const result = validate();
  assert.deepEqual(result.ids, [
    NATIVE_IDENTITY_SUCCESSOR_ID,
    NATIVE_OPEN_OWNER_SUCCESSOR_ID,
    NATIVE_PROJECT_IMPORT_SUCCESSOR_ID,
  ]);
  assert.equal(result.sources.size, 21);
  assert.equal(rows[0].nativeQualification, false);
  assert.equal(rows[1].nativeQualification, false);
  assert.equal(rows[2].nativeQualification, false);
  const original = newNativeDocumentIdentityCases(undefined, originals, result.historyRead);
  assert.equal(original.sources.size, 17);
  assert.deepEqual(original.ids, [originals[0].id]);
  assert.notDeepEqual(
    result.historyRead("head", "crates/vize_canon/src/lsp_client/session.rs"),
    read("head", "crates/vize_canon/src/lsp_client/session.rs"),
  );
});

test("every actual base and head source is required with no unknown-image fallback", () => {
  const sources = validate().sources;
  for (const source of originals[0].sources as { path: string }[]) {
    assert.throws(
      () => validate(changed("base", source.path)),
      /changed base native identity successor source:/,
    );
  }
  for (const path of sources) {
    assert.throws(
      () => validate(changed("head", path)),
      /changed head native identity successor source:/,
    );
    assert.throws(
      () => validate((side, file) => (side === "head" && file === path ? null : read(side, file))),
      /missing head native identity successor source:/,
    );
  }
});

test("the current and original immutable images all refuse tampering", () => {
  for (const [path, message] of [
    [
      rows[0].image.path,
      new RegExp(`changed native identity successor image: ${NATIVE_IDENTITY_SUCCESSOR_ID}`),
    ],
    [
      rows[1].image.path,
      new RegExp(`changed native identity successor image: ${NATIVE_OPEN_OWNER_SUCCESSOR_ID}`),
    ],
    [
      rows[2].image.path,
      new RegExp(`changed native identity successor image: ${NATIVE_PROJECT_IMPORT_SUCCESSOR_ID}`),
    ],
    [originals[0].sourceArchive.path, /changed original identity preimage archive/],
    [
      "tests/_fixtures/differential/typechecker/native-document-identity-11586/after-source.json.gz",
      /changed head identity source:/,
    ],
  ] as const) {
    assert.throws(() => validate(changed("head", path)), message);
  }
});

test("a source successor still requires actual current ABI inputs and compiled producers", () => {
  for (const input of originals[0].legacyProtocol.inputs as { path: string }[]) {
    assert.throws(() => validate(changed("head", input.path)), /changed head identity source:/);
  }
  for (const consumer of originals[0].legacyProtocol.tests as { source: string }[]) {
    assert.throws(
      () => validate(changed("head", consumer.source)),
      /changed head identity source:/,
    );
  }
  assert.throws(
    () =>
      validate((side, path) =>
        side === "head" && path === "tests/tooling/support/native-beta-dto/prepare-role.ts"
          ? Buffer.concat([read(side, path)!, Buffer.from('\nvoid "changed runtime";\n')])
          : read(side, path),
      ),
    /unregistered current common-observer preparation runtime/,
  );
});

test("registered successor history cannot disappear, mutate or admit later source changes", () => {
  assert.deepEqual(
    validate(changed("head", "crates/vize_canon/src/lsp_client/session.rs"), rows).ids,
    [],
  );
  assert.equal(validate(read, rows).sources.size, 0);
  assert.throws(
    () => newNativeIdentitySuccessor(rows, undefined, originals, read),
    /registered identity successor must remain unchanged/,
  );
  const altered = structuredClone(rows);
  altered[0].source = "0".repeat(40);
  assert.throws(
    () => validate(read, rows, altered),
    /registered identity successor must remain unchanged/,
  );
  assert.throws(
    () => validate(read, undefined, altered),
    /unregistered identity successor source revision/,
  );
  assert.throws(
    () => validate(read, undefined, [...rows, ...rows]),
    /one to three registered identity successors required/,
  );
  const unknown = structuredClone(rows);
  unknown[0].unreviewed = true;
  assert.throws(
    () => validate(read, undefined, unknown),
    /closed identity successor fields differ/,
  );
  for (const [field, value, message] of [
    ["nativeQualification", true, /source registration cannot claim native qualification/],
    ["tree", "0".repeat(40), /unregistered identity successor source tree/],
    ["targets", ["unreviewed"], /identity successor target differs/],
  ] as const) {
    for (const index of [0, 1, 2]) {
      const altered = structuredClone(rows);
      altered[index][field] = value;
      assert.throws(() => validate(read, undefined, altered), message);
    }
  }
});

test("the original successor stays immutable and the second image extends its exact source state", () => {
  const first = rows.slice(0, 1);
  const firstReader: Read = (side, path) =>
    side === "head" && Object.hasOwn(firstImage.headImages, path)
      ? Buffer.from(firstImage.headImages[path]!, "base64")
      : read(side, path);
  assert.deepEqual(validate(firstReader, undefined, first).ids, [NATIVE_IDENTITY_SUCCESSOR_ID]);
  assert.throws(
    () => validate(read, undefined, first),
    /changed head native identity successor source:/,
  );
  const appendedReader: Read = (side, path) =>
    side === "base" && Object.hasOwn(firstImage.headImages, path)
      ? Buffer.from(firstImage.headImages[path]!, "base64")
      : read(side, path);
  const appended = validate(appendedReader, first);
  assert.deepEqual(appended.ids, [
    NATIVE_OPEN_OWNER_SUCCESSOR_ID,
    NATIVE_PROJECT_IMPORT_SUCCESSOR_ID,
  ]);
  assert.equal(appended.sources.size, 21);
  const path = "crates/vize_canon/src/lsp_client/session.rs";
  assert.throws(
    () =>
      validate(
        (side, file) =>
          side === "base" && file === path
            ? Buffer.from("unregistered intermediate source")
            : appendedReader(side, file),
        first,
      ),
    /changed base native identity successor source:/,
  );
  assert.throws(
    () => validate(read, undefined, [rows[0], rows[0]]),
    /unregistered identity successor profile/,
  );
  assert.throws(
    () => validate(read, undefined, [rows[1], rows[0]]),
    /unregistered identity successor profile/,
  );
  assert.throws(
    () => validate(read, rows, first),
    /registered identity successor must remain unchanged/,
  );
  const changedPrefix = structuredClone(rows);
  changedPrefix[0].tree = "0".repeat(40);
  assert.throws(
    () => validate(appendedReader, first, changedPrefix),
    /registered identity successor must remain unchanged/,
  );
});

test("the project-import successor extends the published pair and authenticates newly registered base files", () => {
  const retained = rows.slice(0, 2);
  const image = JSON.parse(gunzipSync(fs.readFileSync(rows[1].image.path)).toString("utf8")) as {
    headImages: Record<string, string>;
  };
  const priorHead: Read = (side, path) =>
    side === "head" && Object.hasOwn(image.headImages, path)
      ? Buffer.from(image.headImages[path]!, "base64")
      : read(side, path);
  assert.deepEqual(validate(priorHead, undefined, retained).ids, [
    NATIVE_IDENTITY_SUCCESSOR_ID,
    NATIVE_OPEN_OWNER_SUCCESSOR_ID,
  ]);
  assert.throws(
    () => validate(read, undefined, retained),
    /changed head native identity successor source:/,
  );
  const appendedReader: Read = (side, path) =>
    side === "base" && Object.hasOwn(image.headImages, path)
      ? Buffer.from(image.headImages[path]!, "base64")
      : read(side, path);
  assert.deepEqual(validate(appendedReader, retained).ids, [NATIVE_PROJECT_IMPORT_SUCCESSOR_ID]);
  for (const path of [
    "crates/vize_canon/src/lsp_client/lifecycle/workspace/initial_project_tests.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/project_identity.rs",
  ]) {
    assert.throws(
      () => validate(changed("base", path)),
      new RegExp(`changed base native identity successor source: ${path}`),
    );
    assert.throws(
      () =>
        validate(
          (side, file) =>
            side === "base" && file === path
              ? Buffer.from("unregistered newly covered preimage")
              : appendedReader(side, file),
          retained,
        ),
      /changed base native identity successor source:/,
    );
  }
});

test("the mandatory source gate admits only the complete current corrective transition", () => {
  const paths = [...validate().sources];
  // Later authored-language inputs cannot stand in for this source transition's
  // unrelated-source negative control. Keep those cases in both comparison sides.
  const before = JSON.parse(read("base", manifestPath)!.toString("utf8"));
  before.cases = manifest.cases;
  const sourceRead: Read = (side, path) =>
    side === "base" && path === manifestPath
      ? Buffer.from(JSON.stringify(before))
      : read(side, path);
  const report = (registered: boolean) =>
    analyzeLegacyFix({
      title: "fix(lsp): retain URI owners and every remapped edit",
      paths,
      readFile: (side, path) => {
        if (side === "head" && path === manifestPath && !registered) {
          const without = structuredClone(manifest);
          delete without.nativeDocumentIdentitySuccessors;
          return Buffer.from(JSON.stringify(without));
        }
        return sourceRead(side, path);
      },
    }).products.find((product) => product.product === "typechecker");
  assert.equal(report(false)?.state, "invalid-manifest-or-input");
  const admitted = report(true);
  assert.equal(admitted?.state, "new-inputs-registered", admitted?.reason);
  assert.ok(admitted?.cases.includes(NATIVE_IDENTITY_SUCCESSOR_ID));
  assert.ok(admitted?.cases.includes(NATIVE_OPEN_OWNER_SUCCESSOR_ID));
  assert.ok(admitted?.cases.includes(NATIVE_PROJECT_IMPORT_SUCCESSOR_ID));
  const unrelated = analyzeLegacyFix({
    title: "fix(lsp): retain URI owners and every remapped edit",
    paths: [...paths, "crates/vize_canon/src/lsp_client/unrelated.rs"],
    readFile: sourceRead,
  }).products.find((product) => product.product === "typechecker");
  assert.equal(unrelated?.state, "new-input-missing");
  const future = analyzeLegacyFix({
    title: "fix(lsp): retain URI owners and every remapped edit",
    paths,
    readFile: (side, path) =>
      side === "base" && path === manifestPath
        ? Buffer.from(JSON.stringify(manifest))
        : read(side, path),
  }).products.find((product) => product.product === "typechecker");
  assert.equal(future?.state, "new-input-missing");
});
