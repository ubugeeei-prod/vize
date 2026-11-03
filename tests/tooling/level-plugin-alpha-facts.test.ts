import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";

const native = createRequire(import.meta.url)("../../npm/native/index.js");
const groups = [
  "component-signature",
  "prop-types",
  "emit-types",
  "slot-types",
  "reactivity-classes",
  "component-references",
];
const source = `<script setup lang="ts" generic="T extends 'a b'">
import ChildAlias from './Child.vue'
import type { Public as Alias } from './types'
import { ref } from 'vue'
defineOptions({ name: 'PublicComponent' })
const props = defineProps<{ label: 'a b'; choice: Alias; value?: T }>()
defineEmits<{ save: [value: T] }>()
defineSlots<{ default(props: { item: T }): any }>()
const model = defineModel<string>('title')
const count = ref(0)
defineExpose({ count })
function privateBody() { return 1 }
</script><template><ChildAlias>{{ props.label }} {{ privateBody() }}</ChildAlias></template>`;

function inspect(input: string, demands = groups) {
  let facts;
  const plugin = {
    name: "production-alpha-inspection",
    version: "1",
    fingerprint: "inspect-alpha-v1",
    cacheInputs: [],
    visit: [],
    demands,
    run(json: string) {
      const batch = JSON.parse(json);
      if (facts !== undefined) assert.deepEqual(batch.facts, facts);
      facts = batch.facts;
      return "[]";
    },
  };
  const output = native.lintWithPlugins(input, [plugin], {
    filename: "Public.vue",
    validateDeterminism: true,
  });
  assert.deepEqual(output.diagnostics, []);
  return facts as Record<string, [string, Record<string, any>][]>;
}

const row = (facts: ReturnType<typeof inspect>, group: string, name: string) => {
  const found = facts[group].find(([key]) => key === name);
  assert.ok(found, `${group} does not contain ${name}`);
  return found[1];
};

test("six production alpha groups cross the native batch with exact typed fields", () => {
  const facts = inspect(source);
  assert.deepEqual(Object.keys(facts).sort(), [...groups].sort());
  for (const group of groups) {
    assert.ok(facts[group].length > 0);
    for (const [key, contract] of facts[group]) {
      assert.equal(typeof key, "string");
      assert.equal(contract.schema, 1);
    }
  }
  const signature = row(facts, "component-signature", "Public.vue");
  assert.equal(signature.generic, "T extends 'a b'");
  assert.equal(signature.name, "Public.vue");
  assert.equal(signature.declared_name, "PublicComponent");
  assert.equal(signature.script_setup, true);
  // Source-only analysis has no resolved catalog proof and does not read disk.
  assert.equal(signature.props_complete, false);
  assert.equal(signature.with_defaults, null);
  assert.deepEqual(signature.prop_order, ["label", "choice", "value", "title"]);
  assert.deepEqual(signature.slot_order, ["default"]);
  const save = row(facts, "emit-types", "save");
  assert.equal(save.payload, "[value: T]");
  assert.deepEqual(save.overload_payloads, ["[value: T]"]);
  assert.equal(save.unresolved_type_arguments, null);
  assert.deepEqual(save.validator_type_annotations, []);
  const label = row(facts, "prop-types", "label");
  assert.equal(label.type, "'a b'");
  assert.equal(label.default, null);
  assert.equal(label.model_modifiers, null);
  const dependency = row(facts, "prop-types", "choice").type_dependencies;
  assert.equal(dependency.complete, false);
  const imported = dependency.declarations.find((entry: any) => entry.name === "Alias");
  assert.equal(imported.module, "./types");
  assert.equal(imported.export, "Public");
  assert.equal(imported.body, null);
  assert.ok(
    facts["component-references"].some(
      ([, value]) => value.module === "./Child.vue" && value.export === "default",
    ),
  );
  assert.ok(facts["reactivity-classes"].some(([, value]) => value.name === "count"));
});

test("alpha demands expose only their declared pages while beta demands compose", () => {
  const props = inspect(source, ["prop-types"]);
  assert.deepEqual(Object.keys(props), ["prop-types"]);
  const both = inspect(source, ["prop-types", "bindings"]);
  assert.deepEqual(Object.keys(both).sort(), ["bindings", "prop-types"]);
  assert.deepEqual(both["prop-types"], props["prop-types"]);
  assert.ok(both.bindings.some(([key]) => key === "count"));
});

