import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { componentPropDefaults } from "../../npm/ui/scripts/reference-docs/component-defaults.ts";

const cases = [
  { component: "TextInput", count: 21 },
  { component: "TextArea", count: 15 },
] as const;

function source(component: string): { filename: string; text: string } {
  const filename = `npm/fresco/src/components/${component}.ts`;
  return { filename, text: readFileSync(new URL(`../../${filename}`, import.meta.url), "utf8") };
}

for (const { component, count } of cases) {
  void test(`${component} documents every optional input's actual Vue default`, () => {
    const { filename, text } = source(component);
    const records = componentPropDefaults(filename, text, `${component}Props`, component);
    assert.equal(records.length, count);
    for (const record of records)
      assert.equal(record.documented, record.runtime, `${component}.${record.name}`);
  });
}

void test("changing a runtime default cannot leave its documentation stale", () => {
  const { filename, text } = source("TextArea");
  const changed = text.replace(
    "rows: { type: Number, default: 5 }",
    "rows: { type: Number, default: 6 }",
  );
  assert.notEqual(changed, text);
  const records = componentPropDefaults(filename, changed, "TextAreaProps", "TextArea");
  assert.deepEqual(
    records.filter(({ documented, runtime }) => documented !== runtime),
    [{ name: "rows", documented: "5", runtime: "6" }],
  );
});

void test("explicit undefined overrides implicit Boolean casting in the focus alias", () => {
  const { filename, text } = source("TextInput");
  const changed = text.replace(
    "focused: {\n      type: Boolean,\n      default: undefined,\n    }",
    "focused: { type: Boolean }",
  );
  assert.notEqual(changed, text);
  const records = componentPropDefaults(filename, changed, "TextInputProps", "TextInput");
  assert.deepEqual(
    records.filter(({ documented, runtime }) => documented !== runtime),
    [{ name: "focused", documented: "undefined", runtime: "false" }],
  );
});

void test("missing documentation, new runtime props and computed defaults are refused", () => {
  const { filename, text } = source("TextArea");
  const missing = text.replace("@default 5", "default 5");
  assert.notEqual(missing, text);
  const records = componentPropDefaults(filename, missing, "TextAreaProps", "TextArea");
  assert.deepEqual(
    records.filter(({ documented, runtime }) => documented !== runtime),
    [{ name: "rows", documented: null, runtime: "5" }],
  );
  const added = text.replace("props: {", "props: { undocumented: String,");
  assert.throws(() => componentPropDefaults(filename, added, "TextAreaProps", "TextArea"), {
    message: "Undocumented runtime prop: TextArea.undocumented",
  });
  const computed = text.replace("default: 5", "default: computeRows()");
  assert.throws(() => componentPropDefaults(filename, computed, "TextAreaProps", "TextArea"), {
    message: "Nonliteral Vue default requires an explicit source contract: computeRows()",
  });
});

void test("merged interfaces and overridden runtime declarations cannot hide inputs", () => {
  const { filename, text } = source("TextInput");
  const merged = `${text}\nexport interface TextInputProps { extra?: string }\n`;
  assert.throws(() => componentPropDefaults(filename, merged, "TextInputProps", "TextInput"), {
    message: "Expected a complete local prop interface: TextInputProps",
  });
  const computed = text.replace('name: "TextInput",', 'name: "TextInput", ["pro" + "ps"]: {},');
  assert.notEqual(computed, text);
  assert.throws(() => componentPropDefaults(filename, computed, "TextInputProps", "TextInput"), {
    message: "Expected explicit Vue component options: TextInput",
  });
  const accessor = text.replace(
    'name: "TextInput",',
    'name: "TextInput", get props() { return {}; },',
  );
  assert.notEqual(accessor, text);
  assert.throws(() => componentPropDefaults(filename, accessor, "TextInputProps", "TextInput"), {
    message: "Expected one Vue prop declaration: TextInput",
  });
  for (const option of ["mixins: []", "extends: {}"]) {
    const inherited = text.replace('name: "TextInput",', `name: "TextInput", ${option},`);
    assert.notEqual(inherited, text);
    assert.throws(() => componentPropDefaults(filename, inherited, "TextInputProps", "TextInput"), {
      message: "Inherited Vue props require an explicit source contract: TextInput",
    });
  }
  const factory = text.replace('from "vue";', 'from "custom-vue";');
  assert.notEqual(factory, text);
  assert.throws(() => componentPropDefaults(filename, factory, "TextInputProps", "TextInput"), {
    message: "Expected defineComponent imported from Vue",
  });
});
