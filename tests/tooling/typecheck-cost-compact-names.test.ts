import assert from "node:assert/strict";
import { test } from "node:test";

import {
  normalizeAlias,
  sectionStarts,
  siteFamily,
  sourceText,
} from "../../tools/benchmarks/scripts/typecheck-cost-families.ts";

const byte = (text: string, needle: string) => {
  const index = text.indexOf(needle);
  assert.notEqual(index, -1, needle);
  return Buffer.byteLength(text.slice(0, index));
};

void test("compact generated names fold like the names they replace", () => {
  const compact = [
    "  // ========== Template Scope (inherits from setup) ==========",
    "  type __VProps0 = {};",
    "  type __VComp0 = typeof MkA;",
    "  type __VCheck0 = $PropChecker<__VComp0, __VProps0, {}>;",
    "  type __VProp0$to = string;",
    "  type __VModel0$value = string;",
    "  (undefined as unknown as __VCheck0)({",
    "",
  ].join("\n");
  const text = sourceText(Buffer.from(compact));
  const sections = sectionStarts(text);
  const family = (needle: string) => siteFamily(text, sections, byte(compact, needle)).family;
  assert.equal(family("type __VProps0"), "contract-alias");
  assert.equal(family("type __VComp0"), "contract-alias");
  assert.equal(family("type __VCheck0"), "contract-alias");
  assert.equal(family("type __VProp0$to"), "contract-alias");
  assert.equal(family("type __VModel0$value"), "model");
  assert.equal(family("(undefined as unknown as __VCheck0)"), "props-call");
  assert.equal(normalizeAlias("__VProps0"), normalizeAlias("__MkA_Props_0"));
  assert.equal(normalizeAlias("__VComp12"), "__C_Component_N");
  assert.equal(normalizeAlias("__VValueProps3"), "__C_ValueProps_N");
  assert.equal(normalizeAlias("__VCheckProps3"), "__C_CheckProps_N");
  assert.equal(normalizeAlias("__VCheckTail3"), "__C_CheckTail_N");
  assert.equal(normalizeAlias("__VCheck3"), "__C_Check_N");
  assert.equal(normalizeAlias("__VProp4$model_value"), "__C_N_prop_*");
  assert.equal(normalizeAlias("__VModel4$model_value"), "__C_N_model_*");
  assert.equal(normalizeAlias("__MkSwitch_4_model_model_value"), "__C_N_model_*");
});
