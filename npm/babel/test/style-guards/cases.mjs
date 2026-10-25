import assert from "node:assert/strict";
import fs from "node:fs";
export const inputs = JSON.parse(
  fs.readFileSync(new URL("./fixtures/inputs.json", import.meta.url)),
);
assert.equal(inputs.length, 8);
const priority = JSON.parse(fs.readFileSync(new URL("./fixtures/priority.json", import.meta.url)));
assert.equal(priority.length, 8);
export const cases = [
  ...inputs,
  ...priority,
  ...["STYLE", "Style"].flatMap((spelling) =>
    ["before", "after"].map((order) => ({
      id: spelling.toLowerCase() + "-" + (spelling === "STYLE" ? "upper" : "mixed") + "-" + order,
      expression: '{"fontWeight":"700"}',
      fold: false,
      spelling,
      order,
    })),
  ),
];
assert.equal(cases.length, 20);
export function attributes(row, template = false, staticStyle = false) {
  const style = staticStyle
    ? `style="${row.referenceCss}"`
    : template
      ? ':style="styles"'
      : `style={${row.expression}}`;
  const additional = row.spelling ? `${row.spelling}="color:red"` : "";
  return `type="button" class="style-guard" ${row.order === "after" ? style + " " + additional : additional + " " + style}`;
}
export function source(row) {
  return `// 日本語 😀\nimport { ref } from "vue";\nexport const controls = {};\nconst metrics = globalThis.__styleGuardMetrics;\nexport const App = () => {\n  metrics.setup++;\n  const count = ref(0);\n  const increment = event => { metrics.clicks++; metrics.events.push({type:event.type,trusted:event.isTrusted}); count.value++; };\n  controls.reset = () => { count.value = 0; };\n  return <button ${attributes(row)} onClick={increment}>{count.value}</button>;\n};\n`.replaceAll(
    "\n",
    "\r\n",
  );
}
export function reference(row, staticStyle) {
  return `import * as Vue from "vue";import {render} from "./${row.id}-${staticStyle ? "static" : "dynamic"}-render.mjs";\nexport const controls={};\nexport const App=Vue.defineVaporComponent({setup(){const metrics=globalThis.__styleGuardMetrics;metrics.setup++;const count=Vue.ref(0);const increment=event=>{metrics.clicks++;metrics.events.push({type:event.type,trusted:event.isTrusted});count.value++;};controls.reset=()=>{count.value=0};return {count,increment,styles:${row.expression}};},render});\n`;
}
