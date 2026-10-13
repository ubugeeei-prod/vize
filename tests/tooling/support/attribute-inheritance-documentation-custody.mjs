import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { codeFences } from "./public-vue-imports-doc-fence-custody.mjs";

export const inheritanceDocumentationFixture =
  "tests/_fixtures/differential/linter/attribute-inheritance-reference/documentation-successor.json.gz";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const documentPaths = [
  "docs/content/ja/rules/reference/script-no-duplicate-attr-inheritance.md",
  "docs/content/rules/reference/script-no-duplicate-attr-inheritance.md",
  ...["en", "ja", "zh"].map(
    (language) => `crates/vize/src/commands/explain/snapshots/${language}.txt`,
  ),
];
const beforeBad = "defineOptions({ inheritAttrs: true })\nexport default { inheritAttrs: true }";
const beforeGood =
  "defineOptions({ inheritAttrs: false }) // intentional opt-out\n" +
  "export default {}                      // default inheritance, unstated";
const template = '<template><div><button v-bind="$attrs">Save</button></div></template>';
const afterBad = "<script setup></script>\n" + template;
const afterGood = "<script setup>defineOptions({ inheritAttrs: false })</script>\n" + template;

function replaceOnce(source, before, after) {
  assert.equal(source.split(before).length, 2, "one complete approved edit");
  const successor = source.replace(before, after);
  assert.equal(successor.split(after).length, 2, "unambiguous complete inverse");
  assert.equal(successor.replace(after, before), source);
  return successor;
}

export function composeInheritanceDocument(before, filename) {
  assert(documentPaths.includes(filename), "only the five named complete documents");
  if (filename.includes("/snapshots/")) {
    const start = "=== script/no-duplicate-attr-inheritance\n";
    const end = "=== script/no-export-in-script-setup\n";
    assert.equal(before.split(start).length, 2);
    assert.equal(before.split(end).length, 2);
    const packet = before.split(start)[1].split(end)[0];
    const indent = (text) =>
      text
        .split("\n")
        .map((line) => "  " + line)
        .join("\n");
    const changed = replaceOnce(
      replaceOnce(packet, indent(beforeBad), indent(afterBad)),
      indent(beforeGood),
      indent(afterGood),
    );
    return replaceOnce(before, start + packet + end, start + changed + end);
  }
  let after = replaceOnce(
    before,
    '```vue\n<script lang="ts">\n' + beforeBad + "\n</script>\n```",
    "```vue\n" + afterBad + "\n```",
  );
  after = replaceOnce(
    after,
    '```vue\n<script lang="ts">\n' + beforeGood + "\n</script>\n```",
    "```vue\n" + afterGood + "\n```",
  );
  const paragraphs = filename.includes("/ja/")
    ? [
        [
          "Vue の既定で有効な属性継承を、`inheritAttrs: true` であらためて指定しています。このルールは、ルート要素に `$attrs` を展開するコードがない場合も、この冗長な指定を報告します。",
          "自動の属性継承を有効にしたまま、ボタンにも `$attrs` を転送しています。同じ属性が二重に適用される可能性があります。",
        ],
        [
          "`inheritAttrs: false` は継承を無効にする指定であり、空のオプションは既定の継承を暗黙に使います。どちらも冗長な `true` を指定しません。",
          "`inheritAttrs: false` で自動の継承を無効にし、ボタンへの明示的な転送だけを残します。",
        ],
      ]
    : [
        [
          "The explicit `inheritAttrs: true` values restate Vue’s default. This rule reports that redundant literal even when no root `$attrs` spread is shown.",
          "The component keeps automatic attribute inheritance and also forwards `$attrs` to its button. The same fallthrough attributes can be applied twice.",
        ],
        [
          "`inheritAttrs: false` expresses a real opt-out, while the empty options object leaves default inheritance implicit. Neither restates the redundant `true` value.",
          "`inheritAttrs: false` disables automatic inheritance, leaving the explicit button binding as the single forwarding path.",
        ],
      ];
  for (const [oldParagraph, newParagraph] of paragraphs)
    after = replaceOnce(after, oldParagraph, newParagraph);
  return replaceOnce(
    after,
    "no_duplicate_attr_inheritance.rs#L73",
    "no_duplicate_attr_inheritance.rs#L35",
  );
}

