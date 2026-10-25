import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  checkPreparationRegistration,
  newCheckPreparationCases,
} from "../../tools/support/compat/github/check-preparation-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";
import { fileURLToPath } from "node:url";

const root = new URL("../../", import.meta.url);
const before = JSON.parse(
  readFileSync(
    new URL("./fixtures/check-preparation-before-source-images.json", import.meta.url),
    "utf8",
  ),
) as Record<string, string | null>;
const registration = () => structuredClone(checkPreparationRegistration);
function source(side: "base" | "head", path: string): Buffer | null {
  if (side === "base") {
    assert(Object.hasOwn(before, path), path);
    const original = before[path];
    assert.notEqual(original, undefined, path);
    return original === null ? null : Buffer.from(original);
  }
  try {
    return readFileSync(new URL(path, root));
  } catch {
    return null;
  }
}

void test("real CLI preparation registration retains exact input, producer, source pairs and batch rates", () => {
  const manifestPath = fileURLToPath(
    new URL("tests/_fixtures/differential/typechecker/manifest.json", root),
  );
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as {
    cases: Array<{ id: string }>;
    preparationCases: unknown;
  };
  assert.deepEqual(manifest.preparationCases, [registration()]);
  assert.deepEqual(newCheckPreparationCases("typechecker", [], manifest.preparationCases, source), {
    ids: [checkPreparationRegistration.id],
    sources: new Set(checkPreparationRegistration.sources.map((pair) => pair.path)),
  });
  assert.deepEqual(
    newCheckPreparationCases("typechecker", [registration()], [registration()], source),
    { ids: [], sources: new Set() },
  );
  const loaded = loadTypecheckerManifest(manifestPath);
  assert.deepEqual(loaded.manifest.cases, manifest.cases);
  assert.deepEqual(
    loaded.cases.map((entry) => entry.id),
    manifest.cases.map((entry) => entry.id),
  );
  assert(!loaded.cases.some((entry) => entry.id === checkPreparationRegistration.id));
});

void test("CLI preparation authority rejects missing bodies, changed pairs and false producer credit", () => {
  for (const path of [
    checkPreparationRegistration.input.path,
    checkPreparationRegistration.test.path,
    checkPreparationRegistration.producer.path,
    ...checkPreparationRegistration.sources.map((pair) => pair.path),
  ]) {
    for (const replacement of [null, Buffer.from("changed authority\n")]) {
      assert.throws(
        () =>
          newCheckPreparationCases("typechecker", [], [registration()], (side, current) =>
            side === "head" && current === path ? replacement : source(side, current),
          ),
        path,
      );
    }
  }
  for (const pair of checkPreparationRegistration.sources) {
    assert.throws(
      () =>
        newCheckPreparationCases("typechecker", [], [registration()], (side, path) =>
          side === "base" && path === pair.path
            ? Buffer.from("wrong before image")
            : source(side, path),
        ),
      pair.path,
    );
  }
  const changed = registration();
  assert.throws(() => newCheckPreparationCases("lsp", [], [changed], source));
  assert.throws(() => newCheckPreparationCases("typechecker", [], [changed, changed], source));
  for (const row of [
    { ...registration(), id: "typechecker/preparation/unregistered" },
    { ...registration(), targets: ["batch"] },
    { ...registration(), sources: [] },
    { ...registration(), sources: [...registration().sources].reverse() },
    { ...registration(), producer: { ...registration().producer, packets: 8 } },
    { ...registration(), producer: { ...registration().producer, binary: "/usr/local/bin/vize" } },
    { ...registration(), test: { ...registration().test, sha256: "0".repeat(64) } },
    { ...registration(), extra: "unknown field" },
  ])
    assert.throws(() => newCheckPreparationCases("typechecker", [], [row], source));
});
