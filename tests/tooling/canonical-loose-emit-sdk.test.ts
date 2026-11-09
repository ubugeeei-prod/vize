import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { emitInferenceExports } from "../../npm/cli/scripts/emit-inference-exports.ts";
import { beforeGenericEventSpecialization } from "./support/generic-event-specialization-source.ts";
import { beforeSharedListenerRelationSource } from "./support/listener-relation-source-transition.ts";
import { beforeCanonLeftoversSource } from "./support/canon-leftovers-output-transition.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const files = ["ts", "mts", "cts"].map((extension) => `npm/cli/virtual-ts.d.${extension}`);
const leaf = "npm/cli/virtual-ts-emits.d.ts";
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const hash = (text: string) => createHash("sha256").update(text).digest("hex");

void test("all public emit factories exactly follow the canonical producer", async () => {
  const outputs = await emitInferenceExports(root, new Map());
  assert.deepEqual([...outputs.keys()], [leaf, ...files]);
  assert.equal(outputs.get(leaf), read(leaf));
  assert.ok(read(leaf).endsWith("export type { $NativeEmitPropsFactory, $EmitPropsFactory };\n"));
  for (const [file, source] of outputs) {
    assert.equal(source, read(file));
    if (file === leaf) continue;
    assert.equal(source.match(/^export type \$EmitPropsFactory</gmu)?.length, 1);
    assert.equal(source.match(/^export type \$ConstructorEmitPropsFactory</gmu)?.length, 1);
    for (const helper of ["$LooseEmitProps", "$LooseEmitPropsFactory", "$EmitIsWide"]) {
      assert.equal(source.includes(`type ${helper}`), false);
    }
    const start = source.indexOf("import type * as __vizeEmitTypes");
    const end = source.indexOf("// Component usage helpers", start);
    assert.ok(start >= 0 && end > start);
    assert.equal(source.slice(start, end).includes("$ListenerArgs"), false);
  }
});

void test("unknown, partial, duplicate, foreign, and newly exported families are refused", async () => {
  const file = files[0]!;
  const original = read(file);
  const mutations = [
    original.replace(
      "export type $ConstructorEmitPropsFactory",
      "type $ConstructorEmitPropsFactory",
    ),
    original.replace("__vizeEmitTypes.$NativeEmitPropsFactory<C>", "C"),
    original.replace("./virtual-ts-emits.js", "./unknown-emits.js"),
    original + "\nexport type $EmitPropsFactory<C> = C;\n",
    original.replace(
      "type $NativeEmitPropsFactory<",
      "type ForeignFactory = true;\ntype $NativeEmitPropsFactory<",
    ),
  ];
  for (const source of mutations) {
    assert.notEqual(source, original);
    await assert.rejects(() => emitInferenceExports(root, new Map([[file, source]])));
    assert.equal(read(file), original, "rejected input cannot change SDK bytes");
  }
  const originalLeaf = read(leaf);
  for (const source of [
    originalLeaf.replace(
      "readonly __vizeLooseListener: []",
      "readonly __vizeLooseListener: unknown[]",
    ),
    originalLeaf.replace(/^type \$LooseEmitProps = .*;\n/gmu, ""),
    originalLeaf + "\nexport type $EmitPropsFactory<C> = C;\n",
    originalLeaf.replace("type $LooseEmitProps =", "export type $LooseEmitProps ="),
    originalLeaf.replace("export type { $NativeEmitPropsFactory, $EmitPropsFactory };", ""),
  ]) {
    assert.notEqual(source, originalLeaf);
    await assert.rejects(() => emitInferenceExports(root, new Map([[leaf, source]])));
    assert.equal(read(leaf), originalLeaf);
  }
  const shared =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt";
  const changed = read(shared).replace(
    "readonly __vizeLooseListener: []",
    "readonly __vizeLooseListener: unknown[]",
  );
  await assert.rejects(() => emitInferenceExports(root, new Map([[shared, changed]])));
});

void test("the private module repair exactly restores the authenticated SDK sync", () => {
  const corpus =
    "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/";
  const oldBytes = read(corpus + "canonical-loose-emit-sdk-update-11815.json");
  assert.equal(hash(oldBytes), "b39a4a47a11d2e343c2cec25a899830c0ac13403b9445d2bc73715f533e22247");
  type Identity = { bytes: number; sha256: string };
  type Entry = {
    path: string;
    before: Identity;
    after: Identity;
    edits: { before: string; after: string }[];
  };
  const old = JSON.parse(oldBytes) as {
    files: Entry[];
    newAssets: (Identity & { path: string })[];
  };
  const bytes = read(corpus + "private-emit-sdk-update-11815.json");
  const bridge = read(
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/private_emit_sdk.rs",
  );
  assert.ok(bridge.includes(`const RECORD_SHA: &str = "${hash(bytes)}";`));
  const record = JSON.parse(bytes) as {
    files: Entry[];
    newAssets: (Identity & { path: string })[];
    nativeAcceptanceCredit: boolean;
  };
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.files.length, 5);
  assert.equal(record.newAssets.length, 1);
  const identity = (text: string) => ({ bytes: Buffer.byteLength(text), sha256: hash(text) });
  const beforeOverloadSuccessor = (file: string, text: string) => {
    if (files.includes(file)) {
      const addition =
        'import type {} from "./virtual-ts-emit-overloads.js" with { "resolution-mode": "import" };\n';
      assert.equal(text.split(addition).length, 2);
      return text.replace(addition, "");
    }
    if (file === "npm/cli/package.json") {
      const before = '    "virtual-ts-globals.d.ts",\n';
      const after = '    "virtual-ts-emit-overloads.d.ts",\n' + before;
      assert.equal(text.split(after).length, 2);
      return text.replace(after, before);
    }
    return text;
  };
  for (const entry of record.files) {
    const inverse = (current: string) => {
      assert.deepEqual(identity(current), entry.after);
      for (const edit of entry.edits) {
        assert.ok(edit.after);
        assert.equal(current.split(edit.after).length, 2);
        current = current.replace(edit.after, edit.before);
      }
      assert.deepEqual(identity(current), entry.before);
      return current;
    };
    const current = beforeGenericEventSpecialization(
      read,
      entry.path,
      beforeOverloadSuccessor(
        entry.path,
        beforeSharedListenerRelationSource(
          entry.path,
          beforeCanonLeftoversSource(entry.path, read(entry.path)),
        ),
      ),
    );
    const previous = inverse(current);
    const origin = old.files.find((file) => file.path === entry.path)?.after;
    const asset = old.newAssets.find((file) => file.path === entry.path);
    if (origin || asset)
      assert.deepEqual(
        identity(previous),
        origin ?? { bytes: asset!.bytes, sha256: asset!.sha256 },
      );
    for (const changed of [
      current + "\n",
      current + current,
      current.replace(entry.edits[0]!.after, ""),
    ]) {
      assert.throws(() => inverse(changed));
    }
  }
  for (const asset of record.newAssets) {
    assert.deepEqual(
      identity(
        beforeGenericEventSpecialization(
          read,
          asset.path,
          beforeSharedListenerRelationSource(
            asset.path,
            beforeCanonLeftoversSource(asset.path, read(asset.path)),
          ),
        ),
      ),
      {
        bytes: asset.bytes,
        sha256: asset.sha256,
      },
    );
  }
});
