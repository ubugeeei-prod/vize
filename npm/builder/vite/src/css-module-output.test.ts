// #11526, #11544: real compiler output carries its module shape and style
// bindings attach to the `_sfc_main` the assembly exported, so generating a
// module never parses the emitted code again. The probe
// runs in a child process because `oxc-parser` must be mocked before the
// output helpers load.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const child = spawnSync(
  process.execPath,
  [
    "--experimental-test-module-mocks",
    "--no-warnings",
    fileURLToPath(new URL("../test/css-module-output.ts", import.meta.url)),
  ],
  { cwd: new URL("..", import.meta.url), encoding: "utf8" },
);
assert.equal(child.status, 0, child.stderr);
type Probe = { parses: number; code: string };
const { out: results, noDefault }: { out: Record<string, Probe>; noDefault: Probe } = JSON.parse(
  child.stdout,
);

for (const [name, { parses, code }] of Object.entries(results)) {
  assert.equal(parses, 0, `${name}: generating the module must not re-parse it`);
  const exported = code.indexOf("export default _sfc_main");
  assert.ok(exported >= 0, `${name}: the component stays the default export`);
  const binding = name.startsWith("customElement")
    ? /_sfc_main\.styles = \[_style_0\];/
    : name.startsWith("optionsModule")
      ? /_sfc_main\.__cssModules\["classes"\] = classes;/
      : /_sfc_main\.__cssModules\["\$style"\] = \$style;/;
  assert.match(code, binding, `${name}: the style binding is attached`);
}
assert.equal(Object.keys(results).length, 7);
// Negative control: nothing is attached to a module that exports no `_sfc_main`.
assert.equal(noDefault.parses, 0);
assert.doesNotMatch(noDefault.code, /_sfc_main/, "no binding without an exported _sfc_main");
assert.match(noDefault.code, /import \$style from/, "its style import is still emitted");

console.log("✅ vite-plugin-vize CSS module output tests passed!");
