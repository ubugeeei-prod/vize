import assert from "node:assert/strict";

import type { CompiledModule, CustomBlockInfo } from "./types.ts";
import { generateOutputWithMap } from "./utils/index.ts";

const filePath = "/src/App.vue";
const component = "const _sfc_main = {\n  name: 'App'\n};\nexport default _sfc_main\n";

function compiled(customBlocks: CustomBlockInfo[] | undefined): CompiledModule {
  return {
    code: component,
    scopeId: "msg12345",
    hasScoped: false,
    ...(customBlocks ? { customBlocks } : {}),
  };
}

const message: CustomBlockInfo = {
  type: "message",
  content: "Hello from custom block",
  src: null,
  attrs: {},
  index: 0,
};

const output = generateOutputWithMap(compiled([message]), {
  isProduction: true,
  isDev: false,
  filePath,
});

assert.match(
  output.code,
  /import block0 from "\/src\/App\.vue\?vue&type=message&index=0";/,
  "custom block import uses vue&type= and the block list index",
);
assert.match(
  output.code,
  /if \(typeof block0 === 'function'\) block0\(_sfc_main\)/,
  "a function default export is called with the component",
);
const callAt = output.code.indexOf("block0(_sfc_main)");
const exportAt = output.code.lastIndexOf("export default _sfc_main");
assert.ok(
  callAt >= 0 && exportAt >= 0 && callAt < exportAt,
  "the call runs after _sfc_main exists",
);

const templateOnly = generateOutputWithMap(
  {
    code: "export function render() {\n  return null;\n}\n",
    scopeId: "msg12345",
    hasScoped: false,
    customBlocks: [{ ...message, attrs: { lang: "txt" } }],
  },
  { isProduction: true, isDev: false, filePath },
);
assert.match(templateOnly.code, /const _sfc_main = \{ render \};/);
assert.match(
  templateOnly.code,
  /import block0 from "\/src\/App\.vue\?vue&type=message&index=0&lang=txt";/,
);
const templateCall = templateOnly.code.indexOf("block0(_sfc_main)");
const templateExport = templateOnly.code.lastIndexOf("export default _sfc_main");
const templateConst = templateOnly.code.indexOf("const _sfc_main = { render }");
assert.ok(
  templateConst >= 0 && templateConst < templateCall && templateCall < templateExport,
  "a template-only SFC calls the block after _sfc_main is created",
);

const srcBlock: CustomBlockInfo = {
  type: "message",
  content: "",
  src: "./message.txt",
  attrs: { src: "./message.txt" },
  index: 0,
};
const srcOutput = generateOutputWithMap(compiled([srcBlock]), {
  isProduction: true,
  isDev: false,
  filePath,
});
assert.match(
  srcOutput.code,
  /import block0 from "\/src\/App\.vue\?vue&type=message&index=0&src=true";/,
  "src custom blocks set src=true and omit the src attribute",
);

const absent = generateOutputWithMap(compiled(undefined), {
  isProduction: true,
  isDev: false,
  filePath,
});
assert.equal(
  absent.code.includes("block0"),
  false,
  "missing customBlocks emits no custom-block import",
);
assert.equal(
  absent.code,
  generateOutputWithMap(
    { code: component, scopeId: "msg12345", hasScoped: false },
    { isProduction: true, isDev: false, filePath },
  ).code,
);

console.log("✅ vite-plugin-vize custom block output tests passed!");