export function decodeInheritanceBody(row) {
  const body = Buffer.from(row.base64, "base64");
  assert.equal(body.toString("base64"), row.base64);
  assert.equal(body.length, row.bytes);
  assert.equal(digest(body), row.sha256);
  assert.equal(blob(body), row.gitBlob);
  return body;
}

export function validateInheritanceDocumentation(successor) {
  assert.equal(successor.schema, "vize.lint.attribute-inheritance.documentation-successor");
  assert.equal(successor.version, 1);
  assert.equal(successor.source, "3f5dcc8001dbd4a258aaac3a2a8e8bcaedec190e");
  assert.equal(successor.main, "2b8c61a3fdb46fb5117c4730b8ec65080ffa6ec2");
  assert.deepEqual(
    successor.documents.map((row) => row.path),
    documentPaths,
  );
  const custody = JSON.parse(decodeInheritanceBody(successor.custody));
  assert.equal(custody.entries.length, 317);
  const entry = custody.entries.find((row) => row.id === "script/no-duplicate-attr-inheritance");
  for (const row of successor.documents) {
    assert.equal(row.before.path, row.path);
    assert.equal(row.after.path, row.path);
    const before = decodeInheritanceBody(row.before).toString();
    const after = decodeInheritanceBody(row.after).toString();
    assert.equal(after, composeInheritanceDocument(before, row.path));
    if (!row.path.includes("/snapshots/")) {
      assert([entry.path, entry.englishPath].includes(row.path));
      assert.equal(digest(codeFences(before).join("\0")), entry.wholeCodeSha256);
    }
  }
  assert.deepEqual(
    successor.sourceAuthority.map(({ base64: _base64, ...identity }) => identity),
    [
      {
        path: "crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs",
        bytes: 5084,
        sha256: "fd04094d6360b37639e017c7384e4df82eeb93b4c31226d4cbce30a24888f73f",
        gitBlob: "f0fc2dc2a379396eb21fb7e687cba3097b098ce6",
      },
      {
        path: "crates/vize_patina/tests/fixtures/attribute-inheritance-reference/cases.json",
        bytes: 51501,
        sha256: "8738f3d81b86ab7986ad456148524716c9a49f297e272d99643e3dbaa19cebac",
        gitBlob: "fc823fecf37b7de33ba0db552e1370ececbf65c8",
      },
    ],
  );
  for (const row of successor.sourceAuthority) decodeInheritanceBody(row);
  const composition = successor.composition;
  assert.equal(composition.source, "ecc78c9a34f49198c092f5e306218c23cfb90372");
  const pr = JSON.parse(decodeInheritanceBody(composition.pr));
  const main = JSON.parse(decodeInheritanceBody(composition.main));
  const current = JSON.parse(decodeInheritanceBody(composition.after));
  const changedPaths = [
    "crates/vize_maestro/src/ide/diagnostics/default_migration_tests.rs",
    "crates/vize_patina/src/linter/script_rules.rs",
    "crates/vize_patina/src/linter/script_rules/sfc.rs",
    "crates/vize_patina/src/preset/default_migration_tests.rs",
    "crates/vize_patina/src/rules/script.rs",
    "crates/vize_patina/src/rules/script/props_emits.rs",
    "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs",
    "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default/typed.rs",
    "crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs",
    "crates/vize_patina/src/rules/vue/no_mutating_props.rs",
    "docs/davinci/plan/consumer-migration-surfaces/linter/vize_patina.tsv",
  ];
  assert.deepEqual(
    composition.changed.map((row) => row.path),
    changedPaths,
  );
  assert.equal(current.current.length, 323);
  assert.deepEqual(
    pr.current.map((row) => row.path),
    current.current.map((row) => row.path),
  );
  assert.deepEqual(
    main.current.map((row) => row.path),
    current.current.map((row) => row.path),
  );
  for (const key of Object.keys(current).filter((name) => name !== "current")) {
    assert.deepEqual(current[key], pr[key]);
    assert.deepEqual(current[key], main[key]);
  }
  const replacements = new Map();
  for (const row of composition.changed) {
    const own = [
      "crates/vize_patina/src/linter/script_rules.rs",
      "crates/vize_patina/src/linter/script_rules/sfc.rs",
    ].includes(row.path);
    const incoming = own ? "pr" : "main";
    for (const side of ["pr", "main", "after"]) {
      assert.equal(row[side].path, row.path);
      decodeInheritanceBody(row[side]);
      const identity = side === "after" ? current : side === "pr" ? pr : main;
      const { base64: _base64, ...wanted } = row[side];
      assert.deepEqual(
        identity.current.find((item) => item.path === row.path),
        { ...wanted, mode: "100644" },
      );
    }
    if (row.path.endsWith("vize_patina.tsv")) {
      let body = decodeInheritanceBody(row.main);
      for (const edit of composition.inventoryEdits.toReversed()) {
        const removed = Buffer.from(edit.before);
        assert.deepEqual(body.subarray(edit.offset, edit.offset + removed.length), removed);
        body = Buffer.concat([
          body.subarray(0, edit.offset),
          Buffer.from(edit.after),
          body.subarray(edit.offset + removed.length),
        ]);
      }
      assert.deepEqual(body, decodeInheritanceBody(row.after));
      let restored = body;
      for (const edit of composition.inventoryEdits) {
        const added = Buffer.from(edit.after);
        assert.deepEqual(restored.subarray(edit.offset, edit.offset + added.length), added);
        restored = Buffer.concat([
          restored.subarray(0, edit.offset),
          Buffer.from(edit.before),
          restored.subarray(edit.offset + added.length),
        ]);
      }
      assert.deepEqual(restored, decodeInheritanceBody(row.main));
    } else assert.deepEqual(decodeInheritanceBody(row.after), decodeInheritanceBody(row[incoming]));
    replacements.set(
      row.path,
      current.current.find((item) => item.path === row.path),
    );
  }
  for (const origin of [pr, main])
    assert.deepEqual(
      origin.current.map((row) => replacements.get(row.path) ?? row),
      current.current,
    );
  assert.deepEqual(
    composition.pins.map((row) => row.path),
    [
      "tests/tooling/support/no-mutating-component-join.mjs",
      "tests/tooling/no-mutating-component-join-source.test.mjs",
    ],
  );
  for (const [index, row] of composition.pins.entries()) {
    const after = decodeInheritanceBody(row.after).toString();
    for (const side of ["pr", "main"]) {
      const before = decodeInheritanceBody(row[side]).toString();
      const oldDigest = index === 0 ? composition[side].sha256 : composition.pins[0][side].sha256;
      const newDigest = index === 0 ? composition.after.sha256 : composition.pins[0].after.sha256;
      assert.equal(after, replaceOnce(before, oldDigest, newDigest));
    }
  }
  assert.deepEqual(successor.credit, { sourceOnly: true, native: false, performance: false });
  return successor;
}

