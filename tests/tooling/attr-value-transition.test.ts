import assert from "node:assert/strict";
import { test } from "node:test";
import {
  attrValuePredecessors,
  linePrefixAt,
} from "../../tools/support/compat/content-mapper/attr-value-transition.ts";

const owned = (value: string) =>
  `: $AttrValue<__WitnessChild_Props_0, "value", typeof WitnessChild, __WitnessChild_0_prop_value> = ${value} satisfies __WitnessChild_0_prop_value;\n`;
const name = "  const __vize_prop_check_0_value";

test("an owned #12458 check yields its plain predecessor line", () => {
  assert.deepEqual(attrValuePredecessors(name, owned("1")), [
    "  const __vize_prop_check_0_value: __WitnessChild_0_prop_value = 1;\n",
  ]);
  assert.deepEqual(attrValuePredecessors(name, owned("'日本 🧭'")), [
    "  const __vize_prop_check_0_value: __WitnessChild_0_prop_value = '日本 🧭';\n",
  ]);
});

test("a parenthesized owned value also yields its unwrapped predecessor", () => {
  assert.deepEqual(attrValuePredecessors(name, owned("(a ?? b)")), [
    "  const __vize_prop_check_0_value: __WitnessChild_0_prop_value = (a ?? b);\n",
    "  const __vize_prop_check_0_value: __WitnessChild_0_prop_value = a ?? b;\n",
  ]);
});

test("other directives and mismatched contracts are not certified", () => {
  for (const [prefix, generated] of [
    [name, "  const __vize_prop_check_0_value: __WitnessChild_0_prop_value = 1;\n"],
    [name, owned("1").replace(" satisfies __WitnessChild_0_prop_value", " satisfies __Other")],
    [name, owned("1").replace("$AttrValue<", "$PropValue<")],
    ["  const other", owned("1")],
    ["  void __vize_prop_check_0_value", owned("1")],
  ])
    assert.deepEqual(attrValuePredecessors(prefix!, generated!), []);
});

test("the line prefix is read in UTF-8 bytes from the line start", () => {
  const source = Buffer.from(`// 日本 🧭\n${name}${owned("1")}`);
  const start = source.indexOf(": $AttrValue<");
  assert.equal(linePrefixAt(source, start), name);
  assert.equal(linePrefixAt(source, 0), "");
});