test("body edits preserve alpha values and authored literal edits invalidate interfaces", () => {
  const before = inspect(source);
  const body = source
    .replace("return 1", "return 2")
    .replace("props.label }}", "props.label }} changed body");
  assert.deepEqual(inspect(body), before);
  const changed = inspect(source.replace("label: 'a b'", "label: 'ab'"));
  assert.notDeepEqual(changed["prop-types"], before["prop-types"]);
  assert.equal(row(changed, "prop-types", "label").type, "'ab'");
  assert.deepEqual(changed["component-references"], before["component-references"]);
});

test("alpha fact requests reject unsupported foreign scripts", () => {
  assert.throws(
    () =>
      inspect('<script setup lang="moonbit">let x=1</script><template>{{x}}</template>', [
        "prop-types",
      ]),
    /production JS plugin facts require a JavaScript or TypeScript script/,
  );
});

test("authored slot order survives canonical page sorting", () => {
  const ordered = source.replace(
    "defineSlots<{ default(props: { item: T }): any }>()",
    "defineSlots<{ zebra(props: { item: T }): any; default(props: {}): any }>()",
  );
  const facts = inspect(ordered);
  assert.deepEqual(row(facts, "component-signature", "Public.vue").slot_order, [
    "zebra",
    "default",
  ]);
  assert.deepEqual(
    facts["slot-types"].map(([key]) => key),
    ["default", "zebra"],
  );
});

test("every emit overload and unresolved generic crosses the native boundary", () => {
  const input = `<script setup lang="ts">
 type Later = string
 defineProps<{ stable: boolean }>()
 defineEmits<{ (event: 'save', value: number): void; (event: 'save', value: Later): void; stable: [] }>()
 </script><template><button /></template>`;
  const before = inspect(input);
  const save = row(before, "emit-types", "save");
  assert.equal(save.payload, "[value: number]");
  assert.deepEqual(save.overload_payloads, ["[value: number]", "[value: Later]"]);
  assert.equal(save.type_dependencies.complete, true);
  assert.equal(
    save.type_dependencies.declarations.find((entry: any) => entry.name === "Later").body,
    "string",
  );
  const changed = inspect(input.replace("Later = string", "Later = boolean"));
  assert.notDeepEqual(row(changed, "emit-types", "save"), save);
  assert.deepEqual(row(changed, "emit-types", "stable"), row(before, "emit-types", "stable"));
  assert.deepEqual(changed["prop-types"], before["prop-types"]);
  const generic = inspect(
    input.replace("(event: 'save', value: Later)", "<T extends Later>(event: 'save', value: T)"),
  );
  const unresolved = row(generic, "emit-types", "save");
  assert.deepEqual(unresolved.overload_payloads, ["[value: number]", null]);
  assert.match(unresolved.unresolved_type_arguments, /<T extends Later>/);
  assert.equal(unresolved.type_dependencies.complete, false);
  assert.ok(unresolved.type_dependencies.declarations.some((entry: any) => entry.name === "Later"));
});

test("authored foreign and external templates refuse before any JS callback", () => {
  for (const input of [
    '<template lang="pug">button {{ value }}</template>',
    '<template src="./external.html" />',
  ]) {
    assert.throws(() => inspect(input), /JS plugin visits require an inline HTML template/);
  }
});

test("unused bindings use the real primary producer and host byte spans", () => {
  const input = `<script setup>/* 日本語😀 */ const used = 1; const unused = 2;</script><template>{{ used }}</template>`;
  const facts = inspect(input, ["unused-bindings"]);
  const start = Buffer.byteLength(input.slice(0, input.indexOf("unused")));
  assert.deepEqual(facts, { "unused-bindings": [["unused", { span: [start, start + 6] }]] });
  const consumed = inspect(input.replace("{{ used }}", "{{ used }} {{ unused }}"), [
    "unused-bindings",
  ]);
  assert.deepEqual(consumed, { "unused-bindings": [] });
});

test("runtime validator headers carry public types while body edits preserve facts", () => {
  const input = `<script setup lang="ts">
 type Bound = { id: string }
 const shared = { save: <T extends Bound>(value: T, count: number = 1): boolean => { return true } }
 defineEmits({ ...shared, stable: (value: number): boolean => true })
 </script><template><button /></template>`;
  const before = inspect(input);
  const save = row(before, "emit-types", "save");
  assert.deepEqual(save.validator_signatures, [
    "<T extends Bound>(value: T, count?: number): boolean",
  ]);
  assert.equal(save.unresolved_type_arguments, null);
  assert.deepEqual(save.validator_type_annotations, []);
  assert.equal(save.type_dependencies.complete, false);
  assert.ok(save.type_dependencies.declarations.some((entry: any) => entry.name === "Bound"));
  const body = input
    .replace("return true", "return value.id.length > count")
    .replace("number = 1", "number = 42");
  assert.deepEqual(inspect(body), before);
  const changed = inspect(input.replace("id: string", "id: number"));
  assert.notDeepEqual(row(changed, "emit-types", "save"), save);
  assert.deepEqual(row(changed, "emit-types", "stable"), row(before, "emit-types", "stable"));
});