export function loadInheritanceDocumentation(
  root,
  read = (name) => fs.readFileSync(path.join(root, name)),
) {
  const bytes = read(inheritanceDocumentationFixture);
  assert.equal(digest(bytes), "fea94ba9b889598d87fa46049380b5d836d3212a5ffabc267be64f6af780a05c");
  const successor = validateInheritanceDocumentation(JSON.parse(gunzipSync(bytes)));
  const originals = [successor.custody, successor.publicImportPrimary];
  for (const row of originals) assert.deepEqual(read(row.path), decodeInheritanceBody(row));
  for (const row of successor.documents)
    assert.deepEqual(read(row.path), decodeInheritanceBody(row.after));
  return successor;
}

export function createInheritanceFenceVerifier(root) {
  const successor = loadInheritanceDocumentation(root);
  const entry = JSON.parse(decodeInheritanceBody(successor.custody)).entries.find(
    (row) => row.id === "script/no-duplicate-attr-inheritance",
  );
  return (supplied, filename, fences) => {
    if (supplied.id !== entry.id) return false;
    assert.deepEqual(supplied, entry);
    const row = successor.documents.find((item) => item.path === filename);
    assert(row && [entry.path, entry.englishPath].includes(filename));
    assert.deepEqual(fences, codeFences(decodeInheritanceBody(row.after).toString()));
    return true;
  };
}
