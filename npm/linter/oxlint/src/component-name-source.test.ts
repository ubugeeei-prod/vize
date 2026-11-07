import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { asArray, asRecord, asString, parseJson } from "../../../../tools/support/json-value.ts";
import { componentNameSource, componentNameSuccessorPath } from "./component-name-source.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const read = (path: string) => readFileSync(join(root, path));

void test("the casing source keeps the complete original and its exact finite successors", () => {
  const source = componentNameSource(root);
  assert.equal(source.historical.length, 16768);
  assert.equal(source.current.length, 16838);
  assert.deepEqual(asArray(source.successor.changes), [
    {
      ruleId: "vue/component-name-in-template-casing",
      before: "docs/content/rules/vue.md",
      after: "docs/content/rules/reference/vue-component-name-in-template-casing.md",
      occurrences: 1,
    },
    {
      ruleId: "script/custom-event-name-casing",
      before: "docs/content/rules/type-and-script.md",
      after: "docs/content/rules/reference/script-custom-event-name-casing.md",
      occurrences: 1,
    },
  ]);
  assert.deepEqual(asArray(source.successor.latestChanges), []);
  const prefix = '"src/Casing.vue",\n        r#"';
  const extract = (bytes: Buffer) => {
    const rust = bytes.toString("utf8"),
      start = rust.indexOf(prefix);
    assert(start >= 0);
    const end = rust.indexOf('"#,', start + prefix.length);
    assert(end > start);
    return rust.slice(start + prefix.length, end);
  };
  assert.equal(extract(source.historical), extract(source.current));
  for (const raw of asArray(source.successor.changes)) {
    const change = asRecord(raw);
    for (const locale of ["", "ja/"]) {
      const page = readFileSync(
        join(root, asString(change.after).replace("docs/content/", `docs/content/${locale}`)),
        "utf8",
      );
      assert.equal(/^title: "([^"]+)"$/mu.exec(page)?.[1], change.ruleId);
    }
  }
});

void test("casing source custody rejects wrong original/current, unknown successors, and missing history", () => {
  const source = componentNameSource(root);
  const originalPath = asString(asRecord(source.successor.historical).snapshot);
  const currentPath = asString(source.successor.path);
  for (const path of [
    originalPath,
    currentPath,
    asString(asRecord(source.successor.originalDescriptor).path),
  ]) {
    assert.throws(() =>
      componentNameSource(root, (name) =>
        name === path ? Buffer.concat([read(name), Buffer.from("x")]) : read(name),
      ),
    );
  }
  assert.throws(() =>
    componentNameSource(root, (name) => (name === currentPath ? source.historical : read(name))),
  );
  assert.throws(() =>
    componentNameSource(root, (name) => {
      if (name === originalPath) throw new Error("missing original consumer");
      return read(name);
    }),
  );
  for (const path of ["foreign/consumer.rs", "../escaped.rs"]) {
    const changed = asRecord(parseJson(read(componentNameSuccessorPath).toString("utf8")));
    changed.path = path;
    assert.throws(() =>
      componentNameSource(root, (name) =>
        name === componentNameSuccessorPath ? Buffer.from(JSON.stringify(changed)) : read(name),
      ),
    );
  }
});