test("active runtime type annotations retain their actual declaration dependencies", () => {
  const input = `<script setup lang="ts">
 type Payload = string
 type Contract = { save: (value: Payload) => boolean }
 defineEmits(({ save: (value: Payload): boolean => true } satisfies Contract))
 </script><template><button /></template>`;
  const before = inspect(input);
  const save = row(before, "emit-types", "save");
  assert.deepEqual(save.validator_type_annotations, ["Contract"]);
  assert.ok(save.type_dependencies.declarations.some((entry: any) => entry.name === "Contract"));
  assert.ok(save.type_dependencies.declarations.some((entry: any) => entry.name === "Payload"));
  const changed = inspect(input.replace("Payload = string", "Payload = number"));
  assert.notDeepEqual(row(changed, "emit-types", "save"), save);
  assert.deepEqual(changed["prop-types"], before["prop-types"]);
});

test("missing property catalogs remain explicit independently of type closure", () => {
  const input = `<script setup lang="ts">
 import type { Public } from './missing'
 defineProps<Public>()
 </script><template><button /></template>`;
  const facts = inspect(input);
  const signature = row(facts, "component-signature", "Public.vue");
  assert.equal(signature.props_complete, false);
  assert.equal(signature.type_dependencies.complete, false);
  assert.deepEqual(signature.prop_order, []);
  assert.deepEqual(facts["prop-types"], []);
});

test("binding source anchors map reordered split scripts through the shared producer", () => {
  const input = `<template>日本語😀{{ used }}</template>
<script setup>const used = 1; const unused = 2</script>
<style>.button { color: red }</style>
<script lang="ts">export const outside = 3</script>`;
  const facts = inspect(input, ["bindings", "unused-bindings"]);
  for (const name of ["used", "unused", "outside"]) {
    const start = Buffer.byteLength(input.slice(0, input.indexOf(name, input.indexOf("<script"))));
    const binding = row(facts, "bindings", name);
    assert.deepEqual(binding.span, [start, start + name.length]);
  }
  const unusedStart = Buffer.byteLength(input.slice(0, input.indexOf("unused")));
  assert.deepEqual(facts["unused-bindings"], [
    ["unused", { span: [unusedStart, unusedStart + 6] }],
  ]);
  const refInput =
    '<script setup>import { ref } from "vue"; const panel = ref(null)</script><template><div ref="panel" /></template>';
  assert.deepEqual(inspect(refInput, ["unused-bindings"]), { "unused-bindings": [] });
});

test("withDefaults retains authored defaults from literal and constant objects", () => {
  const defaults = "{ label: 'a b', count: () => 2 }";
  const input = `<script setup lang="ts">
 const props = withDefaults(defineProps<{ label?: string; count?: number }>(), ${defaults})
 </script><template>{{ props.label }}</template>`;
  const before = inspect(input);
  assert.equal(row(before, "component-signature", "Public.vue").with_defaults, defaults);
  assert.equal(row(before, "prop-types", "label").default, "'a b'");
  assert.equal(row(before, "prop-types", "count").default, "() => 2");
  const constant = input
    .replace("const props =", `const defaults = ${defaults}; const props =`)
    .replace(`>(), ${defaults})`, ">(), defaults)");
  const resolved = inspect(constant);
  assert.equal(row(resolved, "component-signature", "Public.vue").with_defaults, "defaults");
  assert.deepEqual(resolved["prop-types"], before["prop-types"]);
  const changedConstant = inspect(constant.replace("'a b'", "'ab'"));
  assert.equal(row(changedConstant, "prop-types", "label").default, "'ab'");
  assert.notDeepEqual(changedConstant["prop-types"], resolved["prop-types"]);
  const changed = inspect(input.replace("'a b'", "'ab'"));
  assert.notDeepEqual(row(changed, "prop-types", "label"), row(before, "prop-types", "label"));
  assert.equal(row(changed, "prop-types", "label").default, "'ab'");
});
