import assert from "node:assert/strict";
import { test } from "node:test";

import { generateOutput } from "./utils/index.ts";

void test("generateOutput ignores export default text inside template literals", () => {
  const output = generateOutput(
    {
      code: [
        "const message = `",
        "export default fake",
        "`;",
        "export function render() {",
        "  return message;",
        "}",
      ].join("\n"),
      scopeId: "literalexport",
      hasScoped: false,
      styles: [],
    },
    {
      isProduction: true,
      isDev: false,
    },
  );

  assert.match(output, /export default fake/);
  assert.doesNotMatch(output, /const _sfc_main = fake/);
  assert.match(output, /const _sfc_main = \{\};/);
  assert.match(output, /export default _sfc_main;/);
});

void test("generateOutput preserves pure annotations on default exports", () => {
  const output = generateOutput(
    {
      code: [
        'import { defineComponent } from "vue";',
        "export default /*#__PURE__*/ defineComponent({ name: 'Annotated' });",
      ].join("\n"),
      scopeId: "annotated",
      hasScoped: false,
      styles: [],
    },
    {
      isProduction: true,
      isDev: false,
    },
  );

  assert.match(output, /const _sfc_main = \/\*#__PURE__\*\/ defineComponent/);
  assert.match(output, /export default _sfc_main;/);
});
